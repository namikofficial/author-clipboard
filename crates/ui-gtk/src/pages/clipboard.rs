//! Real clipboard page wired to the daemon via IPC.
//!
//! Wires together:
//! * `widgets::SearchEntry2` for debounced full-text search
//! * `widgets::FilterBar` for the 7 filter chips
//! * `widgets::ItemRow` for the list of items
//! * `shared::picker::load_entries` to read from the DB
//! * `IpcClient::send_command(Copy)` to write to clipboard on Enter
//!
//! US-001/US-002 are inherited from the popup/manager window's
//! global Esc + `/` controllers. This page is responsible for
//! data flow only.

use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, ListBox, Orientation, SelectionMode, Widget};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use author_clipboard_shared::config::Config;
use author_clipboard_shared::ipc::{CopyMode, IpcClient, IpcCommand};
use author_clipboard_shared::picker::{
    self, PickerEntry, PickerFilter, PickerOptions, PickerSource,
};
use author_clipboard_shared::Database;

use crate::widgets::filter_bar::{FilterBar, OnChange as OnFilterChange};
use crate::widgets::item_row::ItemRow;
use crate::widgets::search::SearchEntry2;

/// Build the clipboard page widget.
///
/// `on_copy` is called when the user confirms a copy (Enter on a
/// selected row). The page is read-only; the window layer decides
/// what to do with the toast / close-the-window.
pub fn build(config: &Config, on_copy: impl Fn(i64, &str) + 'static) -> impl IsA<Widget> {
    let page = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // ── Search + filter bar ─────────────────────────────────────
    let state = Rc::new(RefCell::new(PageState::default()));
    let list_box = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .show_separators(false)
        .build();
    list_box.add_css_class("clipboard-list");

    // The list rows are `ListBoxRow` widgets. We hold the
    // `ItemRow` structs in a side Vec so the `bind` method can be
    // called when the underlying data changes.
    let item_rows: Rc<RefCell<Vec<(i64, ItemRow)>>> = Rc::new(RefCell::new(Vec::new()));

    // The list refresh function, captured for the search/filter
    // callbacks to call.
    let list_for_refresh = list_box.clone();
    let state_for_refresh = state.clone();
    let config_clone = config.clone();
    let item_rows_for_refresh = item_rows.clone();
    let refresh = move || {
        let s = state_for_refresh.borrow();
        let entries = load_entries_for(&config_clone, &s.query, s.filter, s.count);
        rebuild_list(&list_for_refresh, &item_rows_for_refresh, &entries);
    };

    // Search entry.
    let refresh_for_search = refresh.clone();
    let state_for_search = state.clone();
    let on_query: crate::widgets::search::OnQuery = Rc::new(move |q: String| {
        state_for_search.borrow_mut().query = q;
        refresh_for_search();
    });
    let search = SearchEntry2::new("Search clipboard history…", "", on_query);
    page.append(search.widget());

    // Filter bar.
    let refresh_for_filter = refresh.clone();
    let state_for_filter = state.clone();
    let on_filter: OnFilterChange = Rc::new(move |f: PickerFilter| {
        state_for_filter.borrow_mut().filter = f;
        refresh_for_filter();
    });
    let bar = FilterBar::new(PickerFilter::All, on_filter);
    page.append(bar.widget());

    // List (scrollable).
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .child(&list_box)
        .build();
    scrolled.add_css_class("clipboard-scroll");
    page.append(&scrolled);

    // Copy on Enter: find the selected row's item id and call on_copy.
    let item_rows_for_copy = item_rows.clone();
    let on_copy = Rc::new(on_copy);
    list_box.connect_row_activated(move |_list, row| {
        let index = row.index();
        if index >= 0 {
            let idx = index as usize;
            if let Some((id, _)) = item_rows_for_copy.borrow().get(idx) {
                on_copy(*id, "text/plain");
            }
        }
    });

    // Initial load.
    let entries = load_entries_for(config, "", PickerFilter::All, 50);
    rebuild_list(&list_box, &item_rows, &entries);

    // Schedule a refresh 200ms after the page is first shown so we
    // pick up any clipboard changes the daemon captured while the
    // window was being built.
    let refresh_for_tick = refresh.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
        refresh_for_tick();
    });

    page
}

#[derive(Debug, Default)]
struct PageState {
    query: String,
    filter: PickerFilter,
    count: usize,
}

