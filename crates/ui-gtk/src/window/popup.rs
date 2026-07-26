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

#[allow(deprecated, clippy::unnecessary_wraps, clippy::too_many_lines)]
fn build_popup(app: &adw::Application, config: &PopupConfig) -> anyhow::Result<()> {
    let settings = Settings::new();
    let (default_w, default_h) = settings.as_ref().map_or((780, 620), Settings::popup_size);

    let window = adw::Window::builder()
        .application(app)
        .title("Clipboard")
        .default_width(default_w)
        .default_height(default_h)
        .resizable(true)
        .build();
    window.set_size_request(540, 420);
    let app_for_close = app.clone();
    window.connect_close_request(move |_| {
        app_for_close.quit();
        glib::Propagation::Proceed
    });

    // ── Layer-shell init ─────────────────────────────────────
    if config.layer_shell && gtk4_layer_shell::is_supported() {
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Overlay);
        window.set_namespace(Some("author-clipboard-picker"));
        // Do not reserve an exclusive zone — the popup should not push
        // other windows out of the way.
        window.set_exclusive_zone(0);
        // Anchor to top; Left and Right anchors make it span the
        // focused monitor width.
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    } else if config.layer_shell {
        tracing::warn!("layer-shell not supported; popup will use XDG window");
    }

    // ── Shared state ─────────────────────────────────────────
    let state: std::rc::Rc<std::cell::RefCell<crate::app::AppState>> =
        std::rc::Rc::new(std::cell::RefCell::new(crate::app::AppState {
            mode: crate::app::AppMode::Popup,
            ..Default::default()
        }));

    // ── Real clipboard page (data via IPC) ────────────────────
    let shared_config = author_clipboard_shared::config::Config::load();
    let close_after = shared_config.picker.close_after_copy;
    let props = crate::pages::clipboard::ClipboardPageProps {
        initial_query: config.query.clone().unwrap_or_default(),
        initial_filter: config.filter,
        count: config.count,
        source: config.source,
        include_sensitive: config.include_sensitive,
        action: config.action,
    };
    let window_for_copy = window.clone();
    let page = crate::pages::clipboard::build(&props, &state, move |req| {
        tracing::info!(id = req.id, mime = %req.mime, mode = ?req.mode, "popup action");
        if let Err(e) = crate::pages::clipboard::copy_via_ipc(req.id, &req.mime, req.mode) {
            tracing::warn!(?e, "popup copy failed");
        }
        if close_after {
            window_for_copy.close();
        }
    });

    // ── Status hint ───────────────────────────────────────────
    let status = gtk4::Label::new(Some(
        "↑↓ navigate · / search · Enter copy · Esc or close button to dismiss · resize from window edges",
    ));
    status.set_halign(gtk4::Align::Start);
    status.add_css_class("popup-status");

    // ── Shell: page above, status below ──────────────────────
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.add_css_class("popup-shell");
    content.append(&build_popup_header(&window, config));
    content.append(&page);
    let state_for_rail = state.clone();
    let window_for_rail = window.clone();
    content.append(&crate::widgets::action_bar::build(move |action| {
        use crate::widgets::action_bar::RailAction;
        use author_clipboard_shared::ipc::{CopyMode, IpcClient, IpcCommand};
        let state = state_for_rail.borrow();
        let Some(id) = state.selected_id else { return };
        let item = state.items.iter().find(|item| item.id == id);
        let Some(selected) = item.cloned() else {
            return;
        };
        if action == RailAction::Reveal {
            if !selected.sensitive && !selected.encrypted {
                return;
            }
            drop(state);
            crate::app::reduce(
                &mut state_for_rail.borrow_mut(),
                crate::app::Action::RevealRedacted,
            );
            let dialog = gtk4::MessageDialog::builder()
                .transient_for(&window_for_rail)
                .modal(true)
                .text("Protected clipboard item")
                .secondary_text(&selected.content)
                .build();
            dialog.add_button("Hide now", gtk4::ResponseType::Close);
            dialog.connect_response(|dialog, _| dialog.close());
            let dialog_for_timeout = dialog.clone();
            glib::timeout_add_local_once(std::time::Duration::from_secs(5), move || {
                dialog_for_timeout.close();
            });
            dialog.present();
            return;
        }
        if action == RailAction::AddToCollection {
            drop(state);
            crate::pages::clipboard::show_collection_chooser_for_window(&window_for_rail, id);
            return;
        }
        let command = match action {
            RailAction::Copy => IpcCommand::Copy {
                id,
                mode: CopyMode::Copy,
                mime: None,
            },
            RailAction::QuickPaste => IpcCommand::Copy {
                id,
                mode: CopyMode::QuickPaste,
                mime: None,
            },
            RailAction::PlainText => IpcCommand::Copy {
                id,
                mode: CopyMode::CopyPlainText,
                mime: None,
            },
            RailAction::Transform => {
                let transform = if matches!(
                    author_clipboard_shared::presentation::present(&selected),
                    author_clipboard_shared::presentation::ContentPresentation::Json { .. }
                ) {
                    author_clipboard_shared::transform::TransformKind::JsonPretty
                } else {
                    author_clipboard_shared::transform::TransformKind::Quote
                };
                IpcCommand::Transform {
                    content: selected.content.clone(),
                    transform,
                    sensitive: selected.sensitive || selected.encrypted,
                    confirm_sensitive: false,
                }
            }
            RailAction::CreateSnippet if selected.sensitive || selected.encrypted => {
                tracing::warn!(
                    "refusing to create snippet from protected content without confirmation"
                );
                return;
            }
            RailAction::CreateSnippet => IpcCommand::UpsertSnippet {
                name: format!("clipboard-{id}"),
                content: selected.content.clone(),
            },
            RailAction::Pin if item.is_some_and(|item| item.pinned) => IpcCommand::Unpin { id },
            RailAction::Pin => IpcCommand::Pin { id },
            RailAction::Star => IpcCommand::ToggleStar { id },
            RailAction::Delete => IpcCommand::Delete { id },
            RailAction::AddToCollection => unreachable!("handled before IPC command mapping"),
            RailAction::Reveal => unreachable!("handled before selection lookup"),
        };
        drop(state);
        match IpcClient::new().send_command(&command) {
            Ok(response) if action == RailAction::Transform && response.ok => {
                if let Some(output) = response
                    .data
                    .as_ref()
                    .and_then(|data| data.get("output"))
                    .and_then(serde_json::Value::as_str)
                {
                    if let Some(display) = gdk::Display::default() {
                        display.clipboard().set_text(output);
                    }
                }
            }
            Ok(_) => {}
            Err(error) => tracing::warn!(?error, "popup action failed"),
        }
    }));
    content.append(&status);

    window.set_content(Some(&content));

    // ── Focus tracking for Esc resolution ────────────────────
    {
        let s = state.clone();
        let content_w: gtk4::Widget = content.clone().upcast();
        if let Some(search) = find_search_entry(&content_w) {
            search.connect_has_focus_notify(move |entry| {
                s.borrow_mut().focus = if entry.has_focus() {
                    FocusTarget::Search
                } else {
                    FocusTarget::List
                };
            });
        }
    }
    {
        let s = state.clone();
        let content_w: gtk4::Widget = content.clone().upcast();
        if let Some(list) = find_list_box(&content_w) {
            list.connect_row_selected(move |_, _| {
                s.borrow_mut().focus = FocusTarget::List;
            });
        }
    }

    // ── Key controller (bubble phase) ────────────────────────
    let (tx, rx) = std::sync::mpsc::channel::<crate::Effect>();
    {
        let content_w: gtk4::Widget = content.clone().upcast();
        let search = find_search_entry(&content_w);
        let list = find_list_box(&content_w);
        let window_for_close = window.clone();
        crate::controller::key::install(
            &window,
            &state,
            &tx,
            Some(Box::new(move || window_for_close.close())),
            search.as_ref(),
            list.as_ref(),
        );
    }

    // Handle effects from the channel on the GTK thread.
    glib::idle_add_local(move || {
        while let Ok(eff) = rx.try_recv() {
            if let crate::Effect::AddToast(ref msg) = eff {
                tracing::info!(%msg, "popup toast");
            }
        }
        glib::ControlFlow::Continue
    });

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

