//! Background jobs

pub mod analyze;
pub mod download;
pub mod extract;
pub mod summarize;

use std::path::Path;

use effectum::{Queue, Worker};
use futures::FutureExt;

use crate::server::ServerState;

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
}

pub struct QueueWorkers {
    pub analyze: Worker,
    pub download: Worker,
    pub ffmpeg: Worker,
    pub summarize: Worker,
}

impl QueueWorkers {
    pub async fn shutdown(self) {
        tokio::join!(
            self.analyze.unregister(None).map(|r| r.ok()),
            self.download.unregister(None).map(|r| r.ok()),
            self.ffmpeg.unregister(None).map(|r| r.ok()),
            self.summarize.unregister(None).map(|r| r.ok()),
        );
    }
}

pub async fn create_queue(queue_location: &Path) -> Result<Queue, effectum::Error> {
    Queue::new(queue_location).await
}

pub async fn init(
    state: &ServerState,
    init_recurring_jobs: bool,
) -> Result<QueueWorkers, effectum::Error> {
    // create the queue

    // register the jobs
    let analyze_runner = analyze::register(&state.queue, init_recurring_jobs).await?;
    let download_runner = download::register(&state.queue, init_recurring_jobs).await?;
    let extract_runner = extract::register(&state.queue, init_recurring_jobs).await?;
    let summarize_runner = summarize::register(&state.queue, init_recurring_jobs).await?;

    // create the workers
    let worker_analyze = Worker::builder(&state.queue, state.clone())
        .max_concurrency(4)
        .min_concurrency(4)
        .jobs([analyze_runner])
        .build()
        .await?;

    let worker_download = Worker::builder(&state.queue, state.clone())
        .max_concurrency(2)
        .min_concurrency(2)
        .jobs([download_runner])
        .build()
        .await?;

    let worker_ffmpeg = Worker::builder(&state.queue, state.clone())
        .max_concurrency(4)
        .min_concurrency(4)
        .jobs([extract_runner])
        .build()
        .await?;

    let worker_summarize = Worker::builder(&state.queue, state.clone())
        .max_concurrency(2)
        .min_concurrency(2)
        .jobs([summarize_runner])
        .build()
        .await?;

    let workers = QueueWorkers {
        analyze: worker_analyze,
        download: worker_download,
        ffmpeg: worker_ffmpeg,
        summarize: worker_summarize,
    };

    Ok(workers)
}
