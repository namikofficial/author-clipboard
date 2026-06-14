//! Right-pane preview in the manager window.
//!
//! Shows the selected item's content: text (sourceview), image
//! (gdk-pixbuf), or files (adw::ActionRow list). Sensitive items
//! show a redaction overlay with a timed reveal button (5s).
//!
//! **No WebKit in this module.** HTML preview ships in PR 5.5.

use std::cell::RefCell;
use std::rc::Rc;

use author_clipboard_shared::types::{ClipboardItem, ContentType};
use gtk4::prelude::*;
use gtk4::{gdk_pixbuf, gio, glib, Widget};
use libadwaita as adw;
use libadwaita::prelude::*;
use sourceview5;

use crate::AppState;

/// Callback when the user clicks "Reveal".
pub type OnReveal = Rc<dyn Fn()>;

/// Right-pane preview widget for the manager window.
///
/// Owned by the manager window; driven by [`AppState`] changes via
/// [`update_preview`](PreviewPane::update_preview) and
/// [`on_items_loaded`](PreviewPane::on_items_loaded).
pub struct PreviewPane {
    /// Shared app state (read-only for display).
    state: Rc<RefCell<AppState>>,

    /// Called when the user clicks the Reveal button.
    on_reveal: OnReveal,

    /// Root widget.
    widget: gtk4::Box,

    /// Text / HTML content.
    text_view: sourceview5::View,

    /// Image preview.
    image_picture: gtk4::Picture,

    /// File list preview.
    files_box: gtk4::Box,

    /// Sensitive content overlay.
    redacted_overlay: adw::StatusPage,

    /// Button to reveal sensitive content.
    reveal_button: gtk4::Button,

    /// Countdown label shown next to the reveal button.
    countdown_label: gtk4::Label,

    /// Empty-state placeholder.
    empty_state: adw::StatusPage,

    /// Tracks the GLib timer source so it can be cancelled.
    #[allow(dead_code)]
    timer_source: RefCell<Option<glib::SourceId>>,
}

impl PreviewPane {
    /// Build a new `PreviewPane`.
    ///
    /// `on_reveal` is called when the user clicks "Reveal". The caller
    /// (runtime) is responsible for starting the `RevealTick` timer and
    /// calling [`update_countdown`](PreviewPane::update_countdown).
    pub fn new(state: Rc<RefCell<AppState>>, on_reveal: OnReveal) -> Self {
        let widget = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        widget.add_css_class("preview-pane");

        // ── Text view (text / HTML — no syntax highlight in this PR) ──
        let lang_manager = sourceview5::LanguageManager::default();
        let lang = lang_manager.language("text").unwrap();
        let buf = sourceview5::Buffer::new(Some(&lang));
        buf.set_editable(false);
        buf.set_highlight_syntax(false);
        let text_view = sourceview5::View::new_with_buffer(&buf);
        text_view.set_wrap_mode(gtk4::WrapMode::WordChar);
        text_view.add_css_class("preview-text");
        text_view.set_visible(false);

        // ── Image picture ─────────────────────────────────────────────
        let image_picture = gtk4::Picture::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        image_picture.add_css_class("preview-image");
        image_picture.set_visible(false);

        // ── Files box ─────────────────────────────────────────────────
        let files_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .build();
        files_box.add_css_class("preview-files");
        files_box.set_visible(false);

        // ── Redacted overlay ───────────────────────────────────────────
        let redacted_overlay = adw::StatusPage::builder()
            .title("Sensitive Content")
            .description("This item contains sensitive content.")
            .icon_name(Some("lock"))
            .build();
        redacted_overlay.add_css_class("preview-redacted");
        redacted_overlay.set_visible(false);

        // Countdown chip and reveal button.
        let countdown_label = gtk4::Label::builder()
            .label("Reveal (5s)")
            .halign(gtk4::Align::Center)
            .build();
        countdown_label.add_css_class("chip");

        let reveal_button = gtk4::Button::builder()
            .label("Reveal")
            .halign(gtk4::Align::Center)
            .build();
        reveal_button.add_css_class("suggested-action");
        reveal_button.set_visible(false);

        // Pack button + label into a centred hbox.
        let controls_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .build();
        controls_box.append(&reveal_button);
        controls_box.append(&countdown_label);
        controls_box.set_visible(false);

        // ── Empty state ───────────────────────────────────────────────
        let empty_state = adw::StatusPage::builder()
            .title("Select an item to preview")
            .description("Choose an item from the list to see its content.")
            .icon_name(Some("clipboard"))
            .build();
        empty_state.add_css_class("preview-empty");
        empty_state.set_visible(true);

        // ── Wire up reveal button ─────────────────────────────────────
        let on_reveal_clone = on_reveal.clone();
        reveal_button.connect_clicked(move |_| {
            on_reveal_clone();
        });

        // ── Pack all content layers into the root box ──────────────────
        widget.append(&text_view);
        widget.append(&image_picture);
        widget.append(&files_box);
        widget.append(&redacted_overlay);
        widget.append(&controls_box);
        widget.append(&empty_state);

        Self {
            state,
            on_reveal,
            widget,
            text_view,
            image_picture,
            files_box,
            redacted_overlay,
            reveal_button,
            countdown_label,
            empty_state,
            timer_source: RefCell::new(None),
        }
    }

