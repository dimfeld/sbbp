//! summarize background job
#![allow(unused_imports, unused_variables, dead_code)]

use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::ResultExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::JobError;
use crate::{models::video::VideoId, server::ServerState};

/// The payload data for the summarize background job
#[derive(Debug, Serialize, Deserialize)]
pub struct SummarizeJobPayload {
    pub id: VideoId,
}

/// Send the transcript to an LLM for summarization
async fn run(job: RunningJob, state: ServerState) -> Result<(), error_stack::Report<JobError>> {
    let payload: SummarizeJobPayload = job.json_payload().change_context(JobError::Payload)?;

    // Get the transcript from the database
    // Send it to the LLM for summary
    // Store the summary in the database and set processing_state to Ready

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
