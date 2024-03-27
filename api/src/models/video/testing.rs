#![allow(unused_imports, unused_variables, dead_code)]
use super::{VideoCreatePayload, VideoId, VideoUpdatePayload};

/// Generate a VideoCreatePayload for testing.
/// Parameter `i` controls the value of some of the fields, just to make sure that the objects
/// don't all look identical.
pub fn make_create_payload(i: usize) -> VideoCreatePayload {
    VideoCreatePayload {
        id: None,
        processing_state: super::VideoProcessingState::Queued,
        url: (i > 1).then(|| format!("Test object {i}")),
        title: (i > 1).then(|| format!("Test object {i}")),
        duration: (i > 1).then(|| i as i32),
        author: (i > 1).then(|| format!("Test object {i}")),
        date: (i > 1).then(|| <chrono::NaiveDate as Default>::default()),
        metadata: (i > 1).then(|| serde_json::json!({ "key": i })),
        read: i % 2 == 0,
        progress: i as i32,
        images: (i > 1).then(|| <crate::models::video::VideoImages as Default>::default()),
        transcript: (i > 1).then(|| serde_json::json!({ "key": i })),
        summary: (i > 1).then(|| format!("Test object {i}")),
        processed_path: (i > 1).then(|| format!("Test object {i}")),
    }
}

/// Generate a VideoUpdatePayload for testing.
/// Parameter `i` controls the value of some of the fields, just to make sure that the objects
/// don't all look identical.
pub fn make_update_payload(i: usize) -> VideoUpdatePayload {
    VideoUpdatePayload {
        id: None,
        processing_state: super::VideoProcessingState::Processing,
        url: Some(format!("Test object {i}")),
        title: Some(format!("Test object {i}")),
        duration: Some(i as i32),
        author: Some(format!("Test object {i}")),
        date: Some(<chrono::NaiveDate as Default>::default()),
        metadata: Some(serde_json::json!({ "key": i })),
        read: i % 2 == 0,
        progress: i as i32,
        images: Some(<crate::models::video::VideoImages as Default>::default()),
        transcript: Some(serde_json::json!({ "key": i })),
        summary: Some(format!("Test object {i}")),
        processed_path: Some(format!("Test object {i}")),
    }
}
