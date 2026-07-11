//! Real clipboard page wired to the daemon via IPC.
//!
//! Wires together:
//! * `widgets::SearchEntry2` for debounced full-text search
//! * `widgets::FilterBar` for the 7 filter chips
//! * `widgets::ItemRow` for the list of items
//! * `widgets::EmptyState` for the "nothing to show" view
//! * `shared::picker::load_entries` to read from the DB
//! * `IpcClient::send_command(Copy)` to write to clipboard on Enter
//!
//! US-001/US-002 are inherited from the popup/manager window's
//! global Esc + `/` controllers. This page is responsible for
//! data flow only.

use gtk4::prelude::*;
use gtk4::{gdk, glib, Box as GtkBox, ListBox, Orientation, SelectionMode, Widget};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use author_clipboard_shared::ipc::{CopyMode, IpcClient, IpcCommand};
use author_clipboard_shared::picker::{self, PickerEntry, PickerFilter, PickerSource};

use crate::widgets::empty::{EmptyState, EmptyVariant};
use crate::widgets::filter_bar::{FilterBar, OnChange as OnFilterChange};
use crate::widgets::item_row::ItemRow;
use crate::widgets::search::SearchEntry2;

/// Initial-state props for the clipboard page. The page does not know
/// about `PopupConfig` — the window layer translates the subset the
/// page needs.
#[derive(Debug, Clone)]
pub struct ClipboardPageProps {
    /// Pre-fill text for the search entry.
    pub initial_query: String,
    /// Initial filter chip.
    pub initial_filter: PickerFilter,
    /// Maximum items to load (clamped to `>= 1`).
    pub count: usize,
}

impl Default for ClipboardPageProps {
    fn default() -> Self {
        Self {
            initial_query: String::new(),
            initial_filter: PickerFilter::All,
            count: 50,
        }
    }
}

/// Typed payload the page passes to its `on_copy` callback. The
/// `&str` is the MIME type of the selected row (already resolved).
#[derive(Debug, Clone)]
pub struct ClipboardCopyRequest {
    /// Database id of the row the user activated.
    pub id: i64,
    /// MIME type of the row (e.g. `"text/plain"`, `"image/png"`).
    pub mime: String,
}

/// Side-table mapping row index → `(id, ItemRow, mime)`. Kept outside
/// the GTK list so the row-activated callback can recover the data
/// that was current at build time.
type ItemRowTable = Rc<RefCell<Vec<(i64, ItemRow, String)>>>;

