#![allow(unused_imports, unused_variables, dead_code)]
use super::{VideoCreatePayload, VideoId, VideoUpdatePayload};

/// Generate a VideoCreatePayload for testing.
/// Parameter `i` controls the value of some of the fields, just to make sure that the objects
/// don't all look identical.
pub fn make_create_payload(i: usize) -> VideoCreatePayload {
    VideoCreatePayload {
        id: None,
        title: (i > 1).then(|| format!("Test object {i}")),
        read: i % 2 == 0,
        progress: i as i32,
    }
}

/// Generate a VideoUpdatePayload for testing.
/// Parameter `i` controls the value of some of the fields, just to make sure that the objects
/// don't all look identical.
pub fn make_update_payload(i: usize) -> VideoUpdatePayload {
    VideoUpdatePayload {
        id: None,
        title: Some(format!("Test object {i}")),
        read: i % 2 == 0,
        progress: i as i32,
    }
}
