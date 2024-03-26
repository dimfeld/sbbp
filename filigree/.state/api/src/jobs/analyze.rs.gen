//! analyze background job
#![allow(unused_imports, unused_variables, dead_code)]

use effectum::{JobBuilder, JobRunner, Queue, RecurringJobSchedule, RunningJob};
use error_stack::ResultExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::server::ServerState;

#[derive(thiserror::Error, Debug)]
enum AnalyzeJobError {
    #[error("Failed to read payload")]
    Payload,
}

/// The payload data for the analyze background job
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeJobPayload {
    // Fill in your payload data here
}

/// Run the analyze background job
async fn run(
    job: RunningJob,
    state: ServerState,
) -> Result<(), error_stack::Report<AnalyzeJobError>> {
    let payload: AnalyzeJobPayload = job
        .json_payload()
        .change_context(AnalyzeJobError::Payload)?;

    Ok(())
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
