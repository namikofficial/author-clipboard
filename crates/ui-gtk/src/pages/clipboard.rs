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
use author_clipboard_shared::picker::{
    self, PickerAction, PickerEntry, PickerFilter, PickerSource,
};

use crate::widgets::empty::{EmptyState, EmptyVariant};
use crate::widgets::filter_bar::{FilterBar, OnChange as OnFilterChange};
use crate::widgets::item_row::ItemRow;
use crate::widgets::search::SearchEntry2;

/// Initial-state props for the clipboard page.
#[derive(Debug, Clone)]
pub struct ClipboardPageProps {
    /// Pre-fill text for the search entry.
    pub initial_query: String,
    /// Initial filter chip.
    pub initial_filter: PickerFilter,
    /// Maximum items to load (clamped to `>= 1`).
    pub count: usize,
    /// Which data source to display (history, snippets, emoji, …).
    pub source: PickerSource,
    /// Include sensitive items in results.
    pub include_sensitive: bool,
    /// Action to perform on Enter (copy or quick-paste).
    pub action: PickerAction,
}

impl Default for ClipboardPageProps {
    fn default() -> Self {
        let shared = author_clipboard_shared::config::PickerConfig::default();
        Self {
            initial_query: String::new(),
            initial_filter: PickerFilter::All,
            count: shared.max_results,
            source: PickerSource::History,
            include_sensitive: false,
            action: PickerAction::Copy,
        }
    }
}

/// Typed payload the page passes to its `on_copy` callback.
#[derive(Debug, Clone)]
pub struct ClipboardCopyRequest {
    /// Database id of the row the user activated.
    pub id: i64,
    /// MIME type of the row (e.g. `"text/plain"`, `"image/png"`).
    pub mime: String,
    /// Copy mode (copy or quick-paste), driven by config/CLI.
    pub mode: CopyMode,
}

