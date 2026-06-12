//! Debounced search controller. Populated in T011.

#![allow(dead_code, unused_imports)]

/// How long to wait after the last keystroke before firing the search.
pub const DEBOUNCE_MS: u64 = 150;
