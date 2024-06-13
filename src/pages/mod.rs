use std::fmt::Write;

use axum::response::Redirect;
#[allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_extra::extract::{Form, Query};
use axum_htmx::HxTrigger;
use error_stack::Report;
use filigree::html::Svg;
use maud::{html, Markup, Render};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    models::video::{self, VideoId, VideoListResult, VideoProcessingState},
    pages::{auth::WebAuthed, error::HtmlError},
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

async fn add_video_action(
    State(state): State<ServerState>,
    auth: Authed,
    form: Form<AddVideoActionPayload>,
) -> Result<impl IntoResponse, Error> {
    let id = crate::models::video::create_via_url(&state, &auth, &form.url).await?;

    // Hack until filigree supports better model fetching and conversion between types
    let mut videos = crate::models::video::queries::list(
        &state.db,
        &auth,
        &video::queries::ListQueryFilters {
            id: vec![id],
            ..Default::default()
        },
    )
    .await?;
    let video = videos.pop().ok_or(crate::Error::NotFound("new video"))?;

    let body = video_row_fragment(&video, false);
    Ok(body)
}

async fn video_status_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<VideoId>,
) -> Result<impl IntoResponse, Error> {
    let mut videos = crate::models::video::queries::list(
        &state.db,
        &auth,
        &video::queries::ListQueryFilters {
            id: vec![id],
            ..Default::default()
        },
    )
    .await?;
    let video = videos.pop().ok_or(crate::Error::NotFound("new video"))?;
    let body = video_row_fragment(&video, false);

    Ok(body)
}

async fn delete_video_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<VideoId>,
) -> Result<impl IntoResponse, Error> {
    video::queries::delete(&state.db, &auth, &id).await?;
    Ok("")
}

async fn rerun_stage_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path((id, stage)): Path<(crate::models::video::VideoId, String)>,
) -> Result<impl IntoResponse, Error> {
    crate::models::video::rerun_stage(&state, &auth, id, &stage).await?;
    Ok(Redirect::to(&format!("/_action/videos/{id}")))
}

#[derive(serde::Deserialize, serde::Serialize, Debug, JsonSchema)]
pub struct MarkReadActionPayload {
    pub read: bool,
    pub unread_only: bool,
}

pub fn mark_read_icon(read: bool) -> Svg<'static, 'static> {
    Svg::new(if read {
        md_icons::outlined::ICON_EMAIL
    } else {
        md_icons::outlined::ICON_DRAFTS
    })
}

