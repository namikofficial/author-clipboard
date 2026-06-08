//! author-clipboard-hypr-picker: First-party Hyprland/wlroots clipboard picker
//!
//! A lightweight, keyboard-first clipboard picker that opens as a Wayland
//! layer-shell overlay. Designed for Hyprland keybinds but works on any
//! wlroots compositor with layer-shell support.
//!
//! Usage:
//!   author-clipboard-hypr-picker [--source history|snippets|emoji|symbols|kaomoji|all]
//!                                 [--count 50] [--include-sensitive]

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use author_clipboard_shared::config::Config;
use author_clipboard_shared::db::Database;
use author_clipboard_shared::picker::{
    self, PickerAction, PickerEntry, PickerError, PickerOptions, PickerSource,
};
use clap::Parser;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{self as gtk};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const APP_ID: &str = "com.namikofficial.author-clipboard-hypr-picker";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentFilter {
    All,
    Text,
    Images,
    Files,
    Pinned,
    Sensitive,
}

struct PickerState {
    entries: Vec<PickerEntry>,
    filtered: Vec<PickerEntry>,
    selected_index: usize,
    source: PickerSource,
    include_sensitive: bool,
    action: PickerAction,
    pending_sensitive_key: Option<String>,
    status_message: Option<String>,
    search_debounce_source: Option<glib::SourceId>,
    pending_query: Option<String>,
    content_filter: ContentFilter,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "author-clipboard-hypr-picker",
    version,
    about = "First-party Hyprland/wlroots clipboard picker"
)]
struct Cli {
    #[arg(
        short,
        long,
        default_value = "history",
        value_parser = ["history", "snippets", "emoji", "symbols", "kaomoji", "all"]
    )]
    source: String,
    #[arg(short, long, default_value = "50")]
    count: usize,
    #[arg(long)]
    include_sensitive: bool,
    #[arg(short, long, default_value = "copy", value_parser = ["copy", "quick-paste"])]
    action: String,
    #[arg(short, long)]
    query: Option<String>,
}

#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<()> {
    let cli = Cli::parse();
    let source = cli.source.parse().unwrap_or(PickerSource::History);
    let action = cli.action.parse().unwrap_or(PickerAction::Copy);
    let options = PickerOptions {
        source,
        limit: cli.count,
        query: cli.query,
        include_sensitive: cli.include_sensitive,
        action,
    };
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| build_ui(app, options.clone()));
    app.run();
    Ok(())
}

fn uncheck_others(buttons: &[gtk::ToggleButton], active_idx: usize) {
    for (i, btn) in buttons.iter().enumerate() {
        if i != active_idx {
            btn.set_active(false);
        }
    }
}

fn apply_filter(
    state: &Rc<RefCell<PickerState>>,
    filter: ContentFilter,
    list_box: &gtk::ListBox,
    status: &gtk::Label,
) {
    let mut s = state.borrow_mut();
    s.content_filter = filter;
    let query = s.pending_query.clone().unwrap_or_default();

    let filtered = filter_entries(&s.entries, &query, filter);
    s.filtered = filtered;
    s.selected_index = 0;
    drop(s);

    populate_list_box(list_box, &state.borrow());
    update_status(status, &state.borrow());

    if let Some(row) = list_box.row_at_index(0) {
        list_box.select_row(Some(&row));
    }
}

fn filter_entries(entries: &[PickerEntry], query: &str, filter: ContentFilter) -> Vec<PickerEntry> {
    let base_filtered = if query.is_empty() {
        entries.to_vec()
    } else {
        picker::filter_entries(entries, query)
    };

    base_filtered
        .into_iter()
        .filter(|e| match filter {
            ContentFilter::All => true,
            ContentFilter::Text => matches!(
                e.content_type,
                Some(author_clipboard_shared::types::ContentType::Text)
            ),
            ContentFilter::Images => matches!(
                e.content_type,
                Some(author_clipboard_shared::types::ContentType::Image)
            ),
            ContentFilter::Files => matches!(
                e.content_type,
                Some(author_clipboard_shared::types::ContentType::Files)
            ),
            ContentFilter::Pinned => e.pinned,
            ContentFilter::Sensitive => e.sensitive,
        })
        .collect()
}