/// Build the clipboard page widget.
///
/// `on_copy` is called when the user confirms a copy (Enter on a
/// selected row). The page is read-only; the window layer decides
/// what to do with the toast / close-the-window.
///
/// Layout is a vertical stack of three CSS-styled sections:
///
/// ```text
///   .popup-section-search   — search entry
///   .popup-section-filter   — filter chip bar
///   .popup-section-list     — scrollable list / empty state
/// ```
#[allow(clippy::too_many_lines)]
pub fn build(
    props: &ClipboardPageProps,
    app_state: Rc<RefCell<crate::app::AppState>>,
    on_copy: impl Fn(ClipboardCopyRequest) + 'static,
) -> impl IsA<Widget> {
    let page = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();

    // ── Section: search ────────────────────────────────────────
    let search_section = GtkBox::builder().orientation(Orientation::Vertical).build();
    search_section.add_css_class("popup-section");
    search_section.add_css_class("popup-section-search");

    // ── Section: filter ────────────────────────────────────────
    let filter_section = GtkBox::builder().orientation(Orientation::Vertical).build();
    filter_section.add_css_class("popup-section");
    filter_section.add_css_class("popup-section-filter");

    // ── Section: list / empty state ────────────────────────────
    let list_section = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .vexpand(true)
        .hexpand(true)
        .build();
    list_section.add_css_class("popup-section");
    list_section.add_css_class("popup-section-list");

    // ── Search + filter bar ─────────────────────────────────────
    let state = Rc::new(RefCell::new(PageState::from_props(props)));
    let list_box = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .show_separators(false)
        .build();
    list_box.add_css_class("clipboard-list");

    // The list rows are `ListBoxRow` widgets. We hold the
    // `ItemRow` structs in a side Vec so the `bind` method can be
    // called when the underlying data changes.
    let item_rows: ItemRowTable = Rc::new(RefCell::new(Vec::new()));

    // The scrollable list and the empty state both live inside
    // `list_section`. Only one is visible at a time; the
    // `refresh` closure toggles between them.
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .child(&list_box)
        .build();
    scrolled.add_css_class("clipboard-scroll");

    let empty_state = EmptyState::new();

    // The list refresh function, captured for the search/filter
    // callbacks to call.
    let list_for_refresh = list_box.clone();
    let state_for_refresh = state.clone();
    let item_rows_for_refresh = item_rows.clone();
    let app_state_for_refresh = app_state.clone();
    let scrolled_for_refresh = scrolled.clone();
    let empty_for_refresh = empty_state.widget().clone();
    let refresh = move || {
        let s = state_for_refresh.borrow();
        let entries = load_entries_for(&s.query, s.filter, s.count);
        let items = entries.iter().map(entry_to_item).collect();
        crate::app::reduce(
            &mut app_state_for_refresh.borrow_mut(),
            crate::app::Action::ItemsLoaded(items),
        );
        rebuild_list(&list_for_refresh, &item_rows_for_refresh, &entries);
        if entries.is_empty() {
            scrolled_for_refresh.set_visible(false);
            empty_for_refresh.set_visible(true);
        } else {
            scrolled_for_refresh.set_visible(true);
            empty_for_refresh.set_visible(false);
        }
    };

    // Search entry.
    let refresh_for_search = refresh.clone();
    let state_for_search = state.clone();
    let on_query: crate::widgets::search::OnQuery = Rc::new(move |q: String| {
        state_for_search.borrow_mut().query = q;
        refresh_for_search();
    });
    let search = SearchEntry2::new("Search clipboard history…", &props.initial_query, on_query);
    search_section.append(search.widget());
    page.append(&search_section);

    // Filter bar.
    let refresh_for_filter = refresh.clone();
    let state_for_filter = state.clone();
    let on_filter: OnFilterChange = Rc::new(move |f: PickerFilter| {
        state_for_filter.borrow_mut().filter = f;
        refresh_for_filter();
    });
    let bar = FilterBar::new(props.initial_filter, on_filter);
    filter_section.append(bar.widget());
    page.append(&filter_section);

    // The scroller + empty state both live in `list_section`.
    // The scroller is visible by default; the empty state is
    // hidden until the initial load or a refresh decides
    // otherwise.
    list_section.append(&scrolled);
    list_section.append(empty_state.widget());
    empty_state.widget().set_visible(false);
    page.append(&list_section);

    // Copy on Enter: find the selected row's item id and call on_copy.
    let item_rows_for_copy = item_rows.clone();
    let app_state_for_copy = app_state.clone();
    let on_copy = Rc::new(on_copy);
    list_box.connect_row_activated(move |_list, row| {
        let index = usize::try_from(row.index()).ok();
        if let Some(idx) = index {
            if let Some((id, _row, mime)) = item_rows_for_copy.borrow().get(idx) {
                crate::app::reduce(
                    &mut app_state_for_copy.borrow_mut(),
                    crate::app::Action::Select(Some(*id)),
                );
                on_copy(ClipboardCopyRequest {
                    id: *id,
                    mime: mime.clone(),
                });
            }
        }
    });

    // Ctrl+Shift+C opens the collection chooser for the selected history row.
    let collection_keys = gtk4::EventControllerKey::new();
    let list_for_collection = list_box.clone();
    let rows_for_collection = item_rows.clone();
    let page_for_collection = page.clone();
    collection_keys.connect_key_pressed(move |_controller, key, _code, modifiers| {
        let requested = key == gdk::Key::c
            && modifiers.contains(gdk::ModifierType::CONTROL_MASK)
            && modifiers.contains(gdk::ModifierType::SHIFT_MASK);
        if !requested {
            return glib::Propagation::Proceed;
        }
        let Some(index) = list_for_collection
            .selected_row()
            .and_then(|row| usize::try_from(row.index()).ok())
        else {
            return glib::Propagation::Stop;
        };
        if let Some((item_id, _, _)) = rows_for_collection.borrow().get(index) {
            show_collection_chooser(&page_for_collection, *item_id);
        }
        glib::Propagation::Stop
    });
    page.add_controller(collection_keys);

    // Initial load.
    let entries = load_entries_for(
        &props.initial_query,
        props.initial_filter,
        props.count.max(1),
    );
    crate::app::reduce(
        &mut app_state.borrow_mut(),
        crate::app::Action::ItemsLoaded(entries.iter().map(entry_to_item).collect()),
    );
    rebuild_list(&list_box, &item_rows, &entries);
    // Set the initial empty-state variant + visibility. The
    // variant follows the same rule as the refresh closure:
    // "no results" when the user has typed something,
    // "no sensitive" when filtered to sensitive, otherwise
    // "no items".
    if entries.is_empty() {
        // Decide which empty-state copy to show. The query takes
        // precedence ("no results") so a typed search never
        // shows the bland "clipboard is empty" message.
        let variant = if props.initial_query.is_empty() {
            match props.initial_filter {
                PickerFilter::Sensitive => EmptyVariant::NoSensitive,
                _ => EmptyVariant::NoItems,
            }
        } else {
            EmptyVariant::NoResults
        };
        empty_state.set_variant(variant);
        scrolled.set_visible(false);
        empty_state.widget().set_visible(true);
    }

    // The daemon rewrites this monotonic revision file after every capture.
    // A file monitor gives the open UI an explicit edge-triggered refresh;
    // correctness no longer depends on a guessed post-open delay.
    let revision_file = gtk4::gio::File::for_path(
        author_clipboard_shared::config::Config::load()
            .data_dir
            .join(".history_revision"),
    );
    if let Ok(monitor) = revision_file.monitor_file(
        gtk4::gio::FileMonitorFlags::NONE,
        None::<&gtk4::gio::Cancellable>,
    ) {
        let refresh_for_signal = refresh.clone();
        monitor.connect_changed(move |_, _, _, _| refresh_for_signal());
        // Retain the monitor for exactly the page lifetime.
        page.connect_destroy(move |_| drop(monitor.clone()));
    }

    page
}

