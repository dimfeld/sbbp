//! analyze background job
#![allow(unused_imports, unused_variables, dead_code)]

use bytes::Bytes;
use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::{Report, ResultExt};
use futures::{
    future::{try_join, try_join_all},
    TryStreamExt,
};
use image::{DynamicImage, ImageBuffer};
use serde::{Deserialize, Serialize};
use serde_json::json;
use temp_dir::TempDir;
use tokio_stream::StreamExt;

use super::JobError;
use crate::{
    models::video::{image_filename, VideoId},
    server::ServerState,
};

/// The payload data for the analyze background job
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeJobPayload {
    pub id: VideoId,
    pub storage_prefix: String,
    pub max_index: usize,
}

/// Compare image similarity to see which ones we can remove
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: AnalyzeJobPayload = job.json_payload().change_context(JobError::Payload)?;

    let tempdir = TempDir::new().change_context(JobError::TempDir)?;
    let dir = tempdir.path();

    //  We could pipeline this better but it's simpler to just download all the images at once and
    //  then process them after.
    futures::stream::iter(0..=payload.max_index)
        .map(Ok)
        .try_for_each_concurrent(16, |index| async move {
            let filename = image_filename(index, None);
            let storage_path = format!("{}/{}", payload.storage_prefix, filename);
            state
                .storage
                .images
                .stream_to_disk(&storage_path, dir.join(filename))
                .await
                .change_context(JobError::StorageDownload)
                .attach_printable(filename)?;
            Ok::<(), Report<JobError>>(())
        })
        .await?;

    // Read the images and do structural similarity comparison on them
    try_join(thumbnail_pipeline(&state, &payload), todo!("ssim pipeline")).await?;

    Ok(())
}

async fn thumbnail_pipeline(
    state: &ServerState,
    payload: &AnalyzeJobPayload,
) -> Result<(), Report<JobError>> {
    futures::stream::iter(0..=payload.max_index)
        .map(Ok)
        .try_for_each_concurrent(8, |index| async move {
            // Read the image
            let image = todo!("load the image from disk");
            let thumbnails =
                generate_thumbnails(state, image, &payload.storage_prefix, index).await?;
            let uploads = thumbnails
                .into_iter()
                .map(|(filename, data)| async move {
                    let b = Bytes::from(data);
                    let storage_path = format!("{}/{}", payload.storage_prefix, filename);
                    state
                        .storage
                        .images
                        .put(&storage_path, b)
                        .await
                        .change_context(JobError::StorageUpload)
                        .attach_printable_lazy(|| format!("Thumbnail {storage_path}"))
                })
                .collect::<Vec<_>>();
            try_join_all(uploads).await?;
            Ok(())
        })
        .await
}

async fn generate_thumbnails(
    state: &ServerState,
    image: &DynamicImage,
    storage_prefix: &str,
    index: usize,
) -> Result<Vec<(String, Vec<u8>)>, error_stack::Report<JobError>> {
    tokio::task::spawn_blocking(|| {
        let sizes = [480, 720, 1080];
        sizes
            .iter()
            .filter(|&size| image.width() > *size)
            .map(|&size| {
                let thumbnail =
                    image.resize_to_fill(size, size, image::imageops::FilterType::Lanczos3);
                let mut output = std::io::Cursor::new(Vec::new());

                thumbnail
                    .write_to(&mut output, image::ImageFormat::WebP)
                    .change_context(JobError::Thumbnail)
                    .attach_printable_lazy(|| {
                        format!(
                            "input file {}, thumbnail size: {size}",
                            image_filename(index, None)
                        )
                    })?;

                let filename = image_filename(index, Some(size));
                Ok::<_, Report<JobError>>((filename, output.into_inner()))
            })
            .collect::<Result<Vec<_>, _>>()
    })
    .await
    .unwrap()
}

/// Enqueue the analyze job to run immediately
pub async fn enqueue(
    state: &ServerState,
    payload: &AnalyzeJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the analyze job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    payload: &AnalyzeJobPayload,
    at: chrono::DateTime<chrono::Utc>,
) -> Result<uuid::Uuid, effectum::Error> {
    // convert to time crate
    let timestamp = at.timestamp();
    let t = time::OffsetDateTime::from_unix_timestamp(timestamp)
        .map_err(|_| effectum::Error::TimestampOutOfRange("at"))?;

    create_job_builder()
        .json_payload(payload)?
        .run_at(t)
        .add_to(&state.queue)
        .await
}

/// Register this job with the queue and initialize any recurring jobs.
pub async fn register(
    queue: &Queue,
    init_recurring_jobs: bool,
) -> Result<JobRunner<ServerState>, effectum::Error> {
    let runner = JobRunner::builder("analyze", run)
        .autoheartbeat(false)
        .format_failures_with_debug(true)
        .build();

    Ok(runner)
}

fn create_job_builder() -> JobBuilder {
    JobBuilder::new("analyze").priority(1).weight(1)
}
