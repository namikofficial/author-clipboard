//! Design tokens, AdwStyleManager. Populated in T005.

#![allow(dead_code, unused_imports)]

use libadwaita as adw;
use libadwaita::prelude::*;

/// Apply the cute/branded theme to the default [`adw::StyleManager`].
pub fn apply() {
    let manager = adw::StyleManager::default();
    manager.set_color_scheme(adw::ColorScheme::Default);
}