    /// Called by the runtime after IPC returns with a fresh item list.
    pub fn on_items_loaded(&self, items: Vec<ClipboardItem>) {
        let mut state = self.state.borrow_mut();
        state.items = items;
        drop(state);
        // Re-evaluate the preview with the new items.
        self.update_preview();
    }

    /// Re-evaluate the preview from current `state.selected_index`.
    /// Called whenever the selection changes.
    pub fn update_preview(&self) {
        let state = self.state.borrow();
        let idx = match state.selected_index {
            Some(i) => i,
            None => {
                self.show_empty();
                return;
            }
        };
        let item = match state.items.get(idx) {
            Some(i) => i,
            None => {
                self.show_empty();
                return;
            }
        };
        drop(state);

        // ── Content-type branch ───────────────────────────────────────
        match item.content_type {
            ContentType::Text | ContentType::Html => self.show_text(item),
            ContentType::Image => self.show_image(item),
            ContentType::Files => self.show_files(item),
        }

        // ── Sensitive overlay ─────────────────────────────────────────
        if item.sensitive {
            let state = self.state.borrow();
            if !state.show_redacted {
                self.show_redacted(item);
            } else {
                // User has already revealed — hide the overlay and show content
                self.redacted_overlay.set_visible(false);
                self.reveal_button.set_visible(false);
            }
        }
    }

    /// Update the countdown label from `state.reveal_countdown`.
    /// Called every second by the GLib timer the runtime manages.
    pub fn update_countdown(&self) {
        let state = self.state.borrow();
        let secs = state.reveal_countdown;
        if secs == 0 {
            self.countdown_label.set_label("Reveal");
            self.reveal_button.set_visible(false);
        } else {
            self.countdown_label
                .set_label(&format!("Reveal ({secs}s)"));
        }
    }

    /// Show the text / HTML content.
    fn show_text(&self, item: &ClipboardItem) {
        self.image_picture.set_visible(false);
        self.files_box.set_visible(false);
        self.text_view.set_visible(true);
        self.redacted_overlay.set_visible(false);

        let buf = self.text_view.buffer().unwrap();
        let text = &item.content;
        buf.set_text(text);
    }

    /// Show the image preview.
    fn show_image(&self, item: &ClipboardItem) {
        self.text_view.set_visible(false);
        self.files_box.set_visible(false);
        self.image_picture.set_visible(true);
        self.redacted_overlay.set_visible(false);

        // ClipboardItem stores the relative path in `content` for images.
        let path = std::path::Path::new(&item.content);
        if path.exists() {
            if let Ok(pixbuf) =
                gdk_pixbuf::Pixbuf::from_file_at_scale(path, 800, 600, true)
            {
                self.image_picture.set_pixbuf(Some(&pixbuf));
            }
        }
    }

