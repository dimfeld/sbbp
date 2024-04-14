use std::fmt::Write;

#[allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_htmx::HxTrigger;
use error_stack::Report;
use filigree::{extract::ValidatedForm, maud::Svg};
use maud::{html, Markup, Render};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    models::video::{self, Video, VideoListResultAndPopulatedListResult, VideoProcessingState},
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

struct VideoDuration(Option<i32>);

impl Render for VideoDuration {
    fn render_to(&self, buffer: &mut String) {
        let d = self.0.unwrap_or(0);
        let hours = d / 3600;
        let minutes = (d / 60) % 60;
        let seconds = d % 60;

        if hours > 0 {
            write!(buffer, "{hours}:").unwrap();
        }
        write!(buffer, "{:02}:{:02}", minutes, seconds).unwrap();
    }
}

fn video_row_fragment(video: &VideoListResultAndPopulatedListResult) -> Markup {
    let ready = video.processing_state == VideoProcessingState::Ready;
    let read = video.read;

    let mark_read_icon = Svg::new(if read {
        md_icons::outlined::ICON_MARK_EMAIL_UNREAD
    } else {
        md_icons::outlined::ICON_MARK_EMAIL_READ
    });

    html! {
        li id={"row-" (video.id)} .flex.justify-between {
            .flex.flex-col {
                @if ready {
                    a.underline href={"docs/" (video.id)} { (video.title.as_deref().unwrap_or_default()) }
                    span { (VideoDuration(video.duration)) }
                } @else {
                    p { (video.title.as_deref().or(video.url.as_deref()).unwrap_or_default()) }
                    p { (video.processing_state) }
                }
            }

            .flex.gap-2.items-center {
                @if ready {
                    button
                        .btn.btn-circle.btn-outline
                        aria-label={"Mark" (if read { "Unread" } else { "Read" }) }
                        hx-post={"_action/videos/" (video.id) "/mark_read"}
                        hx-vals={r#"{"new_read":"# (!read) "}"}
                        type="button"
                    {
                        (mark_read_icon)
                    }

                    /*

                  <Toggle let:on={open} let:toggleOff let:toggle>
                    <Button variant="outline" icon={settingsIcon} on:click={toggle} />
                    <Menu {open} explicitClose on:close={toggleOff} let:close>
                      <p class="px-2 text-sm font-medium pl-6 py-2">Reprocess</p>
                      <ReprocessForm id={item.id} stage="download" label="Download" {close} />
                      <ReprocessForm id={item.id} stage="extract" label="Extract" {close} />
                      <ReprocessForm id={item.id} stage="analyze" label="Analyze" {close} />
                      <ReprocessForm id={item.id} stage="transcribe" label="Transcribe" {close} />
                      <ReprocessForm id={item.id} stage="summarize" label="Summarize" {close} />

                      <MenuItem>
                        <form method="POST" action="?/delete" use:enhance>
                          <input type="hidden" name="id" value={item.id} />
                          <button type="submit" on:click={close} class="flex items-center gap-2">
                            <Icon data={deleteIcon} /> Delete
                          </button>
                        </form>
                      </MenuItem>
                    </Menu>
                  </Toggle>

                */
                }
            }
        }
    }
}

async fn video_list(
    state: &ServerState,
    auth: &Authed,
    unread_only: bool,
) -> Result<Markup, Report<Error>> {
    let videos = crate::models::video::queries::list(
        &state.db,
        auth,
        &video::queries::ListQueryFilters {
            per_page: Some(50),
            order_by: Some("-created_at".to_string()),
            read: unread_only.then_some(false),
            ..Default::default()
        },
    )
    .await?;

    Ok(html! {
        section #video-list .flex.flex-col.gap-4 {
            div .flex.justify-end.gap-2 {
                a #unread-only .flex.items-center.gap-2
                    href={"?unread_only=" (!unread_only)}
                    hx-target="#video-list"
                    hx-get={"/?unread_only=" (!unread_only)}
                {
                    input
                        #unread-only-switch
                        name="unread-only"
                        type="checkbox"
                        checked[unread_only]
                        class="toggle";
                    label for="unread-only" { "Unread only" }
                }
            }

            ul #videos .flex.flex-col.gap-4 {
                @for video in videos.iter() {
                    (video_row_fragment(video))
                }
            }
        }
    })
}

#[derive(serde::Deserialize, serde::Serialize, Debug, JsonSchema)]
pub struct HomeQuery {
    pub unread_only: Option<bool>,
}

async fn home_page(
    State(state): State<ServerState>,
    auth: Authed,
    Query(qs): Query<HomeQuery>,
    HxTrigger(trigger): HxTrigger,
) -> Result<impl IntoResponse, HtmlError> {
    let unread_only = qs.unread_only.unwrap_or(true);

    match trigger.as_deref() {
        Some("unread-only") => {
            return video_list(&state, &auth, unread_only)
                .await
                .map_err(HtmlError::from)
        }
        _ => {}
    }

    let body = html! {
    main .relative.p-4.flex.flex-col.gap-4 {
        form .flex.flex-col.gap-2.rounded-lg.border.border-border.p-4 hx-post="_action/add_video" {
            label .text-red-50.flex.gap-2.flex-1 ."max-w-[100ch]".text-base for="path" { "Add a new video" }
            div .flex.gap-2 {
                input #path .flex-1 type="text" name="url" autocomplete="off";
                button type="submit" { "Add" }
            }
        }

        (video_list(&state, &auth, unread_only).await?)
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