#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
fn build_ui(app: &gtk::Application, options: PickerOptions) {
    let config = Config::load();
    let db = Database::open(&config.db_path()).expect("Failed to open database");

    let entries = picker::load_entries(&db, &config, &options).unwrap_or_default();

    let state = Rc::new(RefCell::new(PickerState {
        entries: entries.clone(),
        filtered: entries,
        selected_index: 0,
        source: options.source,
        include_sensitive: options.include_sensitive,
        action: options.action,
        pending_sensitive_key: None,
        status_message: None,
        search_debounce_source: None,
        pending_query: None,
        content_filter: ContentFilter::All,
    }));

    let width: i32 = config.picker.width.try_into().unwrap_or(720);
    let height: i32 = config.picker.height.try_into().unwrap_or(520);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Author Clipboard")
        .default_width(width)
        .default_height(height)
        .decorated(false)
        .resizable(false)
        .build();

    // Layer-shell setup
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);

    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search clipboard\u{2026}"));
    main_box.append(&search_entry);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);
    scrolled.set_min_content_height(300);

    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);

    populate_list_box(&list_box, &state.borrow());

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    let status_label = gtk::Label::new(None);
    update_status(&status_label, &state.borrow());
    main_box.append(&status_label);

    let filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let state_for_filter = Rc::clone(&state);
    let list_for_filter = list_box.clone();
    let status_for_filter = status_label.clone();

    let btn_all = gtk::ToggleButton::with_label("All");
    btn_all.set_active(true);
    let btn_text = gtk::ToggleButton::with_label("Text");
    let btn_images = gtk::ToggleButton::with_label("Images");
    let btn_files = gtk::ToggleButton::with_label("Files");
    let btn_pinned = gtk::ToggleButton::with_label("Pinned");
    let btn_sensitive = gtk::ToggleButton::with_label("Sensitive");

    let filter_buttons: [gtk::ToggleButton; 6] = [
        btn_all.clone(),
        btn_text.clone(),
        btn_images.clone(),
        btn_files.clone(),
        btn_pinned.clone(),
        btn_sensitive.clone(),
    ];
    for (idx, btn) in filter_buttons.iter().enumerate() {
        btn.set_has_tooltip(true);
        match idx {
            0 => btn.set_tooltip_text(Some("Show all items")),
            1 => btn.set_tooltip_text(Some("Text only")),
            2 => btn.set_tooltip_text(Some("Images only")),
            3 => btn.set_tooltip_text(Some("Files only")),
            4 => btn.set_tooltip_text(Some("Pinned items")),
            5 => btn.set_tooltip_text(Some("Sensitive items")),
            _ => {}
        }
        filter_box.append(btn);
    }

    let state_c = Rc::clone(&state_for_filter);
    let list_c = list_for_filter.clone();
    let status_c = status_for_filter.clone();
    let filter_buttons_all = filter_buttons.clone();
    btn_all.connect_toggled(move |b| {
        if b.is_active() {
            uncheck_others(&filter_buttons_all, 0);
            apply_filter(&state_c, ContentFilter::All, &list_c, &status_c);
        }
    });

    let state_c = Rc::clone(&state_for_filter);
    let list_c = list_for_filter.clone();
    let status_c = status_for_filter.clone();
    let filter_buttons_text = filter_buttons.clone();
    btn_text.connect_toggled(move |b| {
        if b.is_active() {
            uncheck_others(&filter_buttons_text, 1);
            apply_filter(&state_c, ContentFilter::Text, &list_c, &status_c);
        }
    });

    let state_c = Rc::clone(&state_for_filter);
    let list_c = list_for_filter.clone();
    let status_c = status_for_filter.clone();
    let filter_buttons_images = filter_buttons.clone();
    btn_images.connect_toggled(move |b| {
        if b.is_active() {
            uncheck_others(&filter_buttons_images, 2);
            apply_filter(&state_c, ContentFilter::Images, &list_c, &status_c);
        }
    });

    let state_c = Rc::clone(&state_for_filter);
    let list_c = list_for_filter.clone();
    let status_c = status_for_filter.clone();
    let filter_buttons_files = filter_buttons.clone();
    btn_files.connect_toggled(move |b| {
        if b.is_active() {
            uncheck_others(&filter_buttons_files, 3);
            apply_filter(&state_c, ContentFilter::Files, &list_c, &status_c);
        }
    });

    let state_c = Rc::clone(&state_for_filter);
    let list_c = list_for_filter.clone();
    let status_c = status_for_filter.clone();
    let filter_buttons_pinned = filter_buttons.clone();
    btn_pinned.connect_toggled(move |b| {
        if b.is_active() {
            uncheck_others(&filter_buttons_pinned, 4);
            apply_filter(&state_c, ContentFilter::Pinned, &list_c, &status_c);
        }
    });

    let state_c = Rc::clone(&state_for_filter);
    let list_c = list_for_filter.clone();
    let status_c = status_for_filter.clone();
    let filter_buttons_sensitive = filter_buttons.clone();
    btn_sensitive.connect_toggled(move |b| {
        if b.is_active() {
            uncheck_others(&filter_buttons_sensitive, 5);
            apply_filter(&state_c, ContentFilter::Sensitive, &list_c, &status_c);
        }
    });

    main_box.append(&filter_box);

    window.set_child(Some(&main_box));

    // ── Keyboard controller ────────────────────────────────────

    let key_controller = gtk::EventControllerKey::new();
    let window_ref = window.clone();
    let state_clone = Rc::clone(&state);
    let list_box_ref = list_box.clone();
    let status_ref = status_label.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifiers| {
        let mut s = state_clone.borrow_mut();
        let len = s.filtered.len();
        if len == 0 {
            return glib::Propagation::Proceed;
        }

        let key_name = key
            .name()
            .map(|n| n.as_str().to_owned())
            .unwrap_or_default();

        match key_name.as_str() {
            "Escape" => {
                window_ref.close();
                glib::Propagation::Stop
            }
            "Down" => {
                s.selected_index = (s.selected_index + 1).min(len - 1);
                select_row_at(&list_box_ref, &s, &status_ref, &state_clone);
                glib::Propagation::Stop
            }
            "Up" => {
                s.selected_index = s.selected_index.saturating_sub(1);
                select_row_at(&list_box_ref, &s, &status_ref, &state_clone);
                glib::Propagation::Stop
            }
            "Return" => {
                if let Some(entry) = s.filtered.get(s.selected_index) {
                    let entry = entry.clone();
                    let action = if modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                        PickerAction::QuickPaste
                    } else {
                        s.action
                    };
                    drop(s);
                    handle_entry_action(
                        &state_clone,
                        &entry,
                        action,
                        &window_ref,
                        &status_ref,
                        Some(&list_box_ref),
                    );
                }
                glib::Propagation::Stop
            }
            "Delete" => {
                handle_delete(&mut s, &list_box_ref, &state_clone, &status_ref);
                glib::Propagation::Stop
            }
            _ => {
                if modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    if key_name.eq_ignore_ascii_case("p") {
                        handle_toggle_pin(&mut s, &list_box_ref, &state_clone, &status_ref);
                        return glib::Propagation::Stop;
                    }
                    if let Some(num) = key_name.chars().next().and_then(|c| c.to_digit(10)) {
                        let idx = (num as usize).saturating_sub(1);
                        if idx < len {
                            if let Some(entry) = s.filtered.get(idx) {
                                let entry = entry.clone();
                                drop(s);
                                handle_entry_action(
                                    &state_clone,
                                    &entry,
                                    PickerAction::Copy,
                                    &window_ref,
                                    &status_ref,
                                    Some(&list_box_ref),
                                );
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                }
                glib::Propagation::Proceed
            }
        }
    });

    window.add_controller(key_controller.clone().upcast::<gtk::EventController>());

    // ── Search filtering ───────────────────────────────────────

    let state_for_search = Rc::clone(&state);
    let list_box_clone = list_box.clone();
    let status_label_clone = status_label.clone();

    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        let mut s = state_for_search.borrow_mut();

        // Cancel any existing debounce timeout
        if let Some(source_id) = s.search_debounce_source.take() {
            source_id.remove();
        }

        // Store the pending query
        s.pending_query = Some(query.clone());
        s.pending_sensitive_key = None;
        s.status_message = None;

        // Schedule a new debounced search (200ms delay)
        let state_clone = Rc::clone(&state_for_search);
        let list_clone = list_box_clone.clone();
        let status_clone = status_label_clone.clone();
        let query_clone = query.clone();

        let source_id = glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            let mut s = state_clone.borrow_mut();
            let current_query = s.pending_query.take().unwrap_or_default();

            // Only process if this is still the current query
            if current_query != query_clone {
                return glib::ControlFlow::Break;
            }

            let filter = s.content_filter;
            drop(s);

            let filtered = filter_entries(&state_clone.borrow().entries, &current_query, filter);
            state_clone.borrow_mut().filtered = filtered;
            state_clone.borrow_mut().selected_index = 0;
            state_clone.borrow_mut().search_debounce_source = None;

            populate_list_box(&list_clone, &state_clone.borrow());
            update_status(&status_clone, &state_clone.borrow());

            if let Some(row) = list_clone.row_at_index(0) {
                list_clone.select_row(Some(&row));
            }

            glib::ControlFlow::Break
        });

        s.search_debounce_source = Some(source_id);
    });

    search_entry.grab_focus();

    if let Some(row) = list_box.row_at_index(0) {
        list_box.select_row(Some(&row));
    }

    window.present();
}

