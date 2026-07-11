# Test Plan: Collections UI

## Automated

| Behavior | Verification |
|---|---|
| Names are trimmed and blank names rejected | collections unit tests |
| Collection rows retain stable IDs and accurate counts | collections unit tests |
| Collections page ID round-trips through settings text | app unit tests |
| GTK crate remains warning-free and builds | `cargo test -p author-clipboard-ui-gtk` |

## Manual

1. Open manager and select **Collections**.
2. Create two named collections and verify alphabetical order and zero badges.
3. Select a populated collection and verify its items and count.
4. Rename a collection and verify the sidebar refreshes.
5. Remove an item and verify it remains in Clipboard history.
6. Delete a collection, cancel once, then confirm and verify its items remain.

