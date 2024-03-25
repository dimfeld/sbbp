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
