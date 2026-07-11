//! Searchable symbol picker page.

use gtk4::Widget;

use crate::widgets::picker_grid::{self, ExpressionItem, PickerPresentation};

/// Build the symbol picker page.
pub fn build() -> Widget {
    let categories: Vec<(String, String)> = author_clipboard_shared::symbols::CATEGORIES
        .iter()
        .map(|cat| (cat.name.to_string(), cat.icon.to_string()))
        .collect();
    let items = author_clipboard_shared::symbols::CATEGORIES
        .iter()
        .flat_map(|cat| {
            cat.symbols
                .iter()
                .map(move |(symbol, description)| ExpressionItem {
                    value: (*symbol).to_string(),
                    description: (*description).to_string(),
                    category: cat.name.to_string(),
                })
        })
        .collect();
    picker_grid::build(
        PickerPresentation {
            title: "Symbols",
            kind: "symbol",
            wide_cells: false,
        },
        &categories,
        items,
    )
}
