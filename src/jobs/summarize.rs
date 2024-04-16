//! summarize background job
#![allow(unused_imports, unused_variables, dead_code)]

use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::ResultExt;
use filigree::testing::ResponseExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::JobError;
use crate::{models::video::VideoId, server::ServerState};

const SYSTEM_PROMPT: &str = "Your task is to summarize Youtube video transcripts. Clearly explain the topics discussed, and notable or surprising details, and the general sentiment around them.";
const SUMMARIZE_PROMPT_PREFIX: &str = "The video transcript follows:";
const SUMMARIZE_ASSISTANT_PREFIX: &str = "The summary of the above transcript is:";

/// The payload data for the summarize background job
#[derive(Debug, Serialize, Deserialize)]
pub struct SummarizeJobPayload {
    pub id: VideoId,
}

/// Send the transcript to an LLM for summarization
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: SummarizeJobPayload = job.json_payload().change_context(JobError::Payload)?;

    // Get the transcript from the database
    let transcript: serde_json::Value = sqlx::query_scalar!(
        "SELECT transcript FROM videos WHERE id = $1",
        payload.id.as_uuid()
    )
    .fetch_one(&state.db)
    .await
    .change_context(JobError::Db)?
    .ok_or(JobError::NoTranscript)
    .attach_printable("Video row had no transcript object")?;

    // Send it to the LLM for summary
    let transcript_text = transcript
        .pointer("/results/channels/0/alternatives/0/paragraphs/transcript")
        .and_then(|v| v.as_str())
        .ok_or(JobError::NoTranscript)
        .attach_printable("Found object but transcript was not at the expected path")?
        .trim();

    let summary_request = json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 768,
        "temperature": 0.5,
        "system": SYSTEM_PROMPT,
        "messages": [
            { "role": "user", "content": format!("{SUMMARIZE_PROMPT_PREFIX}:\n\n{transcript_text}") },
            { "role": "assistant", "content": SUMMARIZE_ASSISTANT_PREFIX }
        ]
    });

    let result: serde_json::Value = state
        .filigree
        .http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &state.secrets.anthropic)
        .header("anthropic-version", "2023-06-01")
        .json(&summary_request)
        .send()
        .await
        .change_context(JobError::Summarizing)?
        .log_error()
        .await
        .change_context(JobError::Summarizing)?
        .json()
        .await
        .change_context(JobError::Summarizing)?;

    let summary = result["content"][0]["text"]
        .as_str()
        .ok_or(JobError::Summarizing)
        .attach_printable("Failed to find summary in response")?;

    // Store the summary in the database and set processing_state to Ready
    sqlx::query!(
        "UPDATE videos SET summary = $1, processing_state = 'ready' WHERE id = $2",
        summary,
        payload.id.as_uuid()
    )
    .execute(&state.db)
    .await
    .change_context(JobError::Db)?;

    Ok(())
}

/// Enqueue the summarize job to run immediately
pub async fn enqueue(
    state: &ServerState,
    name: impl ToString,
    payload: &SummarizeJobPayload,
) -> Result<uuid::Uuid, effectum::Error> {
    create_job_builder()
        .name(name)
        .json_payload(payload)?
        .add_to(&state.queue)
        .await
}

/// Enqueue the summarize job to run at a specific time
pub async fn enqueue_at(
    state: &ServerState,
    name: impl ToString,
    at: chrono::DateTime<chrono::Utc>,
    payload: &SummarizeJobPayload,
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
    let runner = JobRunner::builder("summarize", run)
        .autoheartbeat(false)
        .format_failures_with_debug(true)
        .build();

    Ok(runner)
}

fn create_job_builder() -> JobBuilder {
    JobBuilder::new("summarize").priority(1).weight(1)
}
