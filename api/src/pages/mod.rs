use std::fmt::Write;

#[allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_extra::extract::{Form, Query};
use axum_htmx::HxTrigger;
use error_stack::{Report, ResultExt};
use filigree::{extract::ValidatedForm, html::Svg};
use maud::{html, Markup, Render};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    models::video::{
        self, Video, VideoId, VideoListResultAndPopulatedListResult, VideoProcessingState,
    },
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
    form: Form<AddVideoActionPayload>,
) -> Result<impl IntoResponse, Error> {
    let body = add_video_action_fragment();
    // todo
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
    // todo
    Ok(body)
}

#[derive(serde::Deserialize, serde::Serialize, Debug, JsonSchema)]
pub struct MarkReadActionPayload {
    pub read: bool,
    pub unread_only: bool,
}

fn mark_read_action_fragment(id: VideoId, read: bool, unread_only: bool) -> Markup {
    let next_read = !read;
    let mark_read_icon = Svg::new(if next_read {
        md_icons::outlined::ICON_MARK_EMAIL_READ
    } else {
        md_icons::outlined::ICON_MARK_EMAIL_UNREAD
    });

    let (target, swap) = if unread_only && next_read {
        ("closest li", "delete")
    } else {
        ("this", "outerHTML")
    };

    html! {
        button
            .btn.btn-circle.btn-outline
            aria-label={"Mark" (if next_read { "Read" } else { "Unread" }) }
            hx-post={"_action/videos/" (id) "/mark_read"}
            hx-swap={(swap)}
            hx-target={(target)}
            hx-vals={
                r#"{"read":"# (next_read)
                r#","unread_only":"# (unread_only)
                r#"}"# }
            type="button"
        {
            (mark_read_icon)
        }
    }
}

async fn mark_read_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<crate::models::video::VideoId>,
    form: Form<MarkReadActionPayload>,
) -> Result<impl IntoResponse, Error> {
    sqlx::query!(
        "UPDATE videos
        SET read = $3
        WHERE id = $1 AND organization_id = $2",
        id.as_uuid(),
        auth.organization_id.as_uuid(),
        form.read
    )
    .execute(&state.db)
    .await
    .change_context(Error::Db)?;

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

fn video_row_fragment(video: &VideoListResultAndPopulatedListResult, unread_only: bool) -> Markup {
    let ready = video.processing_state == VideoProcessingState::Ready;
    let read = video.read;

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
                    (mark_read_action_fragment(video.id, read, unread_only))

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
        div .flex.justify-end.gap-2 {
            a #unread-only .flex.items-center.gap-2
                href={"?unread_only=" (!unread_only)}
                hx-target="#video-list"
                hx-push-url="true"
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
            "/_action/videos/:id/rerun/:stage",
            routing::post(rerun_stage_action)
                .route_layer(has_any_permission(vec!["Video:owner", "org_admin"])),
        )
        .route(
            "/_action/videos/:id/mark_read",
            routing::post(mark_read_action)
                .route_layer(has_any_permission(vec!["Video:write", "org_admin"])),
        )
        .merge(login::create_routes())
        .merge(logout::create_routes())
        .merge(forgot::create_routes())
        .merge(reset::create_routes())
        .merge(docs::create_routes())
}
