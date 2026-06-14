//! GSettings bindings for filter, sort, and window state.
//!
//! Persists UI state across popup/manager sessions. User preferences
//! (max items, denylist, etc.) still live in the JSON [`Config`].

#![allow(dead_code, unused_imports, clippy::doc_markdown)]

use gio::prelude::*;

/// The `GSettings` schema ID, mirrored in
/// `data/com.namikofficial.author-clipboard.gschema.xml`.
pub const SCHEMA_ID: &str = "com.namikofficial.author-clipboard.state";

/// One-stop accessor for the runtime's `GSettings`.
pub struct Settings {
    inner: gio::Settings,
}

impl Settings {
    /// Open the schema. If the schema is not installed on the
    /// system, this returns `None` and the UI falls back to
    /// in-memory defaults.
    pub fn new() -> Option<Self> {
        // `gio::Settings::new` panics in some error paths; guard with
        // a `catch_unwind` so the UI never crashes on a missing
        // schema. In practice the schema is always compiled in
        // via `data/gschemas.compiled`.
        std::panic::catch_unwind(|| gio::Settings::new(SCHEMA_ID))
            .ok()
            .map(|inner| Self { inner })
    }

    /// Active filter chip.
    pub fn filter(&self) -> String {
        self.inner.string("filter").to_string()
    }

    /// Set the active filter chip.
    pub fn set_filter(&self, value: &str) {
        if let Err(e) = self.inner.set_string("filter", value) {
            tracing::warn!(?e, "failed to set filter");
        }
    }

    /// Sort order.
    pub fn sort(&self) -> String {
        self.inner.string("sort").to_string()
    }

    /// Set the sort order.
    pub fn set_sort(&self, value: &str) {
        if let Err(e) = self.inner.set_string("sort", value) {
            tracing::warn!(?e, "failed to set sort");
        }
    }

    /// Last visited page in the manager.
    pub fn last_page(&self) -> String {
        self.inner.string("last-page").to_string()
    }

    /// Set the last visited page.
    pub fn set_last_page(&self, value: &str) {
        if let Err(e) = self.inner.set_string("last-page", value) {
            tracing::warn!(?e, "failed to set last-page");
        }
    }

    /// Popup window size.
    pub fn popup_size(&self) -> (i32, i32) {
        (
            self.inner.int("popup-width"),
            self.inner.int("popup-height"),
        )
    }

    /// Manager window size.
    pub fn window_size(&self) -> (i32, i32) {
        (
            self.inner.int("window-width"),
            self.inner.int("window-height"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_id_is_stable() {
        assert_eq!(SCHEMA_ID, "com.namikofficial.author-clipboard.state");
    }
}
