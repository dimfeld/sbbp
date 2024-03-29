//! extract background job
#![allow(unused_imports, unused_variables, dead_code)]

use std::path::Path;

use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::{Report, ResultExt};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use temp_dir::TempDir;
use tokio::fs::DirBuilder;

use super::{check_command_result, JobError};
use crate::{
    models::video::{VideoId, VideoImages, VideoProcessingState, VIDEO_IMAGE_TEMPLATE},
    server::ServerState,
};

/// The payload data for the extract background job
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractJobPayload {
    pub id: VideoId,
    pub storage_prefix: String,
    pub video_filename: String,
}

/// Extract images and audio from the video
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: ExtractJobPayload = job.json_payload().change_context(JobError::Payload)?;

    sqlx::query!(
        "UPDATE videos
        SET processing_state=$2
        WHERE id=$1",
        payload.id.as_uuid(),
        VideoProcessingState::Processing as _
    )
    .execute(&state.db)
    .await
    .change_context(JobError::Db)?;

    let temp_dir = TempDir::new().change_context(JobError::TempDir)?;
    let dir = temp_dir.path();
    let output_location = dir.join(&payload.video_filename);

    let video_storage_path = format!("{}/{}", payload.storage_prefix, payload.video_filename);
    state
        .storage
        .uploads
        .stream_to_disk(&video_storage_path, &output_location)
        .await
        .change_context(JobError::StorageDownload)?;

    let output_location = output_location.to_string_lossy();
    let ((audio_duration, audio_path), (image_duration, images)) = futures::try_join!(
        extract_audio(&state, payload.id, dir, &output_location),
        extract_images(&state, payload.id, dir, &output_location),
    )?;

    sqlx::query!(
        "UPDATE videos
        SET images=$2,
        metadata=metadata || $3
        WHERE id=$1",
        payload.id.as_uuid(),
        sqlx::types::Json(&images) as _,
        json!({
            "audio_extraction": {
                "duration": audio_duration.as_secs(),
                "filename": "audio.mp4",
            },
            "image_extraction": {
                "duration": image_duration.as_secs(),
            }
        })
    )
    .execute(&state.db)
    .await
    .change_context(JobError::Db)?;

    super::transcribe::enqueue(
        &state,
        payload.id,
        &super::transcribe::TranscribeJobPayload {
            id: payload.id,
            storage_prefix: payload.storage_prefix.clone(),
            audio_path,
        },
    )
    .await
    .change_context(JobError::Queue)?;

    super::analyze::enqueue(
        &state,
        payload.id,
        &super::analyze::AnalyzeJobPayload {
            id: payload.id,
            storage_prefix: payload.storage_prefix,
            max_index: images.max_index,
        },
    )
    .await
    .change_context(JobError::Queue)?;

    Ok(())
}

async fn extract_images(
    server: &ServerState,
    id: VideoId,
    dir: &Path,
    video_path: &str,
) -> Result<(std::time::Duration, VideoImages), Report<JobError>> {
    let start = tokio::time::Instant::now();
    let image_dir = dir.join("images");
    DirBuilder::new()
        .create(&image_dir)
        .await
        .change_context(JobError::TempDir)?;

    let image_template = image_dir.join(VIDEO_IMAGE_TEMPLATE);
    let interval = 10;
    let fps = format!("fps=1/{interval}");

    let result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            video_path,
            "-c:v",
            "libwebp",
            image_template.to_string_lossy().as_ref(),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .change_context(JobError::StartingFfmpeg)?
        .wait_with_output()
        .await
        .change_context(JobError::ExtractingImages)?;
    check_command_result(result, JobError::ExtractingImages)?;

    let file_list = tokio::fs::read_dir(image_dir)
        .await
        .change_context(JobError::ExtractingImages)?;
    let file_list = tokio_stream::wrappers::ReadDirStream::new(file_list)
        .try_collect::<Vec<_>>()
        .await
        .change_context(JobError::ExtractingImages)?;

    let num_files = file_list.len();

    futures::stream::iter(file_list)
        .map(Ok)
        .try_for_each_concurrent(16, |path| async move {
            let storage_path = format!("{}/{}", id, path.file_name().to_string_lossy());
            server
                .storage
                .images
                .upload_file(&path.path(), &storage_path)
                .await
                .change_context(JobError::StorageUpload)
                .attach_printable_lazy(|| storage_path.clone())
        })
        .await?;

    Ok((
        start.elapsed(),
        VideoImages {
            max_index: num_files - 1,
            interval,
            removed: Vec::new(),
        },
    ))
}

async fn extract_audio(
    server: &ServerState,
    id: VideoId,
    dir: &Path,
    video_path: &str,
) -> Result<(std::time::Duration, String), Report<JobError>> {
    let start = tokio::time::Instant::now();
    let audio_path = dir.join("audio.mp4");
    let result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            video_path,
            "-vn",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            audio_path.to_string_lossy().as_ref(),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .change_context(JobError::StartingFfmpeg)?
        .wait_with_output()
        .await
        .change_context(JobError::ExtractingAudio)?;

    check_command_result(result, JobError::ExtractingAudio)?;

    let storage_path = format!("{}/audio.mp4", id);
    server
        .storage
        .uploads
        .upload_file(&audio_path, &storage_path)
        .await
        .change_context(JobError::StorageUpload)
        .attach_printable_lazy(|| storage_path.clone())?;

    Ok((start.elapsed(), storage_path))
}

/// Enqueue the extract job to run immediately
pub async fn enqueue(
    state: &ServerState,
    name: impl ToString,
    payload: &ExtractJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .name(name)
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the extract job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    name: impl ToString,
    at: chrono::DateTime<chrono::Utc>,
    payload: &ExtractJobPayload,
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
    let runner = JobRunner::builder("extract", run)
        .autoheartbeat(false)
        .format_failures_with_debug(true)
        .build();

    Ok(runner)
}

fn create_job_builder() -> JobBuilder {
    JobBuilder::new("extract").priority(1).weight(1)
}
