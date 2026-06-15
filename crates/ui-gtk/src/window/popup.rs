//! Layer-shell popup. Uses the global key controller from PR 4
//! instead of inline `EventControllerKey` handlers for Esc and `/`.
//! Popup size persisted via `GSettings`.
//!
//! The shell is a vertical stack of three CSS-styled sections:
//!
//! ```text
//!   .popup-section-search   ← search entry
//!   .popup-section-filter   ← filter bar
//!   .popup-section-list     ← scrollable list / empty state
//! ```
//!
//! A `.popup-status` label sits below the list, separated by a
//! 1px border. All paddings are driven by the spacing scale in
//! `data/style.css` so the popup and the manager stay in sync.

use crate::controller::focus::FocusTarget;
use crate::settings::Settings;
use crate::PopupConfig;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use libadwaita as adw;
use libadwaita::prelude::AdwWindowExt;

/// Run the popup. Blocks until the window closes.
pub fn run(config: PopupConfig) -> anyhow::Result<()> {
    tracing::info!(?config, "ui-gtk popup starting");

    let app = adw::Application::builder()
        .application_id("com.namikofficial.author-clipboard.popup")
        .build();

    app.connect_activate(move |app| {
        if let Err(e) = build_popup(app, &config) {
            tracing::error!(?e, "failed to build popup");
            app.quit();
        }
    });

    let args: Vec<String> = vec!["author-clipboard-popup".to_string()];
    let _ = app.run_with_args(&args);
    Ok(())
}

#[allow(clippy::unnecessary_wraps, clippy::too_many_lines)]
fn build_popup(app: &adw::Application, config: &PopupConfig) -> anyhow::Result<()> {
    let settings = Settings::new();
    let (default_w, default_h) = settings.as_ref().map_or((720, 520), Settings::popup_size);

    let window = adw::Window::builder()
        .application(app)
        .title("Clipboard")
        .default_width(default_w)
        .default_height(default_h)
        .resizable(true)
        .build();

    // ── Layer-shell init ─────────────────────────────────────
    if gtk4_layer_shell::is_supported() {
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Overlay);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    } else {
        tracing::warn!("layer-shell not supported; popup will use XDG window");
    }

    // ── Shared state ─────────────────────────────────────────
    let state: std::rc::Rc<std::cell::RefCell<crate::app::AppState>> =
        std::rc::Rc::new(std::cell::RefCell::new(crate::app::AppState {
            mode: crate::app::AppMode::Popup,
            ..Default::default()
        }));

    // ── Real clipboard page (data via IPC) ────────────────────
    let props = crate::pages::clipboard::ClipboardPageProps {
        initial_query: config.query.clone().unwrap_or_default(),
        initial_filter: config.filter,
        count: config.count,
    };
    let window_for_copy = window.clone();
    let page = crate::pages::clipboard::build(&props, move |req| {
        tracing::info!(id = req.id, mime = %req.mime, "popup copy");
        if let Err(e) = crate::pages::clipboard::copy_via_ipc(req.id, &req.mime) {
            tracing::warn!(?e, "popup copy failed");
        }
        // US-001: close after a successful copy (or failure — we'd rather
        // lose the popup than keep it open if the user pressed Enter).
        window_for_copy.close();
    });

    // ── Status hint ───────────────────────────────────────────
    let status = gtk4::Label::new(Some("↑↓ navigate · / search · Enter copy · Esc close"));
    status.set_halign(gtk4::Align::Start);
    status.add_css_class("popup-status");

    // ── Shell: page above, status below ──────────────────────
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.add_css_class("popup-shell");
    content.append(&page);
    content.append(&status);

    window.set_content(Some(&content));

    // ── US-001/US-002: Global key controller ─────────────────
    let (tx, rx) = std::sync::mpsc::channel::<crate::Effect>();
    crate::controller::key::install(&window, &state, &tx);

    // Handle effects from the channel on the GTK thread.
    // Esc close and search focus are handled by the key controller
    // via the reducer + effect channel.
    let window_for_effects = window.clone();
    glib::idle_add_local(move || {
        while let Ok(eff) = rx.try_recv() {
            match eff {
                crate::Effect::Quit => {
                    window_for_effects.close();
                }
                crate::Effect::AddToast(ref msg) => {
                    tracing::info!(%msg, "popup toast");
                }
                _ => {}
            }
        }
        glib::ControlFlow::Continue
    });

    // ── Track focus changes in AppState for Esc resolution ───
    let state_for_focus = state.clone();
    let content_for_focus: gtk4::Widget = content.clone().upcast();
    if let Some(search) = find_search_entry(&content_for_focus) {
        let s = state_for_focus.clone();
        search.connect_has_focus_notify(move |entry| {
            s.borrow_mut().focus = if entry.has_focus() {
                FocusTarget::Search
            } else {
                FocusTarget::List
            };
        });
    }
    if let Some(list) = find_list_box(&content_for_focus) {
        list.connect_row_selected(move |_, _| {
            state_for_focus.borrow_mut().focus = FocusTarget::List;
        });
    }

    // ── Size persistence (debounced) ─────────────────────────
    let settings_for_size = settings.clone();
    let window_for_size = window.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        if let Some(ref s) = settings_for_size {
            let (w, h) = (window_for_size.width(), window_for_size.height());
            s.set_popup_size(w, h);
        }
        glib::ControlFlow::Continue
    });

    // ── Open with the list focused ────────────────────────────
    let content_for_focus: gtk4::Widget = content.clone().upcast();
    window.connect_map(move |_| {
        if let Some(list) = find_list_box(&content_for_focus) {
            list.grab_focus();
        }
    });

    window.present();
    Ok(())
}

fn find_search_entry(widget: &gtk4::Widget) -> Option<gtk4::SearchEntry> {
    if let Ok(entry) = widget.clone().downcast::<gtk4::SearchEntry>() {
        return Some(entry);
    }
    let mut child = widget.first_child();
    while let Some(c) = child {
        if let Some(found) = find_search_entry(&c) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}

fn find_list_box(widget: &gtk4::Widget) -> Option<gtk4::ListBox> {
    if let Ok(list) = widget.clone().downcast::<gtk4::ListBox>() {
        return Some(list);
    }
    let mut child = widget.first_child();
    while let Some(c) = child {
        if let Some(found) = find_list_box(&c) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}