// GtkDialog remains the compatibility path for distributions shipping GTK
// 4.8/4.10; the replacement AlertDialog is not available across our floor.
#[allow(deprecated)]
fn show_collection_chooser(parent: &GtkBox, item_id: i64) {
    let Some(window) = parent
        .root()
        .and_then(|root| root.downcast::<gtk4::Window>().ok())
    else {
        return;
    };
    let dialog = gtk4::Dialog::builder()
        .title("Add to collection")
        .transient_for(&window)
        .modal(true)
        .default_width(360)
        .build();
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
    let list = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .build();
    list.add_css_class("boxed-list");
    let collections: Vec<(String, String)> = IpcClient::new()
        .send_command(&IpcCommand::ListCollections)
        .ok()
        .and_then(|response| response.data)
        .and_then(|data| data.get("collections").cloned())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|value| {
            Some((
                value.get("id")?.as_str()?.to_string(),
                value.get("name")?.as_str()?.to_string(),
            ))
        })
        .collect();
    if collections.is_empty() {
        let empty = gtk4::Label::new(Some("Create a collection in the Collections page first."));
        empty.set_margin_top(18);
        empty.set_margin_bottom(18);
        dialog.content_area().append(&empty);
    } else {
        for (_, collection_name) in &collections {
            let label = gtk4::Label::new(Some(collection_name));
            label.set_halign(gtk4::Align::Start);
            label.set_margin_top(10);
            label.set_margin_bottom(10);
            label.set_margin_start(10);
            label.set_margin_end(10);
            list.append(&label);
        }
        let dialog_for_row = dialog.clone();
        list.connect_row_activated(move |_list, row| {
            let Some(index) = usize::try_from(row.index()).ok() else {
                return;
            };
            let Some(collection) = collections.get(index) else {
                return;
            };
            let response = IpcClient::new().send_command(&IpcCommand::AddToCollection {
                collection_id: collection.0.clone(),
                item_id,
            });
            if response.is_ok_and(|response| response.ok) {
                dialog_for_row.close();
            }
        });
        dialog.content_area().append(&list);
    }
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}