/// Reusable row widgets. Each row carries its own stable database ID.
type ItemRowTable = Rc<RefCell<Vec<ItemRow>>>;

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
    app_state: &Rc<RefCell<crate::app::AppState>>,
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
    app_state.borrow_mut().config.action = props.action;
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
        let entries = load_entries_for(&s.query, s.filter, s.count, s.source, s.include_sensitive);
        let items = entries.iter().map(entry_to_item).collect();
        crate::app::reduce(
            &mut app_state_for_refresh.borrow_mut(),
            crate::app::Action::ItemsLoaded(items),
        );
        rebuild_list(
            &list_for_refresh,
            &item_rows_for_refresh,
            &entries,
            s.query.is_empty(),
        );
        // Select the first row after refresh, so GTK ListBox selection
        // is always authoritative and connected to AppState.
        let selected_id = app_state_for_refresh.borrow().selected_id;
        if let Some(id) = selected_id {
            if let Some(row) = item_rows_for_refresh
                .borrow()
                .iter()
                .find(|row| row.id() == id)
            {
                list_for_refresh.select_row(Some(row.row()));
            } else {
                list_for_refresh.unselect_all();
            }
        } else {
            list_for_refresh.unselect_all();
        }
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

    // ── Synchronize AppState from GTK ListBox selection ───────
    // GTK selection is authoritative. When the user navigates with
    // Up/Down/Home/End/PageUp/PageDown (via ListBox), we update
    // AppState to match.
    let app_state_for_sync = app_state.clone();
    list_box.connect_row_selected(move |_list, row| {
        let id = row.and_then(ItemRow::id_from_row);
        app_state_for_sync.borrow_mut().select_by_id(id);
    });

    // ── Wrap on_copy in Rc before wiring handlers ─────────────
    let on_copy = Rc::new(on_copy);

    // ── Copy on Enter: find the selected row's item id and call on_copy.
    let state_for_copy = state.clone();
    let app_state_for_copy = app_state.clone();
    let on_copy_for_activate = on_copy.clone();
    list_box.connect_row_activated(move |_list, row| {
        let Some(id) = ItemRow::id_from_row(row) else {
            return;
        };
        let Some(item) = app_state_for_copy
            .borrow()
            .items
            .iter()
            .find(|item| item.id == id)
            .cloned()
        else {
            return;
        };
        let mode = match state_for_copy.borrow().action {
            PickerAction::Copy => CopyMode::Copy,
            PickerAction::QuickPaste => CopyMode::QuickPaste,
        };
        on_copy_for_activate(ClipboardCopyRequest {
            id,
            mime: item.mime_type.clone(),
            mode,
        });
    });

    // ── Page-level key handlers ───────────────────────────────
    // These keys are handled at the *page* level (not window level)
    // because they interact with the ListBox or need the page's
    // refresh/side-table context.
    let page_keys = gtk4::EventControllerKey::new();

    // Ctrl+Shift+P — toggle pinned filter
    // Ctrl+Shift+A — toggle starred filter
    // Ctrl+Shift+C — collection chooser
    // Ctrl+Enter — alternate activation
    let page_for_page = page.clone();
    let state_for_page = state.clone();
    let app_state_for_page = app_state.clone();
    let refresh_for_page = refresh.clone();
    let on_copy_for_page = on_copy.clone();
    page_keys.connect_key_pressed(move |_controller, key, _code, modifiers| {
        let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
        let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);

        // Ctrl+Shift+P: toggle pinned filter
        if ctrl && shift && key == gdk::Key::p {
            let mut s = state_for_page.borrow_mut();
            s.filter = if s.filter == PickerFilter::Pinned {
                PickerFilter::All
            } else {
                PickerFilter::Pinned
            };
            drop(s);
            refresh_for_page();
            return glib::Propagation::Stop;
        }

        // Ctrl+Shift+A: toggle starred filter
        if ctrl && shift && key == gdk::Key::a {
            let mut s = state_for_page.borrow_mut();
            s.filter = if s.filter == PickerFilter::Starred {
                PickerFilter::All
            } else {
                PickerFilter::Starred
            };
            drop(s);
            refresh_for_page();
            return glib::Propagation::Stop;
        }

        // Ctrl+Shift+C: collection chooser
        if ctrl && shift && key == gdk::Key::c {
            let Some(item_id) = app_state_for_page.borrow().selected_id else {
                return glib::Propagation::Stop;
            };
            show_collection_chooser(&page_for_page, item_id);
            return glib::Propagation::Stop;
        }

        // Ctrl+Enter: alternate activation (quick-paste if copy mode, etc.)
        if ctrl && !shift && key == gdk::Key::Return {
            if let Some(item) = app_state_for_page.borrow().selected_item().cloned() {
                let mode = match state_for_page.borrow().action {
                    PickerAction::Copy => CopyMode::QuickPaste,
                    PickerAction::QuickPaste => CopyMode::Copy,
                };
                on_copy_for_page(ClipboardCopyRequest {
                    id: item.id,
                    mime: item.mime_type,
                    mode,
                });
            }
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });
    page.add_controller(page_keys);

    // Initial load.
    let entries = load_entries_for(
        &props.initial_query,
        props.initial_filter,
        props.count.max(1),
        props.source,
        props.include_sensitive,
    );
    crate::app::reduce(
        &mut app_state.borrow_mut(),
        crate::app::Action::ItemsLoaded(entries.iter().map(entry_to_item).collect()),
    );
    rebuild_list(
        &list_box,
        &item_rows,
        &entries,
        props.initial_query.is_empty(),
    );
    // Select the first row if items exist, so GTK selection is authoritative.
    if !entries.is_empty() {
        if let Some(row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&row));
        }
    }
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
    show_collection_chooser_for_window(&window, item_id);
}

/// Open the collection chooser for a stable selected item ID.
#[allow(deprecated)]
pub fn show_collection_chooser_for_window(window: &impl IsA<gtk4::Window>, item_id: i64) {
    let dialog = gtk4::Dialog::builder()
        .title("Add to collection")
        .transient_for(window)
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
    source: PickerSource,
    include_sensitive: bool,
    action: PickerAction,
}

