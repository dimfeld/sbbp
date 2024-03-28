//! transcribe background job
#![allow(unused_imports, unused_variables, dead_code)]

use backon::Retryable;
use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::ResultExt;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::JobError;
use crate::{
    models::video::{VideoId, VideoProcessingState},
    server::ServerState,
};

/// The payload data for the transcribe background job
#[derive(Debug, Serialize, Deserialize)]
pub struct TranscribeJobPayload {
    pub id: VideoId,
    pub storage_prefix: String,
    pub audio_path: String,
}

/// Send the audio to a service for speech recognition
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: TranscribeJobPayload = job.json_payload().change_context(JobError::Payload)?;

    let start = tokio::time::Instant::now();

    // send it to deepgram
    let backoff = backon::ExponentialBuilder::default();

    let mut transcribe_result = (|| send_request(&state, payload.id, &payload.audio_path))
        .retry(&backoff)
        .when(|(retryable, _)| *retryable)
        .await
        .map_err(|(_, e)| e)
        .attach_printable_lazy(|| payload.audio_path.clone())?
        .error_for_status()
        .change_context(JobError::Transcribe)
        .attach_printable_lazy(|| payload.audio_path.clone())?
        .json::<serde_json::Value>()
        .await
        .change_context(JobError::Transcribe)
        .attach_printable_lazy(|| payload.audio_path.clone())?;

    // Tag it with the format so that we can more easily use the transcript elsewhere
    transcribe_result["_provider_format"] = "deepgram_v1".into();

    sqlx::query!(
        "UPDATE videos SET
        transcript = $2,
        metadata = metadata || $3
        WHERE id = $1",
        payload.id.as_uuid(),
        json!(transcribe_result),
        json!({
            "transcription": {
                "duration": start.elapsed().as_secs(),
            }
        }),
    )
    .execute(&state.db)
    .await
    .change_context(JobError::Db)?;

    super::summarize::enqueue(
        &state,
        payload.id,
        &super::summarize::SummarizeJobPayload { id: payload.id },
    )
    .await
    .change_context(JobError::Queue)?;

    Ok(())
}

async fn send_request(
    state: &ServerState,
    id: VideoId,
    audio_path: &str,
) -> Result<reqwest::Response, (bool, error_stack::Report<JobError>)> {
    // get audio stream from storage
    let audio = state
        .storage
        .uploads
        .get(audio_path)
        .await
        .change_context(JobError::StorageDownload)
        .map_err(|e| (false, e))?;
    let body = reqwest::Body::wrap_stream(audio.into_stream());

    let id = id.to_string();
    let response = state
        .filigree
        .http_client
        .post("https://api.deepgram.com/v1/listen")
        .header(
            "Authorization",
            format!("Token {}", &state.secrets.deepgram),
        )
        .header("Content-Type", "audio/mpeg")
        .query(&[
            ("paragraphs", "true"),
            ("punctuate", "true"),
            ("utterances", "true"),
            ("smart_format", "true"),
            ("tag", &id),
        ])
        // It comes back pretty quick even with long videos so just wait inline
        .timeout(std::time::Duration::from_secs(300))
        .body(body)
        .send()
        .await
        .change_context(JobError::Transcribe)
        .map_err(|e| (false, e))?
        .error_for_status();

    // TODO This and the "return a bool" are messy, make it work within the error
    let retryable = if let Err(e) = &response {
        let status = e.status().unwrap();
        status.is_server_error()
            || status == StatusCode::TOO_MANY_REQUESTS
            || status == StatusCode::REQUEST_TIMEOUT
    } else {
        false
    };

    response
        .change_context(JobError::Transcribe)
        .map_err(|e| (retryable, e))
}

/// Enqueue the transcribe job to run immediately
pub async fn enqueue(
    state: &ServerState,
    name: impl ToString,
    payload: &TranscribeJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .name(name)
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the transcribe job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    name: impl ToString,
    at: chrono::DateTime<chrono::Utc>,
    payload: &TranscribeJobPayload,
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
    let runner = JobRunner::builder("transcribe", run)
        .autoheartbeat(false)
        .format_failures_with_debug(true)
        .build();

    Ok(runner)
}

fn create_job_builder() -> JobBuilder {
    JobBuilder::new("transcribe").priority(1).weight(1)
}
