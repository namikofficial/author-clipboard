//! Pure state, no GTK deps. Populated in T006.

#![allow(dead_code, unused_imports)]

use crate::PickerFilter;

/// What the global key controller is asking the runtime to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// User pressed Esc. Resolve via [`crate::controller::focus::resolve_escape`].
    Escape,
    /// Focus the search entry.
    FocusSearch,
    /// Move selection up.
    MoveUp,
    /// Move selection down.
    MoveDown,
    /// Move selection left.
    MoveLeft,
    /// Move selection right.
    MoveRight,
    /// Jump to first item.
    First,
    /// Jump to last item.
    Last,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Confirm the current selection.
    Enter,
    /// Show the keyboard shortcuts overlay.
    ShowShortcuts,
}

/// Filter chip + sort order.
#[derive(Debug, Clone)]
pub struct FilterState {
    /// Active filter chip.
    pub filter: PickerFilter,
    /// Active sort order.
    pub sort: SortOrder,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            filter: PickerFilter::All,
            sort: SortOrder::NewestFirst,
        }
    }
}

/// Sort order for the list. `MostUsed` is reserved for a future
/// release (T009 will read it from the `usage_count` field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Most recent first. Default.
    NewestFirst,
    /// Oldest first.
    OldestFirst,
    /// Most frequently used first. (Not yet implemented.)
    MostUsed,
}

impl SortOrder {
    /// Short label for chip text.
    pub fn label(self) -> &'static str {
        match self {
            Self::NewestFirst => "Newest",
            Self::OldestFirst => "Oldest",
            Self::MostUsed => "Most used",
        }
    }
}
