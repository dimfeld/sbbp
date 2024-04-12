#![allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use filigree::extract::ValidatedForm;
use maud::html;
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    pages::{error::HtmlError, layout::root_layout_page},
    server::ServerState,
    Error,
};

async fn docs_page(
    State(state): State<ServerState>,
    auth: Authed,
    Path(docId): Path<String>,
) -> Result<impl IntoResponse, HtmlError> {
    let body = html! {};

    Ok(root_layout_page(Some(&auth), "title", body))
}

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new().route("/docs/:docId", routing::get(docs_page))
}
