#![allow(unused_imports, dead_code)]
use filigree::auth::ObjectPermission;
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use sqlx_transparent_json_decode::sqlx_json_decode;
use ts_rs::TS;

use super::VideoId;
use crate::models::organization::OrganizationId;

#[derive(
    Serialize,
    Deserialize,
    Default,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    TS,
    schemars::JsonSchema,
    sqlx::Type,
)]
#[sqlx(rename_all = "snake_case", type_name = "text")]
#[serde(rename_all = "snake_case")]
pub enum VideoProcessingState {
    #[default]
    Queued,
    Downloaded,
    Extracted,
    Summarized,
}

#[derive(Deserialize, Debug, Clone, schemars::JsonSchema, sqlx::FromRow)]

pub struct Video {
    pub id: VideoId,
    pub organization_id: crate::models::organization::OrganizationId,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processing_state: crate::models::video::VideoProcessingState,
    pub url: Option<String>,
    pub images: Option<serde_json::Value>,
    pub title: Option<String>,
    pub duration: Option<i32>,
    pub author: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub metadata: Option<serde_json::Value>,
    pub read: bool,
    pub progress: i32,
    pub summary: Option<String>,
    pub processed_path: Option<String>,
    pub _permission: ObjectPermission,
}

pub type VideoPopulatedGet = Video;

pub type VideoPopulatedList = Video;

pub type VideoCreateResult = Video;

impl Video {
    // The <T as Default> syntax here is weird but lets us generate from the template without needing to
    // detect whether to add the extra :: in cases like DateTime::<Utc>::default

    pub fn default_id() -> VideoId {
        <VideoId as Default>::default().into()
    }

    pub fn default_organization_id() -> crate::models::organization::OrganizationId {
        <crate::models::organization::OrganizationId as Default>::default().into()
    }

    pub fn default_updated_at() -> chrono::DateTime<chrono::Utc> {
        <chrono::DateTime<chrono::Utc> as Default>::default().into()
    }

    pub fn default_created_at() -> chrono::DateTime<chrono::Utc> {
        <chrono::DateTime<chrono::Utc> as Default>::default().into()
    }

    pub fn default_processing_state() -> crate::models::video::VideoProcessingState {
        <crate::models::video::VideoProcessingState as Default>::default().into()
    }

    pub fn default_url() -> Option<String> {
        None
    }

    pub fn default_images() -> Option<serde_json::Value> {
        None
    }

    pub fn default_title() -> Option<String> {
        None
    }

    pub fn default_duration() -> Option<i32> {
        None
    }

    pub fn default_author() -> Option<String> {
        None
    }

    pub fn default_date() -> Option<chrono::NaiveDate> {
        None
    }

    pub fn default_metadata() -> Option<serde_json::Value> {
        None
    }

    pub fn default_read() -> bool {
        <bool as Default>::default().into()
    }

    pub fn default_progress() -> i32 {
        <i32 as Default>::default().into()
    }

    pub fn default_summary() -> Option<String> {
        None
    }

    pub fn default_processed_path() -> Option<String> {
        None
    }
}

sqlx_json_decode!(Video);

impl Default for Video {
    fn default() -> Self {
        Self {
            id: Self::default_id(),
            organization_id: Self::default_organization_id(),
            updated_at: Self::default_updated_at(),
            created_at: Self::default_created_at(),
            processing_state: Self::default_processing_state(),
            url: Self::default_url(),
            images: Self::default_images(),
            title: Self::default_title(),
            duration: Self::default_duration(),
            author: Self::default_author(),
            date: Self::default_date(),
            metadata: Self::default_metadata(),
            read: Self::default_read(),
            progress: Self::default_progress(),
            summary: Self::default_summary(),
            processed_path: Self::default_processed_path(),
            _permission: ObjectPermission::Owner,
        }
    }
}

impl Serialize for Video {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Video", 17)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("organization_id", &self.organization_id)?;
        state.serialize_field("updated_at", &self.updated_at)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("processing_state", &self.processing_state)?;
        state.serialize_field("url", &self.url)?;
        state.serialize_field("images", &self.images)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("duration", &self.duration)?;
        state.serialize_field("author", &self.author)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.serialize_field("read", &self.read)?;
        state.serialize_field("progress", &self.progress)?;
        state.serialize_field("summary", &self.summary)?;
        state.serialize_field("processed_path", &self.processed_path)?;
        state.serialize_field("_permission", &self._permission)?;
        state.end()
    }
}

#[derive(Deserialize, Debug, Clone, schemars::JsonSchema, sqlx::FromRow)]
#[cfg_attr(test, derive(Serialize))]
pub struct VideoCreatePayloadAndUpdatePayload {
    pub id: Option<VideoId>,
    pub processing_state: crate::models::video::VideoProcessingState,
    pub url: Option<String>,
    pub images: Option<serde_json::Value>,
    pub title: Option<String>,
    pub duration: Option<i32>,
    pub author: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub metadata: Option<serde_json::Value>,
    pub read: bool,
    pub progress: i32,
    pub summary: Option<String>,
    pub processed_path: Option<String>,
}

pub type VideoCreatePayload = VideoCreatePayloadAndUpdatePayload;

pub type VideoUpdatePayload = VideoCreatePayloadAndUpdatePayload;

impl VideoCreatePayloadAndUpdatePayload {
    // The <T as Default> syntax here is weird but lets us generate from the template without needing to
    // detect whether to add the extra :: in cases like DateTime::<Utc>::default

    pub fn default_id() -> Option<VideoId> {
        None
    }

    pub fn default_processing_state() -> crate::models::video::VideoProcessingState {
        <crate::models::video::VideoProcessingState as Default>::default().into()
    }

    pub fn default_url() -> Option<String> {
        None
    }

    pub fn default_images() -> Option<serde_json::Value> {
        None
    }

    pub fn default_title() -> Option<String> {
        None
    }

    pub fn default_duration() -> Option<i32> {
        None
    }

    pub fn default_author() -> Option<String> {
        None
    }

    pub fn default_date() -> Option<chrono::NaiveDate> {
        None
    }

    pub fn default_metadata() -> Option<serde_json::Value> {
        None
    }

    pub fn default_read() -> bool {
        <bool as Default>::default().into()
    }

    pub fn default_progress() -> i32 {
        <i32 as Default>::default().into()
    }

    pub fn default_summary() -> Option<String> {
        None
    }

    pub fn default_processed_path() -> Option<String> {
        None
    }
}

impl Default for VideoCreatePayloadAndUpdatePayload {
    fn default() -> Self {
        Self {
            id: Self::default_id(),
            processing_state: Self::default_processing_state(),
            url: Self::default_url(),
            images: Self::default_images(),
            title: Self::default_title(),
            duration: Self::default_duration(),
            author: Self::default_author(),
            date: Self::default_date(),
            metadata: Self::default_metadata(),
            read: Self::default_read(),
            progress: Self::default_progress(),
            summary: Self::default_summary(),
            processed_path: Self::default_processed_path(),
        }
    }
}