fn select_row_at(
    list_box: &gtk::ListBox,
    state: &PickerState,
    status: &gtk::Label,
    state_ref: &Rc<RefCell<PickerState>>,
) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let idx = state.selected_index as i32;
    if let Some(row) = list_box.row_at_index(idx) {
        list_box.select_row(Some(&row));
        row.grab_focus();
    }
    update_status(status, &state_ref.borrow());
}

fn handle_delete(
    state: &mut PickerState,
    list_box: &gtk::ListBox,
    state_ref: &Rc<RefCell<PickerState>>,
    status: &gtk::Label,
) {
    if let Some(entry) = state.filtered.get(state.selected_index) {
        if let Some(id) = entry.id {
            let config = Config::load();
            if let Ok(db) = Database::open(&config.db_path()) {
                let _ = db.delete_item(id);
                let opts = PickerOptions {
                    source: state.source,
                    limit: state.entries.len(),
                    query: None,
                    include_sensitive: state.include_sensitive,
                    action: state.action,
                };
                if let Ok(new_entries) = picker::load_entries(&db, &config, &opts) {
                    state.entries = new_entries;
                    let cloned = state.entries.clone();
                    state.filtered = cloned;
                    state.selected_index = state
                        .selected_index
                        .min(state.filtered.len().saturating_sub(1));
                    state.pending_sensitive_key = None;
                    state.status_message = Some("Item deleted".to_string());
                }
                populate_list_box(list_box, &state_ref.borrow());
                update_status(status, &state_ref.borrow());
            }
        }
    }
}

