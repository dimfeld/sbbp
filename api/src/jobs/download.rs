//! download background job
#![allow(unused_imports, unused_variables, dead_code)]

use bytes::Bytes;
use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::ResultExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::{models::video::VideoId, server::ServerState};

#[derive(thiserror::Error, Debug)]
enum DownloadJobError {
    #[error("Failed to read payload")]
    Payload,
    #[error("Failed to start downloader")]
    StartingDownloader,
    #[error("Reading video.info.json")]
    ReadingInfoJson,
    #[error("Uploading to storage")]
    Uploading,
    #[error("Database error")]
    Db,
}

/// The payload data for the download background job
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadJobPayload {
    id: VideoId,
    download_url: String,
}

#[derive(Serialize, Deserialize)]
struct InfoJson {
    ext: String,
    title: String,
    aspect_ratio: f64,
    duration: usize,
    release_date: String,
    upload_date: String,
    uploader: String,
    webpage_url: String,
}

/// Run the download background job
async fn run(
    job: RunningJob,
    state: ServerState,
) -> Result<(), error_stack::Report<DownloadJobError>> {
    let payload: DownloadJobPayload = job
        .json_payload()
        .change_context(DownloadJobError::Payload)?;

    let temp = std::env::temp_dir();
    let dirname = payload
        .download_url
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let download_dir = temp.join(&dirname);
    let video_path_template = download_dir.join("video.%(ext)s");
    let infojson_path_template = download_dir.join("infojson:video");

    let download_process = tokio::process::Command::new("yt-dlp")
        .arg("--write-info-json")
        .arg("--output")
        .arg(&video_path_template)
        // Place the infojson at a fixed path
        .arg("--output")
        .arg(&infojson_path_template)
        .arg(&payload.download_url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .change_context(DownloadJobError::StartingDownloader)?;

    let result = download_process
        .wait_with_output()
        .await
        .change_context(DownloadJobError::StartingDownloader)?;

    let info_json_path = download_dir.join("video.info.json");
    let info_json_buffer = tokio::fs::read(&info_json_path)
        .await
        .change_context(DownloadJobError::ReadingInfoJson)?;
    let info_json: InfoJson = serde_json::from_slice(&info_json_buffer)
        .change_context(DownloadJobError::ReadingInfoJson)?;

    let info_json_bytes = Bytes::from(info_json_buffer);
    let infojson_storage_path = format!("{}/video.info.json", download_dir.display());
    state
        .storage
        .uploads
        .put(&infojson_storage_path, info_json_bytes)
        .await
        .change_context(DownloadJobError::Uploading)
        .attach_printable(infojson_storage_path)?;

    let video_fs_path = format!("{}/video.{}", download_dir.display(), info_json.ext);
    let video_storage_path = format!("{}/video.{}", dirname, info_json.ext);
    let video_reader = tokio::fs::File::open(&video_fs_path)
        .await
        .change_context(DownloadJobError::Uploading)
        .attach_printable(video_fs_path)?;
    let mut video_reader = tokio::io::BufReader::with_capacity(32 * 1024 * 1024, video_reader);

    let (id, mut writer) = state
        .storage
        .uploads
        .put_multipart(&video_storage_path)
        .await
        .change_context(DownloadJobError::Uploading)
        .attach_printable_lazy(|| video_storage_path.clone())?;

    tokio::io::copy_buf(&mut video_reader, &mut writer)
        .await
        .change_context(DownloadJobError::Uploading)
        .attach_printable_lazy(|| video_storage_path.clone())?;
    writer
        .flush()
        .await
        .change_context(DownloadJobError::Uploading)
        .attach_printable_lazy(|| video_storage_path.clone())?;

    sqlx::query!(
        "UPDATE videos SET
        title=$2,
        duration=$3,
        processed_path=$4,
        metadata=$5,
        processing_state='downloaded'
        WHERE id=$1
        ",
        payload.id.as_uuid(),
        &info_json.title,
        info_json.duration as i32,
        video_storage_path,
        sqlx::types::Json(&info_json) as _
    )
    .execute(&state.db)
    .await
    .change_context(DownloadJobError::Db)?;

    Ok(())
}

/// Enqueue the download job to run immediately
pub async fn enqueue(
    state: &ServerState,
    payload: &DownloadJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the download job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    payload: &DownloadJobPayload,
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
    let runner = JobRunner::builder("download", run)
        .autoheartbeat(false)
        .format_failures_with_debug(true)
        .build();

    Ok(runner)
}

fn create_job_builder() -> JobBuilder {
    JobBuilder::new("download").priority(1).weight(1)
}