impl PageState {
    fn from_props(props: &ClipboardPageProps) -> Self {
        Self {
            query: props.initial_query.clone(),
            filter: props.initial_filter,
            count: props.count.max(1),
            source: props.source,
            include_sensitive: props.include_sensitive,
            action: props.action,
        }
    }
}

/// Load entries from the database using the shared picker logic.
///
/// The `query` is the current search text; `filter` is the active
/// filter chip; `count` is the max number of items to return.
/// `source` controls which data source (history, snippets, emoji, …)
/// is displayed.
fn load_entries_for(
    query: &str,
    filter: PickerFilter,
    count: usize,
    source: PickerSource,
    include_sensitive: bool,
) -> Vec<PickerEntry> {
    match source {
        PickerSource::Emoji => load_emoji_entries(query, filter),
        PickerSource::Symbols => load_symbol_entries(query, filter),
        PickerSource::Kaomoji => load_kaomoji_entries(query, filter),
        PickerSource::Snippets => load_snippet_entries(query, filter, count),
        PickerSource::All => load_all_entries(query, filter, count, include_sensitive),
        PickerSource::History => load_history_entries(query, filter, count, include_sensitive),
    }
}

fn load_emoji_entries(query: &str, filter: PickerFilter) -> Vec<PickerEntry> {
    picker::filter_and_query(&picker::emoji_entries(query), query, filter)
}

fn load_symbol_entries(query: &str, filter: PickerFilter) -> Vec<PickerEntry> {
    picker::filter_and_query(&picker::symbol_entries(query), query, filter)
}

fn load_kaomoji_entries(query: &str, filter: PickerFilter) -> Vec<PickerEntry> {
    picker::filter_and_query(&picker::kaomoji_entries(query), query, filter)
}

