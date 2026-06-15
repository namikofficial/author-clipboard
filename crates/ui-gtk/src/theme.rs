//! Design tokens, `AdwStyleManager`, and Rust-side constants
//! for the author-clipboard UI.
//!
//! The CSS tokens live in `data/style.css` and are the source of
//! truth. This module exposes the same scale as Rust constants so
//! that widgets built in code (paddings, margins, gaps) can refer
//! to the same values without hardcoding pixels.
//!
//! Naming follows the convention used in `style.css`:
//!   - `Spacing::*` ↔ `--space-*`
//!   - `Radius::*` ↔ `--radius-*`
//!   - `Motion::*` ↔ `--motion-*`
//!   - `Focus::*` ↔ `--focus-ring-*`

#![allow(dead_code, unused_imports, clippy::doc_markdown)]

use libadwaita as adw;
use libadwaita::prelude::*;

/// Spacing scale in logical pixels. Mirrors `--space-*` in CSS.
///
/// Use these constants when calling GTK-side spacing APIs
/// (`set_margin_top`, `set_spacing`, `set_size_request`, etc.)
/// so that paddings stay aligned with the CSS scale.
pub mod spacing {
    /// 2px — hairline gap, e.g. between a row's title and subtitle.
    pub const SPACE_2XS: i32 = 2;
    /// 4px — half-step, used for tight icon gaps and pill padding.
    pub const SPACE_XS: i32 = 4;
    /// 6px — small chip vertical padding.
    pub const SPACE_SM: i32 = 6;
    /// 8px — base spacing, the most common gap between siblings.
    pub const SPACE_MD: i32 = 8;
    /// 12px — list row vertical padding, sidebar row gap.
    pub const SPACE_LG: i32 = 12;
    /// 16px — content area horizontal padding, list row horizontal.
    pub const SPACE_XL: i32 = 16;
    /// 24px — section break, between major regions.
    pub const SPACE_2XL: i32 = 24;
}

/// Corner radius scale in logical pixels. Mirrors `--radius-*`
/// in CSS.
pub mod radius {
    /// 6px — small chips, small buttons.
    pub const RADIUS_SM: i32 = 6;
    /// 12px — item rows, cards.
    pub const RADIUS_MD: i32 = 12;
    /// 16px — large cards, surfaces.
    pub const RADIUS_LG: i32 = 16;
    /// 999px — pill-shaped chips, search entry.
    pub const RADIUS_PILL: i32 = 999;
}

/// Motion durations in milliseconds. Mirrors `--motion-*` in CSS.
///
/// Use these for `glib::timeout_add_local` or any code that
/// drives a CSS transition manually.
pub mod motion {
    /// 120ms — hover, chip toggle, list selection.
    pub const FAST_MS: u64 = 120;
    /// 200ms — modal, toast, mid-weight state change.
    pub const BASE_MS: u64 = 200;
    /// 320ms — page transition, drawer slide.
    pub const SLOW_MS: u64 = 320;
}

/// Focus ring dimensions. Mirrors `--focus-ring-*` in CSS.
pub mod focus {
    /// 2px — focus ring stroke width.
    pub const RING_WIDTH: i32 = 2;
    /// 2px — focus ring offset from the widget edge.
    pub const RING_OFFSET: i32 = 2;
}

/// Typography helpers for widget labels.
///
/// The font-size values are kept in sync with the `font-size`
/// declarations on `.item-title`, `.item-subtitle`, and `.chip`
/// in `style.css`.
pub mod font_size {
    /// 11px — chip labels and metadata text.
    pub const CHIP_PX: i32 = 11;
    /// 11px — item-row subtitle (alias of `CHIP_PX`).
    pub const SUBTITLE_PX: i32 = 11;
    /// 13px — item-row primary title.
    pub const TITLE_PX: i32 = 13;
    /// 16px — emoji / symbol / kaomoji picker cell.
    pub const PICKER_CELL_PX: i32 = 16;
}

/// Apply the cute/branded theme to the default [`adw::StyleManager`].
///
/// This is intentionally tiny: libadwaita owns the system colour
/// scheme, and the custom `style.css` is loaded automatically via
/// the GResource bundle. We only need to express the app's
/// light/dark preference here.
pub fn apply() {
    let manager = adw::StyleManager::default();
    manager.set_color_scheme(adw::ColorScheme::Default);
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[test]
    fn spacing_scale_is_monotonic() {
        // The 4px-based scale must never regress.
        let scale = [
            spacing::SPACE_2XS,
            spacing::SPACE_XS,
            spacing::SPACE_SM,
            spacing::SPACE_MD,
            spacing::SPACE_LG,
            spacing::SPACE_XL,
            spacing::SPACE_2XL,
        ];
        for w in scale.windows(2) {
            assert!(w[0] < w[1], "spacing scale regressed: {} >= {}", w[0], w[1]);
        }
    }

    #[test]
    fn radius_sm_le_radius_md_le_radius_lg() {
        assert!(radius::RADIUS_SM < radius::RADIUS_MD);
        assert!(radius::RADIUS_MD < radius::RADIUS_LG);
    }

    #[test]
    fn pill_is_largest() {
        assert!(radius::RADIUS_PILL >= radius::RADIUS_LG);
    }

    #[test]
    fn motion_scale_is_monotonic() {
        assert!(motion::FAST_MS < motion::BASE_MS);
        assert!(motion::BASE_MS < motion::SLOW_MS);
    }

    #[test]
    fn focus_ring_is_2px() {
        // Documented in design tokens: the focus ring is 2px
        // wide. If this changes, update both the constants and
        // the CSS token.
        assert_eq!(focus::RING_WIDTH, 2);
        assert_eq!(focus::RING_OFFSET, 2);
    }

    #[test]
    fn subtitle_le_title() {
        // Subtitle is always ≤ title font-size so the visual
        // hierarchy stays intact.
        assert!(font_size::SUBTITLE_PX <= font_size::TITLE_PX);
        assert!(font_size::CHIP_PX <= font_size::SUBTITLE_PX);
    }
}