fn populate_list_box(list_box: &gtk::ListBox, state: &PickerState) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    for entry in &state.filtered {
        let label_text = if entry.sensitive {
            let age = entry
                .timestamp
                .as_ref()
                .map(picker::format_age)
                .unwrap_or_default();
            let pin_badge = if entry.pinned { " \u{1f4cc}" } else { "" };
            let age_chip = if age.is_empty() {
                String::new()
            } else {
                format!("  \u{00b7}  {age}")
            };
            format!(
                "\u{1f512}  {}  (sensitive){age_chip}{pin_badge}",
                entry.title
            )
        } else {
            let icon = entry
                .content_type
                .as_ref()
                .map_or("text", |ct| picker::content_type_icon(ct));

            let title = &entry.title;
            let subtitle = entry.subtitle.as_deref().unwrap_or("");
            let age = entry
                .timestamp
                .as_ref()
                .map(picker::format_age)
                .unwrap_or_default();

            let pin_badge = if entry.pinned { " \u{1f4cc}" } else { "" };
            let age_chip = if age.is_empty() {
                String::new()
            } else {
                format!("  \u{00b7}  {age}")
            };

            let subtitle_part = if subtitle.is_empty() {
                String::new()
            } else {
                format!("  \u{00b7}  {subtitle}")
            };

            format!("{icon}  {title}{subtitle_part}{age_chip}{pin_badge}")
        };
        let label = gtk::Label::builder()
            .label(&label_text)
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .build();
        list_box.append(&label);
    }
}

