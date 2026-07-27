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

use author_clipboard_shared::ipc::{CopyMode, IpcCommand};
use author_clipboard_shared::picker::{
    PickerAction, PickerEntry, PickerFilter, PickerSource,
};

use crate::service::{ClipboardService, HistoryRequest};
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
    service: std::sync::Arc<dyn ClipboardService>,
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

    let empty_state_ref: Rc<EmptyState> = Rc::new(EmptyState::new());

    // The list refresh function, captured for the search/filter
    // callbacks to call.
    let list_for_refresh = list_box.clone();
    let state_for_refresh = state.clone();
    let item_rows_for_refresh = item_rows.clone();
    let app_state_for_refresh = app_state.clone();
    let scrolled_for_refresh = scrolled.clone();
    let empty_widget_for_refresh = empty_state_ref.widget().clone();
    let empty_state_for_refresh = empty_state_ref.clone();
    let generation = Rc::new(std::cell::Cell::new(0_u64));
    let refresh = {
        let service = service.clone();
        move || {
            let request = {
                let mut s = state_for_refresh.borrow_mut();
                s.generation = s.generation.saturating_add(1);
                HistoryRequest {
                    query: s.query.clone(),
                    limit: s.count,
                    filter: s.filter,
                    source: s.source,
                    include_sensitive: s.include_sensitive,
                    generation: s.generation,
                }
            };
            generation.set(request.generation);
            let latest = generation.clone();
            let service = service.clone();
            let state = state_for_refresh.clone();
            let app_state = app_state_for_refresh.clone();
            let list = list_for_refresh.clone();
            let item_rows = item_rows_for_refresh.clone();
            let scrolled = scrolled_for_refresh.clone();
            let empty = empty_widget_for_refresh.clone();
            let empty_state = empty_state_for_refresh.clone();
            glib::MainContext::default().spawn_local(async move {
                match service.history(request.clone()).await {
                    Ok(entries)
                        if crate::service::accepts_generation(latest.get(), request.generation) =>
                    {
                        let query_empty = state.borrow().query.is_empty();
                        let items = entries.iter().map(entry_to_item).collect();
                        crate::app::reduce(
                            &mut app_state.borrow_mut(),
                            crate::app::Action::ItemsLoaded(items),
                        );
                        rebuild_list(&list, &item_rows, &entries, query_empty);
                        let selected_id = app_state.borrow().selected_id;
                        if let Some(id) = selected_id {
                            if let Some(row) = item_rows.borrow().iter().find(|row| row.id() == id)
                            {
                                list.select_row(Some(row.row()));
                            } else {
                                list.unselect_all();
                            }
                        } else {
                            list.unselect_all();
                        }
                        scrolled.set_visible(!entries.is_empty());
                        empty.set_visible(entries.is_empty());
                    }
                    Err(error)
                        if crate::service::accepts_generation(latest.get(), request.generation) =>
                    {
                        tracing::warn!(%error, "clipboard service request failed");
                        scrolled.set_visible(false);
                        empty_state.set_error(&error.to_string());
                        empty.set_visible(true);
                    }
                    _ => {}
                }
            });
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
    list_section.append(empty_state_ref.widget());
    empty_state_ref.widget().set_visible(false);
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

    // Initial load is asynchronous; construction never performs socket I/O.
    refresh();

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
fn show_collection_chooser(
    parent: &GtkBox,
    item_id: i64,
    service: std::sync::Arc<dyn ClipboardService>,
) {
    let Some(window) = parent
        .root()
        .and_then(|root| root.downcast::<gtk4::Window>().ok())
    else {
        return;
    };
    show_collection_chooser_for_window(&window, item_id, service);
}

/// Open the collection chooser for a stable selected item ID.
#[allow(deprecated)]
pub fn show_collection_chooser_for_window(
    window: &impl IsA<gtk4::Window>,
    item_id: i64,
    service: std::sync::Arc<dyn ClipboardService>,
) {
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
    let dialog_for_load = dialog.clone();
    let list_for_load = list.clone();
    glib::MainContext::default().spawn_local(async move {
        match service.command(IpcCommand::ListCollections).await {
            Ok(data) => {
                let collections: Vec<(String, String)> = data
                    .get("collections")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|value| {
                        Some((
                            value.get("id")?.as_str()?.to_owned(),
                            value.get("name")?.as_str()?.to_owned(),
                        ))
                    })
                    .collect();
                if collections.is_empty() {
                    dialog_for_load
                        .content_area()
                        .append(&gtk4::Label::new(Some(
                            "Create a collection in the Collections page first.",
                        )));
                } else {
                    for (_, name) in &collections {
                        list_for_load.append(&gtk4::Label::new(Some(name)));
                    }
                    let dialog_for_row = dialog_for_load.clone();
                    let service = service.clone();
                    list_for_load.connect_row_activated(move |_list, row| {
                        let Some(index) = usize::try_from(row.index()).ok() else {
                            return;
                        };
                        let Some(collection) = collections.get(index) else {
                            return;
                        };
                        let service = service.clone();
                        let dialog = dialog_for_row.clone();
                        let collection_id = collection.0.clone();
                        glib::MainContext::default().spawn_local(async move {
                            if service
                                .command(IpcCommand::AddToCollection {
                                    collection_id,
                                    item_id,
                                })
                                .await
                                .is_ok()
                            {
                                dialog.close();
                            }
                        });
                    });
                    dialog_for_load.content_area().append(&list_for_load);
                }
            }
            Err(error) => dialog_for_load
                .content_area()
                .append(&gtk4::Label::new(Some(&format!(
                    "Could not load collections: {error}"
                )))),
        }
    });
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
    generation: u64,
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
            generation: 0,
        }
    }
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
