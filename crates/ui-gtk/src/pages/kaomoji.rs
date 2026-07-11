//! Searchable kaomoji picker page.

use gtk4::Widget;

use crate::widgets::picker_grid::{self, ExpressionItem, PickerPresentation};

/// Build the kaomoji picker page.
pub fn build() -> Widget {
    let categories: Vec<(String, String)> = author_clipboard_shared::kaomoji::CATEGORIES
        .iter()
        .map(|cat| (cat.name.to_string(), cat.icon.to_string()))
        .collect();
    let items = author_clipboard_shared::kaomoji::CATEGORIES
        .iter()
        .flat_map(|cat| {
            cat.items.iter().map(move |value| ExpressionItem {
                value: (*value).to_string(),
                description: cat.name.to_string(),
                category: cat.name.to_string(),
            })
        })
        .collect();
    picker_grid::build(
        PickerPresentation {
            title: "Kaomoji",
            kind: "kaomoji",
            wide_cells: true,
        },
        &categories,
        items,
    )
}
