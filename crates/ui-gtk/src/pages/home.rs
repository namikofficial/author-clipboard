//! Manager home dashboard with local runtime, privacy, and history status.

use author_clipboard_shared::config::Config;
use author_clipboard_shared::Database;
use gtk4::prelude::*;

/// Build the manager's functional home dashboard.
pub fn build(config: &Config) -> gtk4::Box {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    page.set_margin_top(32);
    page.set_margin_bottom(32);
    page.set_margin_start(32);
    page.set_margin_end(32);
    let title = gtk4::Label::new(Some("Author Clipboard"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);
    page.append(&title);
    let subtitle = gtk4::Label::new(Some("Your private Wayland clipboard command center"));
    subtitle.add_css_class("dim-label");
    subtitle.set_halign(gtk4::Align::Start);
    page.append(&subtitle);

    let cards = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(3)
        .min_children_per_line(1)
        .column_spacing(12)
        .row_spacing(12)
        .build();
    let stats = Database::open(&config.db_path())
        .and_then(|db| db.get_stats())
        .ok();
    cards.insert(
        &status_card(
            "History",
            &stats.as_ref().map_or_else(
                || "Unavailable".into(),
                |s| format!("{} local items", s.total_items),
            ),
        ),
        -1,
    );
    cards.insert(
        &status_card(
            "Privacy",
            if config.encrypt_sensitive {
                "Sensitive encryption enabled"
            } else {
                "Sensitive encryption disabled"
            },
        ),
        -1,
    );
    cards.insert(&status_card("Shortcut", &config.keyboard_shortcut), -1);
    page.append(&cards);

    let hint = gtk4::Label::new(Some(
        "Use the sidebar to browse history, collections, expressions, snippets, and settings.",
    ));
    hint.set_wrap(true);
    hint.set_halign(gtk4::Align::Start);
    hint.add_css_class("dim-label");
    page.append(&hint);
    page
}

fn status_card(title: &str, value: &str) -> gtk4::Box {
    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    card.add_css_class("home-status-card");
    card.set_margin_top(12);
    card.set_margin_bottom(12);
    card.set_margin_start(12);
    card.set_margin_end(12);
    let heading = gtk4::Label::new(Some(title));
    heading.add_css_class("heading");
    heading.set_halign(gtk4::Align::Start);
    let value = gtk4::Label::new(Some(value));
    value.set_wrap(true);
    value.set_halign(gtk4::Align::Start);
    card.append(&heading);
    card.append(&value);
    card
}
