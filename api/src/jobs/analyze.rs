//! analyze background job
#![allow(unused_imports, unused_variables, dead_code)]

use std::{io::Write, path::Path};

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

const THUMBNAIL_SIZES: &[u32] = &[720, 1280, 1920];

/// Compare image similarity to see which ones we can remove
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: AnalyzeJobPayload = job.json_payload().change_context(JobError::Payload)?;

    let tempdir = TempDir::new().change_context(JobError::TempDir)?;
    let dir = tempdir.path();

    //  We could pipeline this better but it's simpler to just download all the images at once and
    //  then process them after.
    futures::stream::iter(1..=payload.max_index)
        .map(|index| {
            let filename = image_filename(index, None);
            let storage_path = format!("{}/{}", payload.storage_prefix, filename);
            Ok((index, filename, storage_path, state.clone()))
        })
        .try_for_each_concurrent(16, |(index, filename, storage_path, state)| async move {
            state
                .storage
                .images
                .stream_to_disk(&storage_path, dir.join(&filename))
                .await
                .change_context(JobError::StorageDownload)
                .attach_printable(filename)?;
            Ok::<(), Report<JobError>>(())
        })
        .await?;

    // Read the images and do structural similarity comparison on them
    let (removed, _) = try_join(
        ssim_pipeline(&payload, state.ssim_threshold, dir),
        thumbnail_pipeline(&state, &payload, dir),
    )
    .await?;

    sqlx::query!(
        "UPDATE videos
        SET images = images || $2
        WHERE id=$1",
        payload.id.as_uuid(),
        json!({
            "thumbnail_widths": THUMBNAIL_SIZES,
            "removed": removed,
        })
    )
    .execute(&state.db)
    .await
    .change_context(JobError::Db)?;

    Ok(())
}

async fn ssim_pipeline(
    payload: &AnalyzeJobPayload,
    threshold: f64,
    dir: &Path,
) -> Result<Vec<u32>, error_stack::Report<JobError>> {
    let mut removed = Vec::new();

    let filename = image_filename(payload.max_index, None);
    let mut last_image = image::open(dir.join(&filename))
        .change_context(JobError::ExtractingImages)
        .attach_printable(filename)?
        .into_rgb8();

    for i in 1..=payload.max_index {
        let this_filename = image_filename(i, None);
        let this_image = image::open(dir.join(&this_filename))
            .change_context(JobError::ReadImage)
            .attach_printable(this_filename)?
            .into_rgb8();

        let ssim = image_compare::rgb_hybrid_compare(&last_image, &this_image)
            .change_context(JobError::CalculatingSimilarity)?;
        if ssim.score >= threshold {
            // If this image is too similar to the previous kept image, then remove it.
            removed.push(i as u32);
        } else {
            // If it is different enough, then we keep it and this becomes the new base image.
            last_image = this_image;
        }
    }

    Ok(removed)
}

async fn thumbnail_pipeline(
    state: &ServerState,
    payload: &AnalyzeJobPayload,
    dir: &Path,
) -> Result<(), Report<JobError>> {
    futures::stream::iter(1..=payload.max_index)
        .map(|index| Ok((index, dir.join(&image_filename(index, None)))))
        .try_for_each_concurrent(8, |(index, disk_path)| async move {
            // Read the image
            let image = image::open(&disk_path)
                .change_context(JobError::ReadImage)
                .attach_printable_lazy(|| disk_path.display().to_string())?;
            let thumbnails = generate_thumbnails(image, &payload.storage_prefix, index).await?;
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
    image: DynamicImage,
    storage_prefix: &str,
    index: usize,
) -> Result<Vec<(String, Vec<u8>)>, error_stack::Report<JobError>> {
    tokio::task::spawn_blocking(move || {
        THUMBNAIL_SIZES
            .iter()
            .filter(|&size| image.width() > *size)
            .map(|&size| {
                let thumbnail = image.resize(size, size, image::imageops::FilterType::Lanczos3);

                let encoded = webp::Encoder::from_image(&thumbnail)
                    .map_err(|msg| JobError::WebPEncoder(msg.to_string()))
                    .change_context(JobError::Thumbnail)
                    .attach_printable_lazy(|| {
                        format!(
                            "input file {}, thumbnail size: {size}",
                            image_filename(index, None)
                        )
                    })?
                    .encode(70.0);

                let output = Vec::from(&*encoded);
                let filename = image_filename(index, Some(size as usize));
                Ok::<_, Report<JobError>>((filename, output))
            })
            .collect::<Result<Vec<_>, _>>()
    })
    .await
    .unwrap()
}

/// Enqueue the analyze job to run immediately
pub async fn enqueue(
    state: &ServerState,
    name: impl ToString,
    payload: &AnalyzeJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .name(name)
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the analyze job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    name: impl ToString,
    at: chrono::DateTime<chrono::Utc>,
    payload: &AnalyzeJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    // convert to time crate
    let timestamp = at.timestamp();
    let t = time::OffsetDateTime::from_unix_timestamp(timestamp)
        .map_err(|_| effectum::Error::TimestampOutOfRange("at"))?;

    create_job_builder()
        .name(name)
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
