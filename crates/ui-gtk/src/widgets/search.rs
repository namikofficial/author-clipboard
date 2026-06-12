//! Search entry with `/` focus, Esc clear, and 150ms debounced
//! change emission.

use gtk4::prelude::*;
use gtk4::{glib, EventControllerKey, PropagationPhase, SearchEntry, Widget};
use std::cell::Cell;
use std::rc::Rc;

/// Debounce duration in milliseconds.
pub const DEBOUNCE_MS: u64 = 150;

/// Callback signature for the debounced search query change.
pub type OnQuery = std::rc::Rc<dyn Fn(String)>;

/// A search entry that fires `on_query` 150ms after the user
/// stops typing.
pub struct SearchEntry2 {
    inner: SearchEntry,
    debounce_source: Rc<Cell<Option<glib::SourceId>>>,
    pending_query: Rc<Cell<String>>,
    on_query: OnQuery,
}

impl SearchEntry2 {
    /// Build a new search entry. The placeholder is the
    /// `placeholder_text` argument.
    pub fn new(placeholder: &str, initial: &str, on_query: OnQuery) -> Self {
        let inner = SearchEntry::new();
        inner.set_placeholder_text(Some(placeholder));
        inner.set_text(initial);
        inner.set_hexpand(true);
        inner.set_halign(gtk4::Align::Fill);
        inner.add_css_class("search-entry");
        inner.set_size_request(200, -1);

        let debounce_source: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
        let pending_query: Rc<Cell<String>> = Rc::new(Cell::new(initial.to_string()));
        let on_query_for_change = on_query.clone();
        let debounce_source_for_change = debounce_source.clone();
        let pending_query_for_change = pending_query.clone();

        inner.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            pending_query_for_change.set(query.clone());

            // Cancel any existing debounce timer.
            if let Some(id) = debounce_source_for_change.take() {
                id.remove();
            }

            // Schedule a new debounce timer.
            let on_query_inner = on_query_for_change.clone();
            let debounce_source_inner = debounce_source_for_change.clone();
            let pending_query_inner = pending_query_for_change.clone();
            let source_id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(DEBOUNCE_MS),
                move || {
                    // Only fire if this is still the most recent query.
                    let current = pending_query_inner.take();
                    on_query_inner(current);
                    debounce_source_inner.set(None);
                },
            );
            debounce_source_for_change.set(Some(source_id));
        });

        // Esc while the search has focus: clear and emit immediately.
        let entry_for_esc = inner.clone();
        let debounce_source_for_esc = debounce_source.clone();
        let on_query_for_esc = on_query.clone();
        let esc = EventControllerKey::new();
        esc.set_propagation_phase(PropagationPhase::Bubble);
        esc.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                if !entry_for_esc.text().is_empty() {
                    entry_for_esc.set_text("");
                    // The text change will fire `search_changed`,
                    // which cancels the debounce and emits "".
                    if let Some(id) = debounce_source_for_esc.take() {
                        id.remove();
                    }
                    on_query_for_esc("".to_string());
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        inner.add_controller(esc);

        Self {
            inner,
            debounce_source,
            pending_query,
            on_query,
        }
    }

    /// Borrow the underlying widget.
    pub fn widget(&self) -> &Widget {
        self.inner.upcast_ref()
    }

    /// Set the search text programmatically (skips the debounce).
    pub fn set_text(&self, text: &str) {
        self.inner.set_text(text);
    }

    /// Current text.
    pub fn text(&self) -> String {
        self.inner.text().to_string()
    }
}

// suppress unused import warning for the adw-prelude shim used
// elsewhere in this module.