fn load_snippet_entries(query: &str, filter: PickerFilter, count: usize) -> Vec<PickerEntry> {
    let response = IpcClient::new().send_command(&IpcCommand::ListSnippets);
    let entries: Vec<PickerEntry> = match response {
        Ok(resp) if resp.ok => resp
            .data
            .and_then(|d| d.get("snippets").cloned())
            .and_then(|s| s.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(|v| {
                let id = v.get("id")?.as_i64()?;
                let name = v.get("name")?.as_str()?.to_string();
                let content = v.get("content")?.as_str()?.to_string();
                Some(PickerEntry {
                    id: Some(id),
                    source: PickerSource::Snippets,
                    content_type: Some(author_clipboard_shared::types::ContentType::Text),
                    title: name,
                    subtitle: Some("snippet".to_string()),
                    content,
                    mime_type: Some("text/plain".to_string()),
                    sensitive: false,
                    pinned: false,
                    starred: false,
                    timestamp: None,
                })
            })
            .take(count)
            .collect(),
        _ => Vec::new(),
    };
    if query.is_empty() && filter == PickerFilter::All {
        return entries;
    }
    picker::filter_and_query(&entries, query, filter)
}

fn load_all_entries(
    query: &str,
    filter: PickerFilter,
    count: usize,
    include_sensitive: bool,
) -> Vec<PickerEntry> {
    let mut all = load_history_entries(query, filter, count, include_sensitive);
    if all.len() < count {
        let remaining = count.saturating_sub(all.len());
        let snippets = load_snippet_entries(query, filter, remaining);
        all.extend(snippets);
    }
    all
}

fn load_history_entries(
    query: &str,
    filter: PickerFilter,
    count: usize,
    include_sensitive: bool,
) -> Vec<PickerEntry> {
    let filter_opts = if include_sensitive {
        None
    } else {
        Some(author_clipboard_shared::ipc::FilterOptions {
            sensitive: Some(false),
            ..Default::default()
        })
    };

    let command = if query.is_empty() {
        IpcCommand::History {
            limit: count,
            offset: None,
            filters: filter_opts,
        }
    } else {
        IpcCommand::Search {
            query: query.to_string(),
            limit: Some(count),
            filters: filter_opts,
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
fn rebuild_list(list: &ListBox, item_rows: &ItemRowTable, entries: &[PickerEntry], grouped: bool) {
    let mut old_by_id: std::collections::HashMap<i64, ItemRow> = item_rows
        .borrow_mut()
        .drain(..)
        .map(|row| (row.id(), row))
        .collect();
    let new_ids: std::collections::HashSet<i64> =
        entries.iter().filter_map(|entry| entry.id).collect();
    for (id, row) in &old_by_id {
        if !new_ids.contains(id) {
            list.remove(row.row());
        }
    }
    let mut new_rows = Vec::with_capacity(entries.len());
    let mut previous_group = None;
    for (index, entry) in entries.iter().enumerate() {
        // Render the entry as a `ClipboardItem` so we can reuse
        // `ItemRow::new`. We map the PickerEntry fields to the
        // `ClipboardItem` shape; sensitive + pinned are preserved.
        let item = entry_to_item(entry);
        let id = entry.id.unwrap_or(0);
        let row = old_by_id.remove(&id).map_or_else(
            || ItemRow::new(&item),
            |mut row| {
                row.bind(&item);
                row
            },
        );
        let group = result_group(&item);
        if grouped && previous_group != Some(group) {
            let header = gtk4::Label::new(Some(group));
            header.add_css_class("result-group-header");
            header.set_halign(gtk4::Align::Start);
            row.row().set_header(Some(&header));
        } else {
            row.row().set_header(None::<&gtk4::Widget>);
        }
        previous_group = Some(group);
        let wanted = i32::try_from(index).unwrap_or(i32::MAX);
        let current = row.row().index();
        if row.row().parent().is_none() {
            list.insert(row.row(), wanted);
        } else if current != wanted {
            list.remove(row.row());
            list.insert(row.row(), wanted);
        }
        new_rows.push(row);
    }
    *item_rows.borrow_mut() = new_rows;
}

fn result_group(item: &author_clipboard_shared::types::ClipboardItem) -> &'static str {
    use author_clipboard_shared::presentation::ContentPresentation;
    if item.pinned {
        return "Pinned";
    }
    match author_clipboard_shared::presentation::present(item) {
        ContentPresentation::Url { .. } => "Links",
        ContentPresentation::Code { .. } | ContentPresentation::Json { .. } => "Code & data",
        ContentPresentation::Image { .. } => "Images",
        ContentPresentation::File { .. } => "Files",
        ContentPresentation::Secret { .. } => "Protected",
        _ => "Recent",
    }
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
/// The `mode` parameter controls whether the item is copied or
/// quick-pasted. The window layer (popup) calls this and then
/// closes; the manager layer calls this and shows a toast.
pub fn copy_via_ipc(id: i64, mime: &str, mode: CopyMode) -> Result<String, String> {
    let client = IpcClient::new();
    match client.send_command(&IpcCommand::Copy {
        id,
        mode,
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

    // ── ClipboardPageProps ──────────────────────────────────────────

    #[test]
    fn clipboard_page_props_default_source_is_history() {
        let props = ClipboardPageProps::default();
        assert_eq!(props.source, PickerSource::History);
    }

    #[test]
    fn clipboard_page_props_default_action_is_copy() {
        let props = ClipboardPageProps::default();
        assert_eq!(props.action, PickerAction::Copy);
    }

    #[test]
    fn clipboard_page_props_default_include_sensitive_is_false() {
        let props = ClipboardPageProps::default();
        assert!(!props.include_sensitive);
    }

    #[test]
    fn clipboard_page_props_default_count_matches_config() {
        let props = ClipboardPageProps::default();
        let config = author_clipboard_shared::config::PickerConfig::default();
        assert_eq!(props.count, config.max_results);
    }

    #[test]
    fn clipboard_page_props_default_filter_is_all() {
        let props = ClipboardPageProps::default();
        assert_eq!(props.initial_filter, PickerFilter::All);
    }

    #[test]
    fn clipboard_page_props_can_set_source_to_snippets() {
        let props = ClipboardPageProps {
            source: PickerSource::Snippets,
            ..Default::default()
        };
        assert_eq!(props.source, PickerSource::Snippets);
    }

    #[test]
    fn clipboard_page_props_can_set_source_to_emoji() {
        let props = ClipboardPageProps {
            source: PickerSource::Emoji,
            ..Default::default()
        };
        assert_eq!(props.source, PickerSource::Emoji);
    }

    #[test]
    fn clipboard_page_props_can_set_action_to_quick_paste() {
        let props = ClipboardPageProps {
            action: PickerAction::QuickPaste,
            ..Default::default()
        };
        assert_eq!(props.action, PickerAction::QuickPaste);
    }

    #[test]
    fn clipboard_page_props_can_set_include_sensitive() {
        let props = ClipboardPageProps {
            include_sensitive: true,
            ..Default::default()
        };
        assert!(props.include_sensitive);
    }

    #[test]
    fn clipboard_page_props_can_set_initial_query() {
        let props = ClipboardPageProps {
            initial_query: "test search".to_string(),
            ..Default::default()
        };
        assert_eq!(props.initial_query, "test search");
    }

    #[test]
    fn clipboard_page_props_can_set_initial_filter_pinned() {
        let props = ClipboardPageProps {
            initial_filter: PickerFilter::Pinned,
            ..Default::default()
        };
        assert_eq!(props.initial_filter, PickerFilter::Pinned);
    }

    // ── PageState from Props ────────────────────────────────────────

    #[test]
    fn page_state_from_props_inherits_all_fields() {
        let props = ClipboardPageProps {
            initial_query: "find me".to_string(),
            initial_filter: PickerFilter::Pinned,
            count: 25,
            source: PickerSource::Snippets,
            include_sensitive: true,
            action: PickerAction::QuickPaste,
        };
        let state = PageState::from_props(&props);
        assert_eq!(state.query, "find me");
        assert_eq!(state.filter, PickerFilter::Pinned);
        assert_eq!(state.count, 25);
        assert_eq!(state.source, PickerSource::Snippets);
        assert!(state.include_sensitive);
        assert_eq!(state.action, PickerAction::QuickPaste);
    }

    #[test]
    fn page_state_from_props_clamps_count_to_at_least_one() {
        let props = ClipboardPageProps {
            count: 0,
            ..Default::default()
        };
        let state = PageState::from_props(&props);
        assert_eq!(state.count, 1);
    }

    #[test]
    fn page_state_source_default_is_history() {
        let state = PageState::from_props(&ClipboardPageProps::default());
        assert_eq!(state.source, PickerSource::History);
    }

    // ── ClipboardCopyRequest ────────────────────────────────────────

    #[test]
    fn clipboard_copy_request_holds_mode() {
        let req = ClipboardCopyRequest {
            id: 42,
            mime: "text/plain".to_string(),
            mode: CopyMode::QuickPaste,
        };
        assert_eq!(req.id, 42);
        assert_eq!(req.mime, "text/plain");
        assert_eq!(req.mode, CopyMode::QuickPaste);
    }

    #[test]
    fn clipboard_copy_request_copy_mode() {
        let req = ClipboardCopyRequest {
            id: 7,
            mime: "image/png".to_string(),
            mode: CopyMode::Copy,
        };
        assert_eq!(req.mode, CopyMode::Copy);
    }

    // ── entry_to_item ───────────────────────────────────────────────

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

    #[test]
    fn empty_query_groups_use_shared_presentation_and_pin_priority() {
        let link = author_clipboard_shared::types::ClipboardItem::new_text(
            "https://example.com".to_string(),
        );
        assert_eq!(result_group(&link), "Links");
        let mut pinned = link;
        pinned.pinned = true;
        assert_eq!(result_group(&pinned), "Pinned");
        let secret =
            author_clipboard_shared::types::ClipboardItem::new_text("password=hunter2".to_string());
        assert_eq!(result_group(&secret), "Protected");
    }
}
