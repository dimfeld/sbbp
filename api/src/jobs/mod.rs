//! Background jobs
//!
//! The job flow is:
//! download leads to extract
//! extract leads to analyze and transcribe
//! transcribe leads to summarize

pub mod analyze;
pub mod download;
pub mod extract;
pub mod summarize;
pub mod transcribe;

use std::{os::unix::process::ExitStatusExt, path::Path};

use effectum::{Queue, Worker};
use error_stack::{Report, ResultExt};
use futures::FutureExt;

use crate::{server::ServerState, Error};

#[derive(thiserror::Error, Debug)]
enum JobError {
    #[error("Failed to read payload")]
    Payload,
    #[error("Failed to start video downloader")]
    StartingDownloader,
    #[error("Reading video.info.json")]
    ReadingInfoJson,
    #[error("Uploading to storage")]
    StorageUpload,
    #[error("Downloading from storage")]
    StorageDownload,
    #[error("Database error")]
    Db,
    #[error("Queue error")]
    Queue,
    #[error("Failed to create temporary directory")]
    TempDir,
    #[error("Failed to start ffmpeg")]
    StartingFfmpeg,
    #[error("Failed to extract audio")]
    ExtractingAudio,
    #[error("Failed to extract images")]
    ExtractingImages,
    #[error("Failed creating thumbnails")]
    Thumbnail,
    #[error("Failed reading image")]
    ReadImage,
    #[error("Failed calculating similarity")]
    CalculatingSimilarity,
    #[error("Failed to transcribe audio")]
    Transcribe,
    #[error("Video did not have a transcript")]
    NoTranscript,
    #[error("Failed calling summarize API")]
    Summarizing,
    /// Error encoding webp
    #[error("{0}")]
    WebPEncoder(String),
}

pub struct QueueWorkers {
    pub compute: Worker,
    pub download: Worker,
    pub summarize: Worker,
    pub transcribe: Worker,
}

impl QueueWorkers {
    pub async fn shutdown(self) {
        tokio::join!(
            self.compute.unregister(None).map(|r| r.ok()),
            self.download.unregister(None).map(|r| r.ok()),
            self.summarize.unregister(None).map(|r| r.ok()),
            self.transcribe.unregister(None).map(|r| r.ok()),
        );
    }
}

pub async fn create_queue(queue_location: &Path) -> Result<Queue, effectum::Error> {
    Queue::new(queue_location).await
}

pub async fn init(
    state: &ServerState,
    init_recurring_jobs: bool,
) -> Result<QueueWorkers, error_stack::Report<Error>> {
    // register the jobs
    let analyze_runner = analyze::register(&state.queue, init_recurring_jobs)
        .await
        .change_context(Error::TaskQueue)?;
    let download_runner = download::register(&state.queue, init_recurring_jobs)
        .await
        .change_context(Error::TaskQueue)?;
    let extract_runner = extract::register(&state.queue, init_recurring_jobs)
        .await
        .change_context(Error::TaskQueue)?;
    let summarize_runner = summarize::register(&state.queue, init_recurring_jobs)
        .await
        .change_context(Error::TaskQueue)?;
    let transcribe_runner = transcribe::register(&state.queue, init_recurring_jobs)
        .await
        .change_context(Error::TaskQueue)?;

    // create the workers
    let worker_compute_min_concurrency =
        filigree::config::parse_option::<u16>(std::env::var("WORKER_COMPUTE_MIN_CONCURRENCY").ok())
            .change_context(Error::Config)?
            .unwrap_or(8);
    let worker_compute_max_concurrency =
        filigree::config::parse_option::<u16>(std::env::var("WORKER_COMPUTE_MAX_CONCURRENCY").ok())
            .change_context(Error::Config)?
            .unwrap_or(8);

    let worker_compute = Worker::builder(&state.queue, state.clone())
        .min_concurrency(worker_compute_min_concurrency)
        .max_concurrency(worker_compute_max_concurrency)
        .jobs([analyze_runner, extract_runner])
        .build()
        .await
        .change_context(Error::TaskQueue)?;

    let worker_download_min_concurrency = filigree::config::parse_option::<u16>(
        std::env::var("WORKER_DOWNLOAD_MIN_CONCURRENCY").ok(),
    )
    .change_context(Error::Config)?
    .unwrap_or(2);
    let worker_download_max_concurrency = filigree::config::parse_option::<u16>(
        std::env::var("WORKER_DOWNLOAD_MAX_CONCURRENCY").ok(),
    )
    .change_context(Error::Config)?
    .unwrap_or(2);

    let worker_download = Worker::builder(&state.queue, state.clone())
        .min_concurrency(worker_download_min_concurrency)
        .max_concurrency(worker_download_max_concurrency)
        .jobs([download_runner])
        .build()
        .await
        .change_context(Error::TaskQueue)?;

    let worker_summarize_min_concurrency = filigree::config::parse_option::<u16>(
        std::env::var("WORKER_SUMMARIZE_MIN_CONCURRENCY").ok(),
    )
    .change_context(Error::Config)?
    .unwrap_or(16);
    let worker_summarize_max_concurrency = filigree::config::parse_option::<u16>(
        std::env::var("WORKER_SUMMARIZE_MAX_CONCURRENCY").ok(),
    )
    .change_context(Error::Config)?
    .unwrap_or(16);

    let worker_summarize = Worker::builder(&state.queue, state.clone())
        .min_concurrency(worker_summarize_min_concurrency)
        .max_concurrency(worker_summarize_max_concurrency)
        .jobs([summarize_runner])
        .build()
        .await
        .change_context(Error::TaskQueue)?;

    let worker_transcribe_min_concurrency = filigree::config::parse_option::<u16>(
        std::env::var("WORKER_TRANSCRIBE_MIN_CONCURRENCY").ok(),
    )
    .change_context(Error::Config)?
    .unwrap_or(16);
    let worker_transcribe_max_concurrency = filigree::config::parse_option::<u16>(
        std::env::var("WORKER_TRANSCRIBE_MAX_CONCURRENCY").ok(),
    )
    .change_context(Error::Config)?
    .unwrap_or(16);

    let worker_transcribe = Worker::builder(&state.queue, state.clone())
        .min_concurrency(worker_transcribe_min_concurrency)
        .max_concurrency(worker_transcribe_max_concurrency)
        .jobs([transcribe_runner])
        .build()
        .await
        .change_context(Error::TaskQueue)?;

    let workers = QueueWorkers {
        compute: worker_compute,
        download: worker_download,
        summarize: worker_summarize,
        transcribe: worker_transcribe,
    };

    Ok(workers)
}

/// Check the result of a command, and if it failed return the specified error with additional
/// information attached.
fn check_command_result(
    output: std::process::Output,
    err: JobError,
) -> Result<std::process::Output, Report<JobError>> {
    if output.status.success() {
        Ok(output)
    } else {
        let exit_status = if let Some(signal) = output.status.signal() {
            format!("Received signal {signal}")
        } else if let Some(code) = output.status.code() {
            format!("Exited with code {code}")
        } else {
            "Failed for unknown reason".to_string()
        };

        Err(Report::new(err))
            .attach_printable(exit_status)
            .attach_printable(format!(
                "stdout: {}",
                String::from_utf8(output.stdout).unwrap_or_default()
            ))
            .attach_printable(format!(
                "stderr: {}",
                String::from_utf8(output.stderr).unwrap_or_default()
            ))
    }
}
