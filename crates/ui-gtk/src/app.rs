//! Pure state, no GTK deps. Populated in T006.

#![allow(dead_code, unused_imports)]

/// What the global key controller is asking the runtime to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Escape,
    FocusSearch,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    First,
    Last,
    PageUp,
    PageDown,
    Enter,
    ShowShortcuts,
}

/// Filter chip + sort order. The real `PickerFilter` is added in T019;
/// for now this holds a `String` to keep the skeleton compiling.
#[derive(Debug, Clone)]
pub struct FilterState {
    pub filter: String,
    pub sort: SortOrder,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            filter: "all".to_string(),
            sort: SortOrder::NewestFirst,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    NewestFirst,
    OldestFirst,
    MostUsed,
}
