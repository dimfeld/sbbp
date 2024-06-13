pub mod endpoints;
pub mod queries;
#[cfg(test)]
pub mod testing;
pub mod types;

use error_stack::{Report, ResultExt};
pub use types::*;
use uuid::Uuid;

use crate::{
    auth::Authed,
    jobs::{
        analyze::AnalyzeJobPayload, download::DownloadJobPayload, extract::ExtractJobPayload,
        summarize::SummarizeJobPayload, transcribe::TranscribeJobPayload,
    },
    server::ServerState,
    Error,
};

pub const READ_PERMISSION: &str = "Video::read";
pub const WRITE_PERMISSION: &str = "Video::write";
pub const OWNER_PERMISSION: &str = "Video::owner";

pub const CREATE_PERMISSION: &str = "Video::owner";

filigree::make_object_id!(VideoId, vid);

/// The filename for images associated with a video. This should be kept in sync with [VIDEO_IMAGE_TEMPLATE].
pub fn image_filename(index: usize, width: Option<usize>) -> String {
    if let Some(width) = width {
        format!("image-{:05}-{width}w.webp", index)
    } else {
        format!("image-{:05}.webp", index)
    }
}

pub const THUMBNAIL_FILENAME: &str = "thumbnail.webp";

/// The output template used for ffmpeg when extracting images. This should be kept in sync with [image_filename]
pub const VIDEO_IMAGE_TEMPLATE: &str = "image-%05d.webp";

pub async fn create_via_url(
    state: &ServerState,
    auth: &Authed,
    url: &str,
) -> Result<VideoId, Report<Error>> {
    let id = VideoId::new();
    sqlx::query!(
        "INSERT INTO videos (id, organization_id, processing_state, url, metadata) VALUES
        ($1, $2, $3, $4, '{}'::jsonb)",
        id.as_uuid(),
        auth.organization_id.as_uuid(),
        VideoProcessingState::Queued as _,
        url
    )
    .execute(&state.db)
    .await
    .change_context(Error::Db)?;

    crate::jobs::download::enqueue(
        &state,
        id,
        &crate::jobs::download::DownloadJobPayload {
            id,
            download_url: url.to_string(),
            storage_prefix: id.to_string(),
        },
    )
    .await
    .change_context(Error::TaskQueue)
    .attach_printable("Failed to enqueue download job")?;

    Ok(id)
}

pub async fn rerun_stage(
    state: &ServerState,
    auth: &Authed,
    id: VideoId,
    stage: &str,
) -> Result<Uuid, Error> {
    let video = queries::get(&state.db, auth, &id).await?;
    let storage_prefix = video.id.to_string();

    let result = match stage {
        "download" => {
            let url = video
                .url
                .ok_or(Error::NotFound("Video URL"))
                .attach_printable("Missing URL")?;
            crate::jobs::download::enqueue(
                &state,
                id,
                &DownloadJobPayload {
                    id: video.id,
                    storage_prefix,
                    download_url: url,
                },
            )
            .await
        }
        "extract" => {
            let video_filename = video
                .metadata
                .as_ref()
                .and_then(|m| m.download.as_ref())
                .and_then(|d| d.filename.as_ref())
                .map(|s| s.to_string())
                .ok_or(Error::NotFound("Video not downloaded yet"))?;
            crate::jobs::extract::enqueue(
                &state,
                id,
                &ExtractJobPayload {
                    id,
                    storage_prefix,
                    video_filename,
                },
            )
            .await
        }
        "analyze" => {
            let max_index = video
                .images
                .map(|i| i.max_index)
                .ok_or(Error::NotFound("Video not extracted yet"))?;
            crate::jobs::analyze::enqueue(
                &state,
                id,
                &AnalyzeJobPayload {
                    id,
                    storage_prefix,
                    max_index,
                },
            )
            .await
        }
        "transcribe" => {
            crate::jobs::transcribe::enqueue(
                &state,
                id,
                &TranscribeJobPayload {
                    id,
                    audio_path: format!("{storage_prefix}/audio.mp4"),
                    storage_prefix,
                },
            )
            .await
        }
        "summarize" => {
            crate::jobs::summarize::enqueue(&state, id, &SummarizeJobPayload { id }).await
        }
        _ => return Err(Error::NotFound("Unknown stage")),
    };

    let job_id = result.change_context(Error::TaskQueue)?;

    Ok(job_id)
}