fn update_status(label: &gtk::Label, state: &PickerState) {
    if let Some(message) = &state.status_message {
        label.set_text(message);
        return;
    }

    let total = state.filtered.len();
    let selected = state.selected_index + 1;
    if total == 0 {
        let filter_name = match state.content_filter {
            ContentFilter::All => "items",
            ContentFilter::Text => "text items",
            ContentFilter::Images => "images",
            ContentFilter::Files => "files",
            ContentFilter::Pinned => "pinned items",
            ContentFilter::Sensitive => "sensitive items",
        };
        let query_part = state
            .pending_query
            .as_ref()
            .filter(|q| !q.is_empty())
            .map(|q| format!(" matching \"{q}\""))
            .unwrap_or_default();
        label.set_text(&format!(
            "No {filter_name} found{query_part}  \u{00b7}  try a different filter or search"
        ));
    } else {
        label.set_text(&format!(
            "{selected}/{total}  \u{00b7}  \u{2191}\u{2193} navigate  \u{00b7}  Enter copy  \u{00b7}  Ctrl+Enter quick-paste  \u{00b7}  Ctrl+P pin  \u{00b7}  Ctrl+1-9 quick  \u{00b7}  Esc close"
        ));
    }
}

fn entry_selection_key(entry: &PickerEntry) -> String {
    match entry.id {
        Some(id) => format!("id:{id}"),
        None => format!("content:{}", entry.content),
    }
}

fn handle_entry_action(
    state_ref: &Rc<RefCell<PickerState>>,
    entry: &PickerEntry,
    action: PickerAction,
    window: &gtk::ApplicationWindow,
    status: &gtk::Label,
    list_box: Option<&gtk::ListBox>,
) {
    let config = Config::load();

    if entry.sensitive && config.picker.confirm_sensitive_copy {
        let key = entry_selection_key(entry);
        let needs_confirmation = {
            let mut state = state_ref.borrow_mut();
            let already_confirmed = state.pending_sensitive_key.as_deref() == Some(key.as_str());
            if already_confirmed {
                false
            } else {
                state.pending_sensitive_key = Some(key);
                state.status_message =
                    Some("Sensitive item selected. Press Enter again to confirm copy.".to_string());
                true
            }
        };

        if needs_confirmation {
            update_status(status, &state_ref.borrow());
            return;
        }
    }

    let confirmed_sensitive = entry.sensitive;
    match picker::restore_entry(entry, &config, action, confirmed_sensitive) {
        Ok(result) => {
            {
                let mut state = state_ref.borrow_mut();
                state.pending_sensitive_key = None;
                state.status_message = Some(format!(
                    "Copied as {} ({})",
                    result.mime_type, result.behavior
                ));
            }
            update_status(status, &state_ref.borrow());
            if config.picker.close_after_copy {
                window.close();
            } else if let Some(list) = list_box {
                select_row_at(list, &state_ref.borrow(), status, state_ref);
            }
        }
        Err(PickerError::SensitiveConfirmationRequired) => {
            let mut state = state_ref.borrow_mut();
            state.status_message =
                Some("Sensitive item requires confirmation. Press Enter again.".to_string());
            update_status(status, &state);
        }
        Err(err) => {
            let mut state = state_ref.borrow_mut();
            state.status_message = Some(format!("Restore failed: {err}"));
            update_status(status, &state);
        }
    }
}

fn handle_toggle_pin(
    state: &mut PickerState,
    list_box: &gtk::ListBox,
    state_ref: &Rc<RefCell<PickerState>>,
    status: &gtk::Label,
) {
    if let Some(entry) = state.filtered.get(state.selected_index).cloned() {
        if let Some(id) = entry.id {
            let config = Config::load();
            if let Ok(db) = Database::open(&config.db_path()) {
                let _ = db.toggle_pin(id);
                let opts = PickerOptions {
                    source: state.source,
                    limit: state.entries.len(),
                    query: None,
                    include_sensitive: state.include_sensitive,
                    action: state.action,
                };
                if let Ok(new_entries) = picker::load_entries(&db, &config, &opts) {
                    state.entries.clone_from(&new_entries);
                    state.filtered = new_entries;
                    if let Some(new_index) = state.filtered.iter().position(|e| e.id == Some(id)) {
                        state.selected_index = new_index;
                    } else {
                        state.selected_index = state
                            .selected_index
                            .min(state.filtered.len().saturating_sub(1));
                    }
                    state.status_message = Some("Pin state updated".to_string());
                }
                state.pending_sensitive_key = None;
                populate_list_box(list_box, &state_ref.borrow());
                update_status(status, &state_ref.borrow());
                select_row_at(list_box, &state_ref.borrow(), status, state_ref);
            }
        }
    }
}
