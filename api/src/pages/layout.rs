use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    sync::{Arc, OnceLock, RwLock},
};

use error_stack::{Report, ResultExt};
use filigree::vite_manifest::{watch::ManifestWatcher, Manifest, ManifestError};
use maud::{html, Markup, DOCTYPE};
use sentry::protocol::Map;
use serde::Deserialize;

use crate::{auth::Authed, Error};

pub static MANIFEST: Manifest = Manifest::new();

pub fn init_manifest(
    manifest_path: &Path,
    watch: bool,
) -> Result<Option<ManifestWatcher>, error_stack::Report<ManifestError>> {
    let base_url = "";
    MANIFEST.read_manifest(base_url, manifest_path)?;

    let watcher = if watch {
        Some(filigree::vite_manifest::watch::watch_manifest(
            base_url.to_string(),
            manifest_path.to_path_buf(),
            &MANIFEST,
        ))
    } else {
        None
    };

    Ok(watcher)
}

/// The HTML shell that every page should be wrapped in to enable basic functionality.
pub fn page_wrapper(title: &str, slot: Markup) -> Markup {
    let client_tags = MANIFEST.index();
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                (client_tags)
                title { (title) }
            }
            body { (slot) }
        }
    }
}

/// The root layout of the application
pub fn root_layout(auth: Option<&Authed>, slot: Markup) -> Markup {
    html! {
        (slot)
    }
}

/// The root layout of the application, as a full HTML page
pub fn root_layout_page(auth: Option<&Authed>, title: &str, slot: Markup) -> Markup {
    page_wrapper(title, root_layout(auth, slot))
}
