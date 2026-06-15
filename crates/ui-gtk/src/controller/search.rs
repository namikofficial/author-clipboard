//! Debounced search controller. Populated in T011.

#![allow(dead_code, unused_imports)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// How long to wait after the last keystroke before firing the search.
pub const DEBOUNCE_MS: u64 = 150;

/// Search debounce state.
#[derive(Debug)]
pub struct SearchDebounce {
    /// The pending search query.
    pub pending: String,
    /// When the last keystroke occurred.
    pub last_change: Instant,
}

impl Default for SearchDebounce {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchDebounce {
    /// Create a new debounce state.
    pub fn new() -> Self {
        Self {
            pending: String::new(),
            last_change: Instant::now(),
        }
    }
}

/// Factor out the debounce logic so it can be tested without `GLib` timers.
pub fn debounce_apply(pending: &str, now: Instant, last_change: Instant) -> Option<String> {
    // The `now` parameter is the reference point; `last_change` is when the
    // pending query was last updated. We fire only after `DEBOUNCE_MS` has
    // passed since the last change.
    let elapsed = now.duration_since(last_change);
    if elapsed.as_millis() >= u128::from(DEBOUNCE_MS) {
        Some(pending.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn debounce_apply_fires_after_debounce_ms() {
        let start = Instant::now();
        let after = start + Duration::from_millis(DEBOUNCE_MS + 10);
        assert!(debounce_apply("query", after, start).is_some());
    }

    #[test]
    fn debounce_apply_drops_before_debounce_ms() {
        let start = Instant::now();
        let early = start + Duration::from_millis(DEBOUNCE_MS - 50);
        assert!(debounce_apply("query", early, start).is_none());
    }
}
