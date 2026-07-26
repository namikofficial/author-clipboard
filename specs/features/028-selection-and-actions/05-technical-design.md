# Technical Design: Unify GTK Selection, Actions, and Visible State

`AppState.items` is the current visible snapshot and `selected_id` is the
sole persisted selection. Position is derived locally from the snapshot only
for clamped keyboard movement and deletion fallback. `ItemsLoaded` reconciles
the old ID against the new snapshot and selects the first item for an initial
non-empty snapshot.

`ItemRow` stores its database ID on the GTK row. ListBox callbacks dispatch
that ID directly. The command layer resolves `selected_item()` at dispatch
time; action availability is derived from that item, including protected
reveal/snippet restrictions and pin/star labels.

Affected files: `crates/ui-gtk/src/app.rs`, `pages/clipboard.rs`,
`widgets/item_row.rs`, `widgets/preview.rs`, `widgets/action_bar.rs`,
`window/popup.rs`, and associated tests/docs.
