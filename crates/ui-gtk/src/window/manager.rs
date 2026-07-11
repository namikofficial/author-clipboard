//! `AdwApplicationWindow` manager with sidebar, navigation view,
//! persisted size via `GSettings`, status bar, and toast overlay.

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::app::{Action, AppState, Effect, PageId};
use crate::settings::Settings;
use crate::ManagerConfig;

/// Run the manager window. Blocks until the window closes.
#[allow(clippy::needless_pass_by_value)]
pub fn run(config: ManagerConfig) -> anyhow::Result<()> {
    tracing::info!(?config, "ui-gtk manager starting");

    let app = adw::Application::builder()
        .application_id("com.namikofficial.author-clipboard")
        .flags(adw::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| {
        if let Err(e) = build_manager_window(app) {
            tracing::error!(?e, "failed to build manager");
            app.quit();
        }
    });

    let args: Vec<String> = vec!["author-clipboard".to_string()];
    let _ = app.run_with_args(&args);
    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::too_many_lines,
    clippy::items_after_statements
)]
fn build_manager_window(app: &adw::Application) -> anyhow::Result<()> {
    let settings = Settings::new();
    let (default_w, default_h) = settings.as_ref().map_or((1100, 720), Settings::window_size);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Author Clipboard")
        .default_width(default_w)
        .default_height(default_h)
        .build();

    // ── Shared state ──────────────────────────────────────────
    let state: std::rc::Rc<std::cell::RefCell<AppState>> =
        std::rc::Rc::new(std::cell::RefCell::new(AppState {
            mode: crate::app::AppMode::Manager,
            ..Default::default()
        }));

    // ── Header ────────────────────────────────────────────────
    let header = adw::HeaderBar::builder()
        .title_widget(&gtk4::Label::new(Some("Author Clipboard")))
        .build();

    // ── Content pages (built once, cached) ────────────────────
    let shared_config = author_clipboard_shared::config::Config::load();
    let clipboard_props = crate::pages::clipboard::ClipboardPageProps::default();

    // Clipboard page uses a Paned: list on left, preview on right.
    let clipboard_paned = gtk4::Paned::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .position(420)
        .wide_handle(true)
        .resize_start_child(true)
        .shrink_start_child(false)
        .build();

    // Build preview pane (right side of paned)
    let state_for_reveal = state.clone();
    let preview = crate::widgets::preview::PreviewPane::new(
        state.clone(),
        std::rc::Rc::new(move || {
            let mut s = state_for_reveal.borrow_mut();
            let _effects = crate::app::reduce(&mut s, Action::RevealRedacted);
        }),
    );

    let clipboard_page_content =
        crate::pages::clipboard::build(&clipboard_props, &state, move |req| {
            tracing::info!(id = req.id, mime = %req.mime, "manager copy");
            if let Err(e) = crate::pages::clipboard::copy_via_ipc(req.id, &req.mime) {
                tracing::warn!(?e, "manager copy failed");
            }
        });

    clipboard_paned.set_start_child(Some(&clipboard_page_content));
    clipboard_paned.set_end_child(Some(preview.widget()));

    // ── Sidebar list ──────────────────────────────────────────
    // Width bumped from 180→200 to give the icon + label
    // breathing room. The selected row's pill background and
    // padding are driven by the `list.sidebar` rules in
    // `data/style.css`.
    let sidebar_list = gtk4::ListBox::builder().width_request(200).build();
    sidebar_list.add_css_class("sidebar");

    fn make_sidebar_row(label: &str, icon_name: &str, page_tag: &str) -> gtk4::ListBoxRow {
        // The hbox padding / spacing are all driven by the
        // `list.sidebar > row > box` CSS rule, so the Rust
        // side is just a plain hbox. We keep the widget-name
        // pattern intact because the row-activation handler
        // reads it back to figure out which page to switch to.
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let icon = gtk4::Image::from_icon_name(icon_name);
        let label_w = gtk4::Label::new(Some(label));
        label_w.set_halign(gtk4::Align::Start);
        label_w.set_hexpand(true);
        hbox.append(&icon);
        hbox.append(&label_w);
        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&hbox));
        // Store the page tag in the child box so we can recover it later.
        hbox.set_widget_name(page_tag);
        row
    }

    fn sidebar_icon_for(page: PageId) -> &'static str {
        match page {
            PageId::Home => "go-home-symbolic",
            PageId::Clipboard => "edit-paste-symbolic",
            PageId::Collections => "folder-symbolic",
            PageId::Emoji => "smiley-symbolic",
            PageId::Symbols => "insert-symbol-symbolic",
            PageId::Kaomoji => "face-smile-symbolic",
            PageId::Snippets => "document-new-symbolic",
            PageId::Settings => "preferences-system-symbolic",
        }
    }

    let page_labels: &[(PageId, &str)] = &[
        (PageId::Home, "Home"),
        (PageId::Clipboard, "Clipboard"),
        (PageId::Collections, "Collections"),
        (PageId::Emoji, "Emoji"),
        (PageId::Symbols, "Symbols"),
        (PageId::Kaomoji, "Kaomoji"),
        (PageId::Snippets, "Snippets"),
        (PageId::Settings, "Settings"),
    ];

    // Build all pages upfront.
    let mut page_widgets: Vec<(PageId, gtk4::Widget)> = Vec::new();
    for &(page_id, label) in page_labels {
        let icon = sidebar_icon_for(page_id);
        let row = make_sidebar_row(label, icon, &page_id.to_string());
        sidebar_list.append(&row);

        let content_widget: gtk4::Widget = match page_id {
            PageId::Home => crate::pages::home::build(&shared_config).upcast(),
            PageId::Clipboard => clipboard_paned.clone().upcast(),
            PageId::Collections => crate::pages::collections::build(&shared_config).upcast(),
            PageId::Emoji => crate::pages::emoji::build().upcast(),
            PageId::Symbols => crate::pages::symbols::build().upcast(),
            PageId::Kaomoji => crate::pages::kaomoji::build().upcast(),
            PageId::Snippets => crate::pages::snippets::build(&shared_config).upcast(),
            PageId::Settings => crate::pages::settings::build(&shared_config).upcast(),
        };

        let page_widget = if matches!(page_id, PageId::Clipboard | PageId::Settings) {
            content_widget
        } else {
            gtk4::ScrolledWindow::builder()
                .hscrollbar_policy(gtk4::PolicyType::Never)
                .vscrollbar_policy(gtk4::PolicyType::Automatic)
                .vexpand(true)
                .hexpand(true)
                .child(&content_widget)
                .build()
                .upcast()
        };

        page_widgets.push((page_id, page_widget));
    }

    // ── Stack for page content ────────────────────────────────
    let stack = gtk4::Stack::builder()
        .vexpand(true)
        .hexpand(true)
        .transition_type(gtk4::StackTransitionType::Crossfade)
        .build();

    for (_, widget) in &page_widgets {
        stack.add_child(widget);
    }

    // Show first page.
    if let Some((_, first_widget)) = page_widgets.first() {
        stack.set_visible_child(first_widget);
    }

    // Sidebar navigation.
    let stack_clone = stack.clone();
    let page_widgets_clone = page_widgets.clone();
    let state_for_nav = state.clone();
    sidebar_list.connect_row_selected(move |_list, row| {
        let Some(row) = row else { return };
        // Recover the page ID from the child widget's name.
        let page_tag = row
            .child()
            .and_then(|c| {
                let name = c.widget_name();
                if name.is_empty() {
                    None
                } else {
                    Some(name)
                }
            })
            .unwrap_or_default();
        if page_tag.is_empty() {
            return;
        }
        if let Ok(page_id) = page_tag.parse::<PageId>() {
            if let Some(idx) = page_widgets_clone
                .iter()
                .position(|(pid, _)| pid == &page_id)
            {
                if let Some((_, widget)) = page_widgets_clone.get(idx) {
                    stack_clone.set_visible_child(widget);
                    let mut s = state_for_nav.borrow_mut();
                    let effects = crate::app::reduce(&mut s, Action::PageChanged(page_id));
                    drop(s);
                    for eff in effects {
                        if let Effect::PersistGSettings = eff {
                            if let Some(ref settings) = Settings::new() {
                                settings.set_last_page(page_id);
                            }
                        }
                    }
                }
            }
        }
    });

    // Select initial page from GSettings.
    if let Some(ref settings) = settings {
        let last_page = settings.last_page();
        if let Some(idx) = page_widgets.iter().position(|(pid, _)| pid == &last_page) {
            if let Some(row) = sidebar_list.row_at_index(i32::try_from(idx).unwrap_or(0)) {
                sidebar_list.select_row(Some(&row));
            }
        }
    }

    // ── Status bar ────────────────────────────────────────────
    let status_label = gtk4::Label::new(Some("● Daemon"));
    status_label.set_margin_top(4);
    status_label.set_margin_bottom(4);
    status_label.set_halign(gtk4::Align::Start);
    status_label.set_margin_start(12);

    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    status_box.append(&status_label);
    status_box.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
    status_box.append(&gtk4::Label::new(Some("Tip: / focuses search in popups")));

    // Update status bar from state.
    let status_for_update = status_label.clone();
    let state_for_status = state.clone();
    {
        let s = state_for_status.borrow();
        let items_count = s.items.len();
        let pinned_count = s.items.iter().filter(|i| i.pinned).count();
        let daemon = if s.daemon_running { "●" } else { "○" };
        status_for_update.set_label(&format!(
            "{daemon} Daemon · {items_count} items · {pinned_count} pinned"
        ));
    }

    // ── Content layout ────────────────────────────────────────
    let content_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content_vbox.append(&stack);
    content_vbox.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    content_vbox.append(&status_box);

    // ── Sidebar + content in OverlaySplitView ─────────────────
    let split_view = adw::OverlaySplitView::new();
    split_view.set_sidebar(Some(&sidebar_list));
    split_view.set_content(Some(&content_vbox));
    split_view.set_show_sidebar(true);
    split_view.set_collapsed(false);
    split_view.set_min_sidebar_width(180.0);
    split_view.set_max_sidebar_width(250.0);
    split_view.set_enable_show_gesture(true);

    // Toggle sidebar at 900px breakpoint via periodic check (GTK4
    // notify::width is unreliable for this use case).
    let split_for_resize = split_view.clone();
    let window_for_sidebar = window.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        let width = window_for_sidebar.width();
        split_for_resize.set_show_sidebar(width > 900);
        glib::ControlFlow::Continue
    });

    // ── Toolbar view ──────────────────────────────────────────
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&split_view.upcast::<gtk4::Widget>()));

    // ── Toast overlay ─────────────────────────────────────────
    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&toolbar));
    window.set_content(Some(&toast_overlay));

    // ── Global key controller ─────────────────────────────────
    let (tx, rx) = std::sync::mpsc::channel::<Effect>();
    crate::controller::key::install(&window, &state, &tx);

    // Handle effects from the channel on the GTK thread.
    let toast_overlay_for_rx = toast_overlay.clone();
    glib::idle_add_local(move || {
        while let Ok(eff) = rx.try_recv() {
            match eff {
                Effect::AddToast(ref msg) => {
                    toast_overlay_for_rx.add_toast(adw::Toast::new(msg));
                }
                Effect::Quit => {
                    if let Some(w) = toast_overlay_for_rx
                        .root()
                        .and_then(|r| r.downcast::<adw::ApplicationWindow>().ok())
                    {
                        w.close();
                    }
                }
                _ => {}
            }
        }
        glib::ControlFlow::Continue
    });

    // ── Size persistence ──────────────────────────────────────
    let settings_for_close = settings.clone();
    // Debounced size persistence: write every 500ms while the window is alive.
    let window_for_persist = window.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        if let Some(ref s) = settings_for_close {
            let (w, h) = (window_for_persist.width(), window_for_persist.height());
            s.set_window_size(w, h);
        }
        glib::ControlFlow::Continue
    });

    window.present();
    Ok(())
}
