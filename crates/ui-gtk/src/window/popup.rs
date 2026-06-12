//! Layer-shell popup. Real implementation in T014.
//!
//! For the bug-fix slice (T016-T018) we ship a minimal but
//! *functional* popup: it opens a window, shows a placeholder list,
//! and closes on Esc (US-001) regardless of focus.

use crate::controller::focus::{resolve_escape, EscOutcome, FocusTarget};
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

    // Use a per-thread main context for embedding in callers that
    // already have a tokio runtime.
    let app = adw::Application::builder()
        .application_id("com.namikofficial.author-clipboard.popup")
        .build();

    app.connect_activate(move |app| {
        if let Err(e) = build_popup_window(app, &config) {
            tracing::error!(?e, "failed to build popup");
            app.quit();
        }
    });

    // Empty args; the popup mode is signal-driven, not CLI-arg-driven.
    let args: Vec<String> = vec!["author-clipboard-popup".to_string()];
    let _ = app.run_with_args(&args);
    Ok(())
}

fn build_popup_window(
    app: &adw::Application,
    config: &PopupConfig,
) -> anyhow::Result<()> {
    let window = adw::Window::builder()
        .application(app)
        .title("Clipboard")
        .default_width(720)
        .default_height(520)
        .resizable(false)
        .build();

    // ── Layer-shell init ────────────────────────────────────────────
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

    // ── Layout: search + list placeholder ──────────────────────────
    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let search = gtk4::SearchEntry::new();
    search.set_placeholder_text(Some("Search…"));
    search.set_hexpand(true);
    header.append(&search);

    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    for i in 0..5 {
        let row = adw::ActionRow::builder()
            .title(format!("Item {i} (placeholder)"))
            .subtitle("Real rows land in T013")
            .build();
        list_box.append(&row);
    }

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    content.append(&header);
    content.append(&list_box);

    let status = gtk4::Label::new(Some("↑↓ navigate · / search · Enter copy · Esc close"));
    status.set_margin_top(4);
    status.set_margin_bottom(4);
    content.append(&status);

    window.set_content(Some(&content));

    // ── US-002: open with list focused, not search ──────────────────
    // AdwWindow inherits `set_default_widget`; we use that so Enter
    // on the window fires the list's default action. Focus is set
    // explicitly via `grab_focus` after present.
    window.set_default_widget(Some(&list_box));
    let list_for_initial_focus = list_box.clone();
    window.connect_map(move |_| {
        list_for_initial_focus.grab_focus();
    });

    // ── US-001: global Esc handler in Capture phase ────────────────
    let window_for_esc = window.clone();
    let list_for_focus = list_box.clone();
    let search_for_focus = search.clone();
    let focus_state = std::cell::RefCell::new(FocusTarget::List);

    // Track focus changes
    let focus_state_for_search = focus_state.clone();
    search.connect_has_focus_notify(move |entry| {
        if entry.has_focus() {
            *focus_state_for_search.borrow_mut() = FocusTarget::Search;
        } else {
            *focus_state_for_search.borrow_mut() = FocusTarget::List;
        }
    });
    let focus_state_for_list = focus_state.clone();
    list_box.connect_row_selected(move |_, _| {
        *focus_state_for_list.borrow_mut() = FocusTarget::List;
    });

    let esc_controller = gtk4::EventControllerKey::new();
    esc_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    esc_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            let query_empty = search_for_focus.text().is_empty();
            let outcome = resolve_escape(*focus_state.borrow(), query_empty);
            match outcome {
                EscOutcome::ClearSearch => {
                    search_for_focus.set_text("");
                    list_for_focus.grab_focus();
                }
                EscOutcome::BlurSearch => {
                    list_for_focus.grab_focus();
                }
                EscOutcome::Close => {
                    window_for_esc.close();
                }
                EscOutcome::Proceed => {}
            }
            return glib::Propagation::Stop;
        }
        if key == gdk::Key::slash && !search_for_focus.has_focus() {
            search_for_focus.grab_focus();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(esc_controller);

    window.present();
    Ok(())
}