/// Load entries from the database using the shared picker logic.
///
/// The `query` is the current search text; `filter` is the active
/// filter chip; `count` is the max number of items to return.
fn load_entries_for(
    config: &Config,
    query: &str,
    filter: PickerFilter,
    count: usize,
) -> Vec<PickerEntry> {
    let db = match Database::open(&config.db_path()) {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!("failed to open database: {e}");
            return Vec::new();
        }
    };
    let opts = PickerOptions {
        source: PickerSource::History,
        limit: count,
        query: if query.is_empty() {
            None
        } else {
            Some(query.to_string())
        },
        include_sensitive: matches!(filter, PickerFilter::Sensitive),
        action: author_clipboard_shared::picker::PickerAction::Copy,
    };
    let mut entries = match picker::load_entries(&db, config, &opts) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("failed to load entries: {e}");
            return Vec::new();
        }
    };
    // Apply the text query (the DB-layer `Search` does the heavy
    // lifting when `query` is set, but the picker module's
    // `filter_entries` does the in-memory pass when we already
    // loaded history).
    if !query.is_empty() {
        entries = picker::filter_entries(&entries, query);
    }
    picker::apply_filter(&entries, filter)
}

/// Rebuild the list with the given entries. Reuses existing
/// `ItemRow` widgets where possible to minimize churn.
fn rebuild_list(
    list: &ListBox,
    item_rows: &Rc<RefCell<Vec<(i64, ItemRow)>>>,
    entries: &[PickerEntry],
) {
    // Drop all old children.
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let mut new_rows = Vec::with_capacity(entries.len());
    for entry in entries {
        // Render the entry as a `ClipboardItem` so we can reuse
        // `ItemRow::new`. We map the PickerEntry fields to the
        // `ClipboardItem` shape; sensitive + pinned are preserved.
        let item = entry_to_item(entry);
        let row = ItemRow::new(&item);
        let id = entry.id.unwrap_or(0);
        list.append(row.row());
        new_rows.push((id, row));
    }
    *item_rows.borrow_mut() = new_rows;
}

/// Map a [`PickerEntry`] to a minimal [`ClipboardItem`] for the row.
fn entry_to_item(entry: &PickerEntry) -> author_clipboard_shared::types::ClipboardItem {
    use author_clipboard_shared::types::{ClipboardItem, ContentType};
    let mime = entry
        .mime_type
        .clone()
        .unwrap_or_else(|| "text/plain".to_string());
    let mut item = match entry.content_type {
        Some(ContentType::Image) => {
            let mut item = ClipboardItem::new_text(entry.content.clone());
            item.mime_type = mime;
            item.content_type = ContentType::Image;
            item
        }
        Some(ContentType::Html) => ClipboardItem::new_html(entry.content.clone(), mime),
        Some(ContentType::Files) => ClipboardItem::new_files(entry.content.clone()),
        _ => ClipboardItem::new_text(entry.content.clone()),
    };
    item.sensitive = entry.sensitive;
    item.pinned = entry.pinned;
    item.starred = entry.starred;
    if let Some(ts) = entry.timestamp {
        item.timestamp = ts;
    }
    // Provide a redacted preview if the picker module masked it.
    if entry.sensitive && entry.content == "[hidden]" {
        item.redacted_preview = Some(entry.title.clone());
    }
    item
}

/// Copy an item to the Wayland clipboard via IPC. Returns
/// `Ok(())` on success, `Err(String)` on failure.
///
/// This is the bridge the page uses when the user confirms a copy.
/// The window layer (popup) calls this and then closes; the
/// manager layer calls this and shows a toast.
pub fn copy_via_ipc(id: i64, mime: &str) -> Result<String, String> {
    let client = IpcClient::new();
    let mode = if mime.starts_with("image/") {
        CopyMode::CopyPlainText
    } else {
        CopyMode::Copy
    };
    match client.send_command(&IpcCommand::Copy { id, mode }) {
        Ok(resp) if resp.ok => Ok(mime.to_string()),
        Ok(resp) => Err(resp
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "copy failed".to_string())),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_to_item_preserves_sensitive() {
        let entry = PickerEntry {
            id: Some(7),
            source: PickerSource::History,
            content_type: Some(author_clipboard_shared::types::ContentType::Text),
            title: "Sensitive item".to_string(),
            subtitle: None,
            content: "[hidden]".to_string(),
            mime_type: Some("text/plain".to_string()),
            sensitive: true,
            pinned: false,
            starred: false,
            timestamp: Some(chrono::Utc::now()),
        };
        let item = entry_to_item(&entry);
        assert!(item.sensitive);
        assert_eq!(item.redacted_preview.as_deref(), Some("Sensitive item"));
    }

    #[test]
    fn entry_to_item_preserves_pinned_and_starred() {
        let entry = PickerEntry {
            id: Some(1),
            source: PickerSource::History,
            content_type: Some(author_clipboard_shared::types::ContentType::Text),
            title: "pinned+starred".to_string(),
            subtitle: None,
            content: "x".to_string(),
            mime_type: Some("text/plain".to_string()),
            sensitive: false,
            pinned: true,
            starred: true,
            timestamp: Some(chrono::Utc::now()),
        };
        let item = entry_to_item(&entry);
        assert!(item.pinned);
        assert!(item.starred);
    }
}