fn build_popup_header(window: &adw::Window, config: &PopupConfig) -> gtk4::Box {
    let header = gtk4::Box::new(
        gtk4::Orientation::Horizontal,
        crate::theme::spacing::SPACE_MD,
    );
    header.add_css_class("popup-header");
    header.set_hexpand(true);
    header.set_valign(gtk4::Align::Center);

    let title_col = gtk4::Box::new(
        gtk4::Orientation::Vertical,
        crate::theme::spacing::SPACE_2XS,
    );
    title_col.set_hexpand(true);

    let title = gtk4::Label::new(Some("Author Clipboard"));
    title.add_css_class("popup-title");
    title.set_halign(gtk4::Align::Start);
    title.set_xalign(0.0);

    let mode = if config.layer_shell {
        "Hyprland overlay"
    } else {
        "Resizable native window"
    };
    let subtitle = gtk4::Label::new(Some(&format!(
        "{mode} · history, rich text, files, images, snippets"
    )));
    subtitle.add_css_class("popup-subtitle");
    subtitle.set_halign(gtk4::Align::Start);
    subtitle.set_xalign(0.0);
    subtitle.set_wrap(true);

    title_col.append(&title);
    title_col.append(&subtitle);

    let action_chip = gtk4::Label::new(Some(match config.action {
        crate::PickerAction::Copy => "Copy mode",
        crate::PickerAction::QuickPaste => "Quick paste",
    }));
    action_chip.add_css_class("popup-mode-chip");

    let close = gtk4::Button::builder()
        .icon_name("window-close-symbolic")
        .tooltip_text("Close clipboard picker (Esc)")
        .build();
    close.add_css_class("popup-close-button");
    close.add_css_class("circular");
    let window_for_close = window.clone();
    close.connect_clicked(move |_| {
        window_for_close.close();
    });

    header.append(&title_col);
    header.append(&action_chip);
    header.append(&close);
    header
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
