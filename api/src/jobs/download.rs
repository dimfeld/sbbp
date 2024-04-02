//! download background job
#![allow(unused_imports, unused_variables, dead_code)]

use std::path::{Path, PathBuf};

use bytes::Bytes;
use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use temp_dir::TempDir;
use tokio::io::AsyncWriteExt;

use super::{check_command_result, JobError};
use crate::{
    models::video::{VideoId, VideoProcessingState},
    server::ServerState,
};

/// The payload data for the download background job
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadJobPayload {
    pub id: VideoId,
    pub download_url: String,
    pub storage_prefix: String,
}

#[derive(Serialize, Deserialize)]
struct InfoJson {
    ext: String,
    title: String,
    aspect_ratio: f64,
    duration: usize,
    release_date: Option<String>,
    upload_date: String,
    uploader: String,
    webpage_url: String,
}

/// Run the download background job
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: DownloadJobPayload = job.json_payload().change_context(JobError::Payload)?;
    let start = tokio::time::Instant::now();

    sqlx::query!(
        "UPDATE videos SET
        processing_state=$2
        WHERE id=$1
        ",
        payload.id.as_uuid(),
        VideoProcessingState::Downloading as _,
    )
    .execute(&state.db)
    .await
    .change_context(JobError::Db)?;

    let temp_dir = TempDir::new().change_context(JobError::TempDir)?;
    let download_dir = temp_dir.path();
    let dirname = payload
        .download_url
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let video_path_template = download_dir.join("video.%(ext)s");
    // Place the infojson at a fixed path
    let infojson_path_template = format!("infojson:{}", download_dir.join("video").display());

    let download_process = tokio::process::Command::new("yt-dlp")
        .args([
            "--write-info-json",
            "--output",
            video_path_template.to_string_lossy().as_ref(),
            "--output",
            &infojson_path_template,
            &payload.download_url,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .change_context(JobError::StartingDownloader)?;

    let result = download_process
        .wait_with_output()
        .await
        .change_context(JobError::StartingDownloader)?;
    // check success

    let info_json_path = download_dir.join("video.info.json");
    let info_json_buffer = tokio::fs::read(&info_json_path)
        .await
        .change_context(JobError::ReadingInfoJson)?;
    let info_json: InfoJson =
        serde_json::from_slice(&info_json_buffer).change_context(JobError::ReadingInfoJson)?;
    let video_fs_path = format!("{}/video.{}", download_dir.display(), info_json.ext);

    let info_json_bytes = Bytes::from(info_json_buffer);
    let info_json_storage_path = format!("{}/video.info.json", payload.storage_prefix);
    state
        .storage
        .uploads
        .put(&info_json_storage_path, info_json_bytes)
        .await
        .change_context(JobError::StorageUpload)
        .attach_printable(info_json_storage_path)?;

    let video_filename = format!("video.{}", info_json.ext);
    let video_storage_path = format!("{}/{}", payload.storage_prefix, video_filename);

    state
        .storage
        .uploads
        .upload_file(&video_fs_path, &video_storage_path)
        .await
        .change_context(JobError::StorageUpload)
        .attach_printable_lazy(|| video_storage_path.clone())?;

    let elapsed = start.elapsed();

    sqlx::query!(
        "UPDATE videos SET
        processing_state=$2,
        title=$3,
        duration=$4,
        author=$5,
        processed_path=$6,
        metadata=metadata || $7
        WHERE id=$1
        ",
        payload.id.as_uuid(),
        VideoProcessingState::Downloaded as _,
        &info_json.title,
        info_json.duration as i32,
        &info_json.uploader,
        video_storage_path,
        json!({
            "download": {
                "duration": elapsed.as_secs(),
                "filename": video_filename,
            }
        }),
    )
    .execute(&state.db)
    .await
    .change_context(JobError::Db)?;

    super::extract::enqueue(
        &state,
        payload.id,
        &super::extract::ExtractJobPayload {
            id: payload.id,
            storage_prefix: payload.storage_prefix,
            video_filename,
        },
    )
    .await
    .change_context(JobError::Queue)?;

    Ok(())
}

/// Enqueue the download job to run immediately
pub async fn enqueue(
    state: &ServerState,
    name: impl ToString,
    payload: &DownloadJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .name(name)
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the download job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    name: impl ToString,
    at: chrono::DateTime<chrono::Utc>,
    payload: &DownloadJobPayload,
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
    let runner = JobRunner::builder("download", run)
        .autoheartbeat(false)
        .format_failures_with_debug(true)
        .build();

    Ok(runner)
}

fn create_job_builder() -> JobBuilder {
    JobBuilder::new("download").priority(1).weight(1)
}
