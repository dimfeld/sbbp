pub mod endpoints;
pub mod queries;
#[cfg(test)]
pub mod testing;
pub mod types;

pub use types::*;

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

/// The output template used for ffmpeg when extracting images. This should be kept in sync with [image_filename]
pub const VIDEO_IMAGE_TEMPLATE: &str = "image-%05d.webp";
