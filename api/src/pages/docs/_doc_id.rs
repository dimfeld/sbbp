#![allow(unused_imports)]
use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_extra::extract::{Form, Query};
use filigree::extract::ValidatedForm;
use itertools::Itertools;
use maud::{html, Markup};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    models::video::{Video, VideoId},
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

            println!(
                "{}-{}: {} {} {}",
                start_time, end_time, start_image_idx, end_image_idx, images.interval
            );

            Some(ImageChunk {
                text,
                start_image_idx,
                end_image_idx,
            })
        })
        .collect();

    output
}

fn doc_settings(video_id: VideoId, is_read: bool) -> Markup {
    html! {}
}

async fn docs_page(
    State(state): State<ServerState>,
    auth: Authed,
    Path(doc_id): Path<crate::models::video::VideoId>,
) -> Result<impl IntoResponse, HtmlError> {
    let video = crate::models::video::queries::get(&state.db, &auth, doc_id).await?;
    let aligned = align(&video);
    let images = video.images.unwrap_or_default();
    println!("{:?}", images.removed);
    let removed = images.removed.iter().collect::<HashSet<_>>();

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
                    (doc_settings(video.id, video.read))
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

                (doc_settings(video.id, video.read))
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
            "/docs/:doc_id/_action/image/:image_id",
            routing::get(get_image_action)
                .route_layer(has_any_permission(vec!["Video:read", "org_admin"])),
        )
}