#[derive(Debug)]
struct PageState {
    query: String,
    filter: PickerFilter,
    count: usize,
}

impl PageState {
    fn from_props(props: &ClipboardPageProps) -> Self {
        Self {
            query: props.initial_query.clone(),
            filter: props.initial_filter,
            count: props.count.max(1),
        }
    }
}

/// Load entries from the database using the shared picker logic.
///
/// The `query` is the current search text; `filter` is the active
/// filter chip; `count` is the max number of items to return.
fn load_entries_for(query: &str, filter: PickerFilter, count: usize) -> Vec<PickerEntry> {
    let command = if query.is_empty() {
        IpcCommand::History {
            limit: count,
            offset: None,
            filters: None,
        }
    } else {
        IpcCommand::Search {
            query: query.to_string(),
            limit: Some(count),
            filters: None,
        }
    };
    let response = match IpcClient::new().send_command(&command) {
        Ok(response) if response.ok => response,
        Ok(response) => {
            tracing::warn!(error = ?response.error, "daemon rejected item snapshot");
            return Vec::new();
        }
        Err(error) => {
            tracing::warn!(%error, "failed to load item snapshot through IPC");
            return Vec::new();
        }
    };
    let entries: Vec<PickerEntry> = response
        .data
        .and_then(|data| data.get("items").cloned())
        .and_then(|items| items.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(ipc_item_to_entry)
        .collect();
    // Apply filter + query in one pass via filter_and_query so we don't
    // double-filter (filter_and_query applies filter first, then query).
    picker::filter_and_query(&entries, query, filter)
}

fn ipc_item_to_entry(value: &serde_json::Value) -> Option<PickerEntry> {
    use author_clipboard_shared::types::ContentType;
    let content_type = value
        .get("content_type")?
        .as_str()?
        .parse::<ContentType>()
        .ok()?;
    let content = value.get("content")?.as_str()?.to_string();
    let plain = value
        .get("plain_text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Some(PickerEntry {
        id: Some(value.get("id")?.as_i64()?),
        source: PickerSource::History,
        content_type: Some(content_type),
        title: if plain.is_empty() {
            content.clone()
        } else {
            plain.to_string()
        },
        subtitle: value
            .get("preview")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        content,
        mime_type: value
            .get("mime_type")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        sensitive: value
            .get("sensitive")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        pinned: value
            .get("pinned")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        starred: value
            .get("starred")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        timestamp: value
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok()),
    })
}

