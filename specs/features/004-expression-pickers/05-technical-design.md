# Technical Design: Expression Pickers

Build all three pages through a shared GTK expression-picker component. It owns
search/category filtering, activation, clipboard feedback, and recent-use loading.
Filtering rules live in pure functions so they can be tested without GTK setup.

## Affected Files

- `crates/ui-gtk/src/widgets/picker_grid.rs`
- `crates/ui-gtk/src/pages/emoji.rs`
- `crates/ui-gtk/src/pages/symbols.rs`
- `crates/ui-gtk/src/pages/kaomoji.rs`
- `crates/ui-gtk/data/style.css`
- `specs/features/004-expression-pickers/*`
