# Selection and actions

The GTK UI treats `AppState.selected_id` as the sole stored clipboard
selection. `AppState.items` is the current visible database snapshot. A row's
position is transient: it may be used to calculate a keyboard delta, but it is
never persisted or used to identify an item.

## Reconciliation rules

When `ItemsLoaded` replaces the snapshot:

- a selected ID remains selected if it is still visible;
- an existing selected ID that is filtered out is cleared;
- an initial non-empty snapshot with no selection selects its first item;
- an empty snapshot clears selection.

Deletion removes by ID and selects the item now occupying the deleted
position, or the previous item when the deleted item was last. Pin and star
reducers mutate the matching visible row immediately, before IPC effects are
handled.

## GTK command flow

`ItemRow` stores its database ID on the `GtkListBoxRow`. List selection,
activation, Ctrl+Enter, and the collection shortcut resolve that ID directly.
The preview uses `AppState::selected_item()`, and contextual commands resolve
the same item at dispatch time. Typed commands are represented by
`SelectedItemCommand`; availability is derived from the selected item, with
protected items restricted from snippet creation and eligible for reveal.

The action rail includes copy, quick paste, plain text, transform, snippet,
collection, reveal, pin, star, and delete. Collection selection remains a
follow-up dialog, but receives the stable selected item ID.

## Invariants

1. `selected_id == Some(id)` implies exactly one visible item has that ID.
2. Preview, keyboard activation, and action dispatch resolve the same selected
   item from the current snapshot.
3. No selection means an empty preview and no actionable contextual command.
4. Reordering visible rows cannot retarget an action.

## Verification

The reducer and GTK crate tests cover ID validation, snapshot reconciliation,
movement, deletion fallback, mutation visibility, preview lookup, and row
mapping. The targeted command is:

```text
cargo test -p author-clipboard-ui-gtk
```
