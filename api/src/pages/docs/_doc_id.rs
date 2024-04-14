#![allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_extra::extract::{Form, Query};
use filigree::extract::ValidatedForm;
use maud::{html, Markup};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    pages::{error::HtmlError, layout::root_layout_page},
    server::ServerState,
    Error,
};

fn get_image_action_fragment() -> Markup {
    html! {}
}

async fn get_image_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path((doc_id, image_id)): Path<(crate::models::video::VideoId, String)>,
) -> Result<impl IntoResponse, Error> {
    let body = get_image_action_fragment();

    Ok(body)
}

async fn docs_page(
    State(state): State<ServerState>,
    auth: Authed,
    Path(doc_id): Path<crate::models::video::VideoId>,
) -> Result<impl IntoResponse, HtmlError> {
    let body = html! {};

    Ok(root_layout_page(Some(&auth), "title", body))
}

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new()
        .route("/docs/:doc_id", routing::get(docs_page))
        .route(
            "/docs/:doc_id/_action/image/:image_id",
            routing::get(get_image_action)
                .route_layer(has_any_permission(vec!["Video:read", "org_admin"])),
        )
}
