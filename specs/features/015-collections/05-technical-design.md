# Technical Design: Collections UI

## Scope

Phase 21 completes the native GTK manager surface for the collection storage
and IPC delivered by feature 027 T010-T011. The UI reads and mutates the same
SQLite-backed collection model; it does not maintain UI-only collections.

## Affected Files

| File | Change |
|---|---|
| `crates/ui-gtk/src/pages/collections.rs` | Collection manager, item-count badges, contents and membership actions |
| `crates/ui-gtk/src/pages/clipboard.rs` | Ctrl+Shift+C collection chooser for the selected history item |
| `crates/ui-gtk/src/pages/mod.rs` | Export the page |
| `crates/ui-gtk/src/app.rs` | Add the persistent Collections navigation page ID |
| `crates/ui-gtk/src/window/manager.rs` | Add Collections to the manager sidebar and stack |

## Interaction Design

- The sidebar exposes a dedicated **Collections** destination.
- The left pane lists every collection with its item count as a badge.
- Selecting a collection loads its clipboard items in the right pane.
- The toolbar creates, renames, and deletes collections.
- Each content row can be removed from the selected collection without
  deleting the underlying clipboard item.
- Ctrl+Shift+C on a selected Clipboard row opens a modal chooser; activating a
  collection adds that item and closes the chooser.
- Mutations refresh both panes immediately. Failures are shown inline and do
  not optimistically alter the visible model.

## Data Flow

The page opens `Config::db_path()` through `Database`. Reads use
`list_collections` and `get_collection_items`; mutations use the existing
collection helpers. The daemon and CLI continue to use the equivalent IPC
commands. SQLite foreign keys preserve item/collection lifecycle semantics.

## Decisions

- Counts are derived from `get_collection_items` because the current
  `Collection` contract intentionally contains no denormalized count.
- Collection deletion requires a destructive modal confirmation; deleting it
  never deletes the underlying clipboard items.
- Empty/whitespace-only names are rejected before reaching SQLite.