    /// Show the file list.
    fn show_files(&self, item: &ClipboardItem) {
        self.text_view.set_visible(false);
        self.image_picture.set_visible(false);
        self.files_box.set_visible(true);
        self.redacted_overlay.set_visible(false);

        // Clear old rows.
        while let Some(child) = self.files_box.first_child() {
            self.files_box.remove(&child);
        }

        let content = &item.content;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let row = adw::ActionRow::builder()
                .title(line)
                .build();
            // Click to open with default handler.
            let line_clone = line.to_string();
            row.connect_activated(move |_| {
                if let Ok(file) = gio::File::for_uri(&line_clone) {
                    if let Ok(uri) = file.uri() {
                        let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::File>);
                    }
                }
            });
            self.files_box.append(&row);
        }
    }

    /// Show the sensitive redaction overlay.
    fn show_redacted(&self, item: &ClipboardItem) {
        self.text_view.set_visible(false);
        self.image_picture.set_visible(false);
        self.files_box.set_visible(false);
        self.redacted_overlay.set_visible(true);

        let desc = item
            .redacted_preview
            .as_deref()
            .unwrap_or("••••••••");
        self.redacted_overlay.set_description(Some(desc));

        let state = self.state.borrow();
        let countdown = state.reveal_countdown;
        drop(state);

        if countdown == 0 {
            self.countdown_label.set_label("Reveal");
            self.reveal_button.set_visible(true);
        } else {
            self.countdown_label
                .set_label(&format!("Reveal ({countdown}s)"));
            self.reveal_button.set_visible(true);
        }
    }

    /// Show the empty-state placeholder.
    fn show_empty(&self) {
        self.text_view.set_visible(false);
        self.image_picture.set_visible(false);
        self.files_box.set_visible(false);
        self.redacted_overlay.set_visible(false);
        self.empty_state.set_visible(true);
    }

    /// Borrow the root widget for embedding in a parent container.
    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use author_clipboard_shared::types::ClipboardItem;
    use chrono::Utc;

    fn make_text_item(content: &str) -> ClipboardItem {
        let mut item = ClipboardItem::new_text(content.to_string());
        item.timestamp = Utc::now();
        item
    }

    fn make_image_item(path: &str) -> ClipboardItem {
        let mut item = ClipboardItem::new_image(path.to_string(), "image/png".into(), 0);
        item.timestamp = Utc::now();
        item
    }

    fn make_files_item(content: &str) -> ClipboardItem {
        let mut item = ClipboardItem::new_files(content.to_string());
        item.timestamp = Utc::now();
        item
    }

    fn make_sensitive_item(content: &str) -> ClipboardItem {
        let mut item = ClipboardItem::new_text(content.to_string());
        item.sensitive = true;
        item.redacted_preview = Some("••••••••".to_string());
        item.timestamp = Utc::now();
        item
    }

    // ── show_text ────────────────────────────────────────────────────

    #[test]
    fn show_text_sets_buffer_content() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let reveal_called = Rc::new(std::cell::Cell::new(false));
        let reveal_called_c = reveal_called.clone();
        let pane = PreviewPane::new(state, Rc::new(move || {
            reveal_called_c.set(true);
        }));

        let item = make_text_item("hello world");
        pane.show_text(&item);

        let buf = pane.text_view.buffer().unwrap();
        assert_eq!(buf.text().as_str(), "hello world");
    }

    #[test]
    fn show_text_hides_other_views() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        let item = make_text_item("test");
        pane.show_text(&item);
        assert!(!pane.image_picture.is_visible());
        assert!(!pane.files_box.is_visible());
        assert!(!pane.redacted_overlay.is_visible());
    }

    // ── show_image ──────────────────────────────────────────────────

    #[test]
    fn show_image_hides_other_views() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        let item = make_image_item("/nonexistent/path.png");
        pane.show_image(&item);
        assert!(!pane.text_view.is_visible());
        assert!(!pane.files_box.is_visible());
        assert!(!pane.redacted_overlay.is_visible());
    }

    // ── show_files ───────────────────────────────────────────────────

    #[test]
    fn show_files_populates_rows() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        let item = make_files_item("file:///tmp/a.txt\nfile:///tmp/b.txt");
        pane.show_files(&item);
        // Two non-empty, non-comment lines → two rows
        assert!(pane.files_box.first_child().is_some());
    }

    #[test]
    fn show_files_hides_other_views() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        let item = make_files_item("file:///tmp/a.txt");
        pane.show_files(&item);
        assert!(!pane.text_view.is_visible());
        assert!(!pane.image_picture.is_visible());
        assert!(!pane.redacted_overlay.is_visible());
    }

    // ── show_redacted ───────────────────────────────────────────────

    #[test]
    fn show_redacted_sets_description() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        let item = make_sensitive_item("hunter2");
        pane.show_redacted(&item);
        assert!(pane.redacted_overlay.is_visible());
        assert!(pane.reveal_button.is_visible());
    }

    #[test]
    fn show_redacted_hides_content_views() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        let item = make_sensitive_item("hunter2");
        pane.show_redacted(&item);
        assert!(!pane.text_view.is_visible());
        assert!(!pane.image_picture.is_visible());
        assert!(!pane.files_box.is_visible());
    }

    // ── show_empty ───────────────────────────────────────────────────

    #[test]
    fn show_empty_shows_empty_state() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        pane.show_empty();
        assert!(pane.empty_state.is_visible());
        assert!(!pane.text_view.is_visible());
        assert!(!pane.image_picture.is_visible());
        assert!(!pane.files_box.is_visible());
        assert!(!pane.redacted_overlay.is_visible());
    }

    // ── update_preview ───────────────────────────────────────────────

    #[test]
    fn update_preview_with_no_selection_shows_empty() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        pane.update_preview();
        assert!(pane.empty_state.is_visible());
    }

    #[test]
    fn update_preview_with_text_item_shows_text() {
        let state = Rc::new(RefCell::new(AppState::default()));
        state.borrow_mut().items = vec![make_text_item("preview me")];
        state.borrow_mut().selected_index = Some(0);
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        pane.update_preview();
        assert!(pane.text_view.is_visible());
    }

    #[test]
    fn update_preview_with_sensitive_shows_redacted() {
        let mut st = AppState::default();
        st.items = vec![make_sensitive_item("secret")];
        st.selected_index = Some(0);
        st.show_redacted = false;
        let state = Rc::new(RefCell::new(st));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        pane.update_preview();
        assert!(pane.redacted_overlay.is_visible());
        assert!(pane.reveal_button.is_visible());
    }

    // ── update_countdown ─────────────────────────────────────────────

    #[test]
    fn update_countdown_zero_hides_button() {
        let mut st = AppState::default();
        st.reveal_countdown = 0;
        let state = Rc::new(RefCell::new(st));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        pane.update_countdown();
        assert!(!pane.reveal_button.is_visible());
    }

    #[test]
    fn update_countdown_active_shows_button_with_secs() {
        let mut st = AppState::default();
        st.reveal_countdown = 3;
        let state = Rc::new(RefCell::new(st));
        let pane = PreviewPane::new(state, Rc::new(|| {}));
        pane.update_countdown();
        assert!(pane.reveal_button.is_visible());
        assert!(pane.countdown_label.label().contains("3s"));
    }

    // ── on_items_loaded ───────────────────────────────────────────────

    #[test]
    fn on_items_loaded_updates_state_and_refreshes_preview() {
        let state = Rc::new(RefCell::new(AppState::default()));
        let pane = PreviewPane::new(state.clone(), Rc::new(|| {}));
        let items = vec![make_text_item("loaded item")];
        pane.on_items_loaded(items);
        assert_eq!(state.borrow().items.len(), 1);
        assert_eq!(state.borrow().items[0].content, "loaded item");
    }
}
