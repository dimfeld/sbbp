#![allow(unused_imports)]
use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_extra::extract::{Form, Query};
use filigree::{extract::ValidatedForm, html::Svg};
use itertools::Itertools;
use maud::{html, Markup, Render};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    models::video::{Video, VideoId},
    pages::{auth::WebAuthed, error::HtmlError, layout::root_layout_page},
    server::ServerState,
    Error,
};

#[derive(serde::Deserialize, serde::Serialize, Debug, JsonSchema)]
pub struct MarkReadActionPayload {
    pub read: bool,
}

fn mark_read_action_fragment(doc_id: VideoId, next_read: bool) -> Markup {
    let label = if next_read {
        "Mark as Read"
    } else {
        "Mark as Unread"
    };

    html! {
    button .btn.btn-circle.btn-outline
        type="button"
        aria-label=(label)
        hx-post={"/docs/" (doc_id) "/_action/mark_read"}
        hx-swap="outerHTML"
        hx-vals=(format_args!(r##"{{"read": {next_read} }}"##))
        {
            (crate::pages::mark_read_icon(next_read))
        }
    }
}

async fn mark_read_action(
    State(state): State<ServerState>,
    auth: Authed,
    Path(doc_id): Path<crate::models::video::VideoId>,
    form: Form<MarkReadActionPayload>,
) -> Result<impl IntoResponse, Error> {
    crate::models::video::queries::mark_read(&state.db, &auth, doc_id, form.read).await?;
    let body = mark_read_action_fragment(doc_id, !form.read);

    Ok(body)
}

struct ImageChunk {
    text: String,
    start_image_idx: u64,
    end_image_idx: u64,
}

fn align(video: &Video) -> Vec<ImageChunk> {
    let Some((images, transcript)) = video.images.as_ref().zip(video.transcript.as_ref()) else {
        return vec![];
    };

    let Some(paragraphs) = transcript["results"]["channels"][0]["alternatives"][0]["paragraphs"]
        ["paragraphs"]
        .as_array()
    else {
        return vec![];
    };

    let interval = images.interval as f64;
    let output = paragraphs
        .into_iter()
        .filter_map(|p| {
            let text = p["sentences"]
                .as_array()?
                .iter()
                .filter_map(|s| s["text"].as_str())
                .join(" ");

            let start_time = p["start"].as_f64().unwrap_or(0.0);
            let end_time = p["end"].as_f64().unwrap_or(0.0);

            let start_image_idx = ((start_time / interval).ceil() as u64)
                .max(1)
                .min(images.max_index as u64);
            let end_image_idx = ((end_time / interval).floor() as u64)
                .max(1)
                .min(images.max_index as u64);

            Some(ImageChunk {
                text,
                start_image_idx,
                end_image_idx,
            })
        })
        .collect();

    output
}

async fn docs_page(
    State(state): State<ServerState>,
    auth: WebAuthed,
    Path(doc_id): Path<crate::models::video::VideoId>,
) -> Result<impl IntoResponse, HtmlError> {
    let video = crate::models::video::queries::get(&state.db, &auth, doc_id).await?;
    let aligned = align(&video);
    let images = video.images.unwrap_or_default();
    let removed = images.removed.iter().collect::<HashSet<_>>();

    let next_read = !video.read;

    let body = html! {
        div .relative.w-full.overflow-y-auto
            x-data=(format_args!(r##"{{ large_image: null, max_index:{max_index}, show_removed: false }}"##, max_index=images.max_index)) {

            nav .sticky.top-0.w-full.bg-neutral.text-neutral-content.p-4 {
                header .flex.gap-4.w-full
                    .items-start.justify-start.flex-col
                    ."md:items-center md:justify-between md:flex-row"
                    {
                    h1 .text-3xl {
                        @if let Some(title) = &video.title { (title) }
                    }

                    div .flex.gap-4 {
                        (mark_read_action_fragment(doc_id, next_read))

                        a .btn.btn-outline href="/" {
                            "Back to List"
                        }
                    }
                }

                label .flex.items-center.gap-2 {
                    input .checkbox type="checkbox" x-model="show_removed";
                    "Show removed images"
                }
            }

            main flex.flex-col.items-center.p-4 {

                @if let Some(summary) = &video.summary {
                    section {
                        p.text-2xl { "Video Summary" }
                        p.whitespace-pre-wrap.font-serif.text-xl.leading-relaxed ."max-w-[90ch]" {
                            (summary)
                        }
                    }
                }

                div class="grid lg:grid-cols-[auto_auto] grid-cols-1 gap-x-4 gap-y-2 mt-8 font-serif text-xl leading-relaxed" {
                    @for chunk in aligned {
                        div ."max-w-[65ch]" { (chunk.text) }
                        div .flex.flex-col.gap-2.max-w-lg {
                            @for idx in chunk.start_image_idx..=chunk.end_image_idx {
                                @let removed = removed.contains(&(idx as u32));
                                button
                                    type="button"
                                    x-cloak[removed] x-show=[removed.then_some("show_removed")]
                                    "@click"={"large_image = " (idx)}
                                {
                                    img .object-cover.aspect-video.border .border-red-500[removed]
                                        width="512"
                                        src=(format_args!("/api/videos/{doc_id}/image/{idx}"))
                                        alt={ "Image " (idx)}
                                        loading="lazy";
                                }
                            }
                        }

                    }

                }

            }

            template x-if="large_image" {
                button
                    type="button"
                    "@keyup.escape.window"="large_image = null"
                    "@keyup.left.window"="large_image = Math.max(large_image - 1, 1)"
                    "@keyup.right.window"="large_image = Math.min(large_image + 1, max_index)"
                    "@click"="large_image = null"
                    class="fixed inset-0 z-50"
                {
                    img
                        ":src"=(format_args!("'/api/videos/{doc_id}/image/' + large_image"))
                        ":alt"="'Image ' + large_image";
                }
            }
        }

    };

    Ok(root_layout_page(
        Some(&auth),
        video.title.as_deref().unwrap_or(""),
        body,
    ))
}

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new()
        .route("/docs/:doc_id", routing::get(docs_page))
        .route(
            "/docs/:doc_id/_action/mark_read",
            routing::post(mark_read_action)
                .route_layer(has_any_permission(vec!["Video:write", "org_admin"])),
        )
}
