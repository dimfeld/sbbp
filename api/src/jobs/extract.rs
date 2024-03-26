//! extract background job
#![allow(unused_imports, unused_variables, dead_code)]

use std::path::Path;

use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::ResultExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use temp_dir::TempDir;

use super::JobError;
use crate::{models::video::VideoId, server::ServerState};

/// The payload data for the extract background job
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractJobPayload {
    pub id: VideoId,
    pub storage_location: String,
}

/// Run the extract background job
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: ExtractJobPayload = job.json_payload().change_context(JobError::Payload)?;

    let temp_dir = TempDir::new().change_context(JobError::TempDir)?;
    let dir = temp_dir.path();
    // Get just the last path component to use as the filename
    let filename = payload
        .storage_location
        .rsplit_once('/')
        .map(|p| p.1)
        .unwrap_or(&payload.storage_location);
    let video_path = dir.join(filename);

    let output_location = dir.join(filename);
    state
        .storage
        .uploads
        .stream_to_disk(&payload.storage_location, &video_path)
        .await
        .change_context(JobError::StorageDownload)?;

    let (audio_result, images_result) = futures::join!(
        process_audio(dir, &video_path),
        extract_images(dir, &video_path),
    );

    Ok(())
}

async fn process_audio(dir: &Path, video_path: &Path) -> Result<(), error_stack::Report<JobError>> {
    let audio_path = dir.join("audio.wav");
    let result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            video_path.to_string_lossy().as_ref(),
            "-vn",
            "-acodec",
            "pcm_s16le",
            "-ar",
            "16000",
            "-ac",
            "1",
            audio_path.to_string_lossy().as_ref(),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .change_context(JobError::StartingFfmpeg)?
        .wait_with_output()
        .await
        .change_context(JobError::ExtractingAudio)?;
    // Check success

    // Send audio to deepgram

    Ok(())
}

async fn extract_images(
    dir: &Path,
    video_path: &Path,
) -> Result<(), error_stack::Report<JobError>> {
    let image_template = dir.join("image-%05d.webp");
    let interval = 10;
    let fps = format!("fps=1/{interval}");

    let result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            video_path.to_string_lossy().as_ref(),
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
    // check success

    // Find the resulting images
    // Upload them to storage
    // Do SSIM to determine similarity

    Ok(())
}

/// Enqueue the extract job to run immediately
pub async fn enqueue(
    state: &ServerState,
    payload: &ExtractJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the extract job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    payload: &ExtractJobPayload,
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
    let runner = JobRunner::builder("extract", run)
        .autoheartbeat(false)
        .format_failures_with_debug(true)
        .build();

    Ok(runner)
}

fn create_job_builder() -> JobBuilder {
    JobBuilder::new("extract").priority(1).weight(1)
}
