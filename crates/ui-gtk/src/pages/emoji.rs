//! Searchable emoji picker page.

use gtk4::Widget;

use crate::widgets::picker_grid::{self, ExpressionItem, PickerPresentation};

/// Build the emoji picker page.
pub fn build() -> Widget {
    let categories: Vec<(String, String)> = author_clipboard_shared::emoji::CATEGORIES
        .iter()
        .map(|cat| (cat.name.to_string(), cat.icon.to_string()))
        .collect();
    let items = author_clipboard_shared::emoji::CATEGORIES
        .iter()
        .flat_map(|cat| {
            cat.emojis.iter().map(move |emoji| ExpressionItem {
                value: (*emoji).to_string(),
                description: cat.name.to_string(),
                category: cat.name.to_string(),
            })
        })
        .collect();
    picker_grid::build(
        PickerPresentation {
            title: "Emoji",
            kind: "emoji",
            wide_cells: false,
        },
        &categories,
        items,
    )
}
