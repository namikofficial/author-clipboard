//! GSettings bindings for filter, sort, and window state.
//!
//! Persists UI state across popup/manager sessions. User preferences
//! (max items, denylist, etc.) still live in the JSON [`Config`].

#![allow(dead_code, unused_imports, clippy::doc_markdown)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{reduce, Action, AppState};
use gio::prelude::*;

/// The `GSettings` schema ID, mirrored in
/// `data/com.namikofficial.author-clipboard.gschema.xml`.
pub const SCHEMA_ID: &str = "com.namikofficial.author-clipboard.state";

/// One-stop accessor for the runtime's `GSettings`.
#[derive(Clone)]
pub struct Settings {
    inner: gio::Settings,
}

impl Settings {
    /// Open the schema. If the schema is not installed on the
    /// system, this returns `None` and the UI falls back to
    /// in-memory defaults.
    pub fn new() -> Option<Self> {
        std::panic::catch_unwind(|| gio::Settings::new(SCHEMA_ID))
            .ok()
            .map(|inner| Self { inner })
    }

    /// Active filter chip.
    pub fn filter(&self) -> crate::PickerFilter {
        let s = self.inner.string("filter");
        s.parse().unwrap_or(crate::PickerFilter::All)
    }

    /// Set the active filter chip.
    pub fn set_filter(&self, value: crate::PickerFilter) {
        if let Err(e) = self.inner.set_string("filter", &value.to_string()) {
            tracing::warn!(?e, "failed to set filter");
        }
    }

    /// Sort order.
    pub fn sort(&self) -> crate::SortOrder {
        let s = self.inner.string("sort");
        s.parse().unwrap_or(crate::SortOrder::NewestFirst)
    }

    /// Set the sort order.
    pub fn set_sort(&self, value: crate::SortOrder) {
        if let Err(e) = self.inner.set_string("sort", &value.to_string()) {
            tracing::warn!(?e, "failed to set sort");
        }
    }

    /// Last visited page in the manager.
    pub fn last_page(&self) -> crate::app::PageId {
        let s = self.inner.string("last-page");
        s.parse().unwrap_or(crate::app::PageId::Clipboard)
    }

    /// Set the last visited page.
    pub fn set_last_page(&self, value: crate::app::PageId) {
        if let Err(e) = self.inner.set_string("last-page", &value.to_string()) {
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

    /// Set popup window size.
    pub fn set_popup_size(&self, w: i32, h: i32) {
        if let Err(e) = self.inner.set_int("popup-width", w) {
            tracing::warn!(?e, "failed to set popup-width");
        }
        if let Err(e) = self.inner.set_int("popup-height", h) {
            tracing::warn!(?e, "failed to set popup-height");
        }
    }

    /// Manager window size.
    pub fn window_size(&self) -> (i32, i32) {
        (
            self.inner.int("window-width"),
            self.inner.int("window-height"),
        )
    }

    /// Set manager window size.
    pub fn set_window_size(&self, w: i32, h: i32) {
        if let Err(e) = self.inner.set_int("window-width", w) {
            tracing::warn!(?e, "failed to set window-width");
        }
        if let Err(e) = self.inner.set_int("window-height", h) {
            tracing::warn!(?e, "failed to set window-height");
        }
    }
}

/// Binding between GSettings and the app state.
///
/// Reads initial state at startup and dispatches `ConfigLoaded`/
/// `ManagerConfigLoaded` actions. On each GSettings 'changed' signal
/// it dispatches the appropriate `Action` so the reducer records the
/// new value. The runtime calls [`Self::persist`] to flush state back
/// to GSettings.
pub struct SettingsBinding {
    settings: Settings,
    state: Rc<RefCell<AppState>>,
}

impl SettingsBinding {
    /// Create a new binding. Returns `None` if the GSettings schema
    /// is not installed — the UI will use in-memory defaults.
    pub fn new(state: Rc<RefCell<AppState>>) -> Option<Self> {
        Some(Self {
            settings: Settings::new()?,
            state,
        })
    }

    /// Read all GSettings keys and dispatch `ConfigLoaded` /
    /// `ManagerConfigLoaded` / `PageChanged` to seed the state.
    pub fn read_to_state(&self) {
        let mut state = self.state.borrow_mut();
        let popup_config = crate::PopupConfig {
            filter: self.settings.filter(),
            ..Default::default()
        };
        reduce(&mut state, Action::ConfigLoaded(popup_config));
        let last_page = self.settings.last_page();
        reduce(&mut state, Action::PageChanged(last_page));
    }

    /// Handle a GSettings 'changed' signal. Called by the runtime's
    /// `changed` callback.
    pub fn on_changed(&self, key: &str) {
        let mut state = self.state.borrow_mut();
        match key {
            "filter" => {
                reduce(&mut state, Action::FilterChanged(self.settings.filter()));
            }
            "last-page" => {
                reduce(&mut state, Action::PageChanged(self.settings.last_page()));
            }
            _ => {}
        }
    }

    /// Persist the current state to GSettings. Called by the runtime
    /// after processing an `Effect::PersistGSettings`.
    pub fn persist(&self, effect: &crate::Effect) {
        if let crate::Effect::PersistGSettings = effect {
            let state = self.state.borrow();
            self.settings.set_filter(state.filter);
            self.settings.set_last_page(state.active_page);
        }
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
