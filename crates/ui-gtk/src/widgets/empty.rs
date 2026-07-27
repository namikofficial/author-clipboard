//! Empty states using `AdwStatusPage` with a centered icon and
//! deliberate copy.
//!
//! The variant is selected up front by the caller, so the widget
//! itself is a thin wrapper around `AdwStatusPage` plus the
//! `.empty-state` CSS class for vertical breathing room.

use gtk4::prelude::*;
use gtk4::Widget;
use libadwaita as adw;
use libadwaita::prelude::*;

/// Reason there is nothing to show. Drives the icon + copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyVariant {
    /// The clipboard is empty (no items captured yet).
    NoItems,
    /// The user searched for something with no matches.
    NoResults,
    /// The user filtered to a category with no items.
    NoSensitive,
    /// The daemon isn't running, so the DB is empty.
    DaemonDown,
}

impl EmptyVariant {
    /// `title` for the status page.
    fn title(self) -> &'static str {
        match self {
            Self::NoItems => "Clipboard is empty",
            Self::NoResults => "No matching items",
            Self::NoSensitive => "No sensitive items",
            Self::DaemonDown => "Daemon is not running",
        }
    }

    /// `description` for the status page.
    fn description(self) -> &'static str {
        match self {
            Self::NoItems => "Copy something with Ctrl+C to see it here.",
            Self::NoResults => "Try a different search term or clear the filter.",
            Self::NoSensitive => "Items marked sensitive will appear here.",
            Self::DaemonDown => "Start the daemon to begin capturing clipboard history.",
        }
    }

    /// `icon_name` for the status page.
    fn icon_name(self) -> &'static str {
        match self {
            Self::NoItems => "edit-copy-symbolic",
            Self::NoResults => "system-search-symbolic",
            Self::NoSensitive => "channel-secure-symbolic",
            Self::DaemonDown => "dialog-error-symbolic",
        }
    }
}

/// An empty-state widget.
///
/// Built once per page; call [`set_variant`](EmptyState::set_variant)
/// to switch between copy / icon combos without rebuilding the
/// widget tree.
pub struct EmptyState {
    status_page: adw::StatusPage,
}

impl EmptyState {
    /// Build a new empty state. Defaults to
    /// [`EmptyVariant::NoItems`].
    pub fn new() -> Self {
        // The libadwaita builder methods take `&str` directly
        // (they wrap in `Option` internally); the live setters
        // want `Option<&str>` for the nullable ones. We
        // unwrap the static strings on the way in.
        let status_page = adw::StatusPage::builder()
            .title(EmptyVariant::NoItems.title())
            .description(EmptyVariant::NoItems.description())
            .icon_name(EmptyVariant::NoItems.icon_name())
            .vexpand(true)
            .hexpand(true)
            .build();
        status_page.add_css_class("empty-state");
        status_page.add_css_class("empty-state-no-items");
        Self { status_page }
    }

    /// Switch the variant. Cheap; just rewrites the three text
    /// properties on the inner `AdwStatusPage`.
    pub fn set_variant(&self, variant: EmptyVariant) {
        // libadwaita's StatusPage setters have a mix of
        // `&str` and `Option<&str>` signatures; pass the
        // shape each one wants.
        self.status_page.set_title(variant.title());
        self.status_page
            .set_description(Some(variant.description()));
        self.status_page.set_icon_name(Some(variant.icon_name()));
        // Drop the old variant class, add the new one so
        // CSS-level tweaks (color, weight) can target specific
        // variants if we want to later.
        for v in [
            "empty-state-no-items",
            "empty-state-no-results",
            "empty-state-no-sensitive",
            "empty-state-daemon-down",
        ] {
            self.status_page.remove_css_class(v);
        }
        self.status_page.add_css_class(match variant {
            EmptyVariant::NoItems => "empty-state-no-items",
            EmptyVariant::NoResults => "empty-state-no-results",
            EmptyVariant::NoSensitive => "empty-state-no-sensitive",
            EmptyVariant::DaemonDown => "empty-state-daemon-down",
        });
    }

    /// Switch to an error state with the given error message.
    ///
    /// This overrides whatever variant was set previously and shows
    /// a "Service error" title plus the real error text as a description.
    pub fn set_error(&self, error: &str) {
        self.status_page.set_title("Service error");
        self.status_page.set_description(Some(error));
        self.status_page.set_icon_name(Some("dialog-error-symbolic"));
        for v in [
            "empty-state-no-items",
            "empty-state-no-results",
            "empty-state-no-sensitive",
            "empty-state-daemon-down",
        ] {
            self.status_page.remove_css_class(v);
        }
        self.status_page.add_css_class("empty-state-error");
    }

    /// Borrow the root widget.
    pub fn widget(&self) -> &Widget {
        self.status_page.upcast_ref()
    }
}

impl Default for EmptyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_titles_are_distinct() {
        let titles = [
            EmptyVariant::NoItems.title(),
            EmptyVariant::NoResults.title(),
            EmptyVariant::NoSensitive.title(),
            EmptyVariant::DaemonDown.title(),
        ];
        // No two variants may share a title; that would collapse
        // them into one UI state and break accessibility (screen
        // readers would announce the same title for different
        // empty reasons).
        for (i, a) in titles.iter().enumerate() {
            for (j, b) in titles.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "variants share title: {a}");
                }
            }
        }
    }

    #[test]
    fn variant_descriptions_are_non_empty() {
        for v in [
            EmptyVariant::NoItems,
            EmptyVariant::NoResults,
            EmptyVariant::NoSensitive,
            EmptyVariant::DaemonDown,
        ] {
            assert!(!v.description().is_empty());
        }
    }

    #[test]
    fn variant_icons_are_non_empty() {
        for v in [
            EmptyVariant::NoItems,
            EmptyVariant::NoResults,
            EmptyVariant::NoSensitive,
            EmptyVariant::DaemonDown,
        ] {
            assert!(!v.icon_name().is_empty());
        }
    }
}
