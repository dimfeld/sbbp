#[allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use filigree::extract::ValidatedForm;
use maud::{html, Markup};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    pages::error::HtmlError,
    server::ServerState,
    Error,
};

mod auth;
mod docs;
mod error;
mod forgot;
mod generic_error;
pub mod layout;
mod login;
mod logout;
pub mod not_found;
mod reset;

pub use generic_error::*;
use layout::*;
pub use not_found::*;

#[derive(serde::Deserialize, serde::Serialize, Debug, JsonSchema)]
pub struct AddVideoActionPayload {
    pub url: String,
}

fn add_video_action_fragment() -> Markup {
    html! {}
}

async fn add_video_action(
    State(state): State<ServerState>,
    auth: Authed,
    form: ValidatedForm<AddVideoActionPayload>,
) -> Result<impl IntoResponse, Error> {
    let body = add_video_action_fragment();

    Ok(body)
}

fn rerun_stage_action_fragment() -> Markup {
    html! {}
}

async fn rerun_stage_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path((id, stage)): Path<(crate::models::video::VideoId, String)>,
) -> Result<impl IntoResponse, Error> {
    let body = rerun_stage_action_fragment();

    Ok(body)
}

#[derive(serde::Deserialize, serde::Serialize, Debug, JsonSchema)]
pub struct MarkReadActionPayload {
    pub read: bool,
}

fn mark_read_action_fragment() -> Markup {
    html! {}
}

async fn mark_read_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<crate::models::video::VideoId>,
    form: ValidatedForm<MarkReadActionPayload>,
) -> Result<impl IntoResponse, Error> {
    let body = mark_read_action_fragment();

    Ok(body)
}

fn get_image_action_fragment() -> Markup {
    html! {}
}

async fn get_image_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path(image_id): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let body = get_image_action_fragment();

    Ok(body)
}

async fn home_page(
    State(state): State<ServerState>,
    auth: Authed,
) -> Result<impl IntoResponse, HtmlError> {
    let body = html! {
    main ."relative p-4 flex flex-col gap-4" x-data="{ unread_only: true }" {
        form ."flex flex-col gap-2 rounded-lg border border-border p-4" hx-post="_action/add_video" {
            label ."flex gap-2 flex-1 max-w-[100ch] text-base" for="path" { "Add a new video" }
            div ."flex gap-2" {
                input #path .flex-1 type="text" name="url" autocomplete="off";
                button type="submit" { "Add" }
            }
        }

        div ."flex justify-end gap-2" {
            div ."flex items-center gap-2" {
                input #unread-only name="unread-only" type="checkbox" class="toggle" x-model="unread_only";
                label for="unread-only" { "Unread only" }
            }
        }
    }
    };

    Ok(root_layout_page(Some(&auth), "SBBP", body))
}

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new()
        .route("/", routing::get(home_page))
        .route(
            "/_action/add_video",
            routing::post(add_video_action)
                .route_layer(has_any_permission(vec!["Video:write", "org_admin"])),
        )
        .route(
            "/_action/videos/:id/rerun/:stage",
            routing::post(rerun_stage_action)
                .route_layer(has_any_permission(vec!["Video:owner", "org_admin"])),
        )
        .route(
            "/_action/videos/:id/mark_read",
            routing::post(mark_read_action)
                .route_layer(has_any_permission(vec!["Video:write", "org_admin"])),
        )
        .route(
            "/_action/image/:image_id",
            routing::get(get_image_action)
                .route_layer(has_any_permission(vec!["Video:read", "org_admin"])),
        )
        .merge(login::create_routes())
        .merge(logout::create_routes())
        .merge(forgot::create_routes())
        .merge(reset::create_routes())
        .merge(docs::create_routes())
}