/// Rebuild the list with the given entries. Reuses existing
/// `ItemRow` widgets where possible to minimize churn.
fn rebuild_list(list: &ListBox, item_rows: &ItemRowTable, entries: &[PickerEntry]) {
    let mut old_by_id: std::collections::HashMap<i64, (ItemRow, String)> = item_rows
        .borrow_mut()
        .drain(..)
        .map(|(id, row, mime)| (id, (row, mime)))
        .collect();
    let new_ids: std::collections::HashSet<i64> =
        entries.iter().filter_map(|entry| entry.id).collect();
    for (id, (row, _)) in &old_by_id {
        if !new_ids.contains(id) {
            list.remove(row.row());
        }
    }
    let mut new_rows = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        // Render the entry as a `ClipboardItem` so we can reuse
        // `ItemRow::new`. We map the PickerEntry fields to the
        // `ClipboardItem` shape; sensitive + pinned are preserved.
        let item = entry_to_item(entry);
        let id = entry.id.unwrap_or(0);
        let mime = entry_mime(entry);
        let row = old_by_id.remove(&id).map_or_else(
            || ItemRow::new(&item),
            |(mut row, _)| {
                row.bind(&item);
                row
            },
        );
        let wanted = i32::try_from(index).unwrap_or(i32::MAX);
        let current = row.row().index();
        if row.row().parent().is_none() {
            list.insert(row.row(), wanted);
        } else if current != wanted {
            list.remove(row.row());
            list.insert(row.row(), wanted);
        }
        new_rows.push((id, row, mime));
    }
    *item_rows.borrow_mut() = new_rows;
}

/// Map a [`PickerEntry`] to a minimal [`ClipboardItem`] for the row.
fn entry_to_item(entry: &PickerEntry) -> author_clipboard_shared::types::ClipboardItem {
    use author_clipboard_shared::types::{ClipboardItem, ContentType};
    let mime = entry_mime(entry);
    let mut item = match entry.content_type {
        Some(ContentType::Image) => {
            let mut item = ClipboardItem::new_text(entry.content.clone());
            item.mime_type = mime;
            item.content_type = ContentType::Image;
            item
        }
        Some(ContentType::Html) => {
            ClipboardItem::new_html(entry.content.clone(), entry.title.clone())
        }
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

/// Returns the MIME type for a picker entry, defaulting to text/plain.
fn entry_mime(entry: &PickerEntry) -> String {
    entry
        .mime_type
        .clone()
        .unwrap_or_else(|| "text/plain".to_string())
}

/// Copy an item to the Wayland clipboard via IPC. Returns
/// `Ok(mime)` on success, `Err(String)` on failure.
///
/// This is the bridge the page uses when the user confirms a copy.
/// The window layer (popup) calls this and then closes; the
/// manager layer calls this and shows a toast.
pub fn copy_via_ipc(id: i64, mime: &str) -> Result<String, String> {
    let client = IpcClient::new();
    match client.send_command(&IpcCommand::Copy {
        id,
        mode: CopyMode::Copy,
        mime: Some(mime.to_string()),
    }) {
        Ok(resp) if resp.ok => Ok(mime.to_string()),
        Ok(resp) => Err(resp
            .error
            .map_or_else(|| "copy failed".to_string(), |e| e.message)),
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

    #[test]
    fn entry_to_item_preserves_html_and_plain_text_fallback() {
        let entry = PickerEntry {
            id: Some(8),
            source: PickerSource::History,
            content_type: Some(author_clipboard_shared::types::ContentType::Html),
            title: "Formatted heading".to_string(),
            subtitle: None,
            content: "<h1>Formatted heading</h1>".to_string(),
            mime_type: Some("text/html".to_string()),
            sensitive: false,
            pinned: false,
            starred: false,
            timestamp: Some(chrono::Utc::now()),
        };
        let item = entry_to_item(&entry);
        assert_eq!(item.content, "<h1>Formatted heading</h1>");
        assert_eq!(item.mime_type, "text/html");
        assert_eq!(item.plain_text.as_deref(), Some("Formatted heading"));
    }

    #[test]
    fn ipc_snapshot_item_maps_to_picker_entry() {
        let value = serde_json::json!({
            "id": 91,
            "content": "hello",
            "plain_text": "hello",
            "preview": "hello",
            "mime_type": "text/plain",
            "content_type": "text",
            "timestamp": "2026-07-12T00:00:00Z",
            "pinned": true,
            "starred": true,
            "sensitive": false
        });
        let entry = ipc_item_to_entry(&value).expect("valid IPC item");
        assert_eq!(entry.id, Some(91));
        assert_eq!(entry.title, "hello");
        assert!(entry.pinned);
        assert!(entry.starred);
    }
}