fn mark_read_action_fragment(id: VideoId, read: bool, unread_only: bool) -> Markup {
    let next_read = !read;
    let (target, swap) = if unread_only && next_read {
        ("closest li", "delete")
    } else {
        ("this", "outerHTML")
    };

    html! {
        button
            .btn.btn-circle.btn-outline
            aria-label={"Mark" (if next_read { "Read" } else { "Unread" }) }
            hx-post={"_action/mark_read/" (id)}
            hx-swap=(swap)
            hx-target=(target)
            hx-vals={
                r#"{"read":"# (next_read)
                r#","unread_only":"# (unread_only)
                r#"}"# }
            type="button"
        {
            (mark_read_icon(next_read))
        }
    }
}

async fn mark_read_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<crate::models::video::VideoId>,
    form: Form<MarkReadActionPayload>,
) -> Result<impl IntoResponse, Error> {
    video::queries::mark_read(&state.db, &auth, id, form.read).await?;

    let body = mark_read_action_fragment(id, form.read, form.unread_only);

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

fn reprocess_button(id: VideoId, label: &str, stage: &str) -> Markup {
    html! {
        li {
            button flex.gap-2.justify-start
                type="button"
                hx-post={"/_action/videos/" (id) "/rerun/" (stage)}
                hx-target={"#row-" (id)}
                hx-swap="outerHTML"
                "@click"="open = false"
            {
                (Svg::new(md_icons::outlined::ICON_REFRESH))
                (label)
            }
        }
    }
}

fn video_row_fragment(video: &VideoListResult, unread_only: bool) -> Markup {
    let ready = video.processing_state == VideoProcessingState::Ready;
    let read = video.read;

    let trigger = if ready { "none" } else { "load delay:5s" };

    html! {
        li id={"row-" (video.id)}
            .flex.justify-between
            hx-get={"_action/videos/" (video.id)}
            hx-trigger=(trigger)
            hx-swap="outerHTML"
        {
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
                    (mark_read_action_fragment(video.id, read, unread_only))

                    div .relative x-data="{ open: false }" {
                        button .btn.btn-circle.btn-outline
                            aria-label="Menu"
                            "@click"="open = !open"
                        {
                            (Svg::new(md_icons::outlined::ICON_SETTINGS))
                        }

                        ul .menu.absolute.right-0.mt-1.bg-base-200.text-base-content.z-50.rounded-lg
                            x-show="open"
                            x-cloak
                            x-transition
                            "@click.outside"="open = false"
                        {
                            p.menu-title { "Reprocess" }
                            (reprocess_button(video.id, "Download", "download"))
                            (reprocess_button(video.id, "Extract", "extract"))
                            (reprocess_button(video.id, "Analyze", "analyze"))
                            (reprocess_button(video.id, "Transcribe", "transcribe"))
                            (reprocess_button(video.id, "Summarize", "summarize"))

                            li {
                                button flex.gap-2.justify-start
                                    type="button"
                                    hx-delete={"/_action/videos/" (video.id)}
                                    hx-confirm={"Are you sure you want to delete '" (video.title.as_deref().unwrap_or_default()) "'?"}
                                    hx-target={"#row-" (video.id)}
                                    hx-swap="delete"
                                    "@click"="open = false"
                                {
                                    (Svg::new(md_icons::outlined::ICON_DELETE))
                                    "Delete"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn video_list(
    state: &ServerState,
    auth: &WebAuthed,
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
        div .flex.justify-end.gap-2 {
            a #unread-only .flex.items-center.gap-2
                href={"?unread_only=" (!unread_only)}
                hx-target="#video-list"
                hx-push-url="true"
                hx-get={"/?unread_only=" (!unread_only)}
            {
                label.label.gap-2 {
                    input
                        #unread-only-switch
                        name="unread-only"
                        type="checkbox"
                        checked[unread_only]
                        class="toggle";
                    span.label-text { "Unread only" }
                }
            }
        }

        ul #videos .flex.flex-col.gap-4 {
            @for video in videos.iter() {
                (video_row_fragment(video, unread_only))
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
    auth: WebAuthed,
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
        form .flex.flex-col.gap-2.rounded-lg.border.border-neutral.p-4
            hx-post="/_action/add_video"
            // This returns a video row so place it at the top of the list
            hx-target="#videos"
            hx-swap="afterbegin"
            // Clear the text field
            "hx-on:htmx:after-on-load"="this.reset()"
        {
            label .label-text.flex.gap-2.flex-1.text-base for="path" { "Add a new video" }
            div .flex.gap-4 {
                input #path .flex-1.input.input-bordered type="text" name="url" autocomplete="off";
                button .btn.btn-outline type="submit" { "Add" }
            }
        }

        section #video-list .flex.flex-col.gap-4 {
            (video_list(&state, &auth, unread_only).await?)
        }
    }
    };

    Ok(root_layout_page(Some(&auth), "Home", body))
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
            "/_action/videos/:id",
            routing::get(video_status_action)
                .route_layer(has_any_permission(vec!["Video:read", "org_admin"])),
        )
        .route(
            "/_action/videos/:id",
            routing::delete(delete_video_action)
                .route_layer(has_any_permission(vec!["Video:write", "org_admin"])),
        )
        .route(
            "/_action/videos/:id/rerun/:stage",
            routing::post(rerun_stage_action)
                .route_layer(has_any_permission(vec!["Video:owner", "org_admin"])),
        )
        .route(
            "/_action/mark_read/:id",
            routing::post(mark_read_action)
                .route_layer(has_any_permission(vec!["Video:write", "org_admin"])),
        )
        .merge(login::create_routes())
        .merge(logout::create_routes())
        .merge(forgot::create_routes())
        .merge(reset::create_routes())
        .merge(docs::create_routes())
}
