# Requirements: Unify GTK Selection, Actions, and Visible State

## Acceptance criteria

1. `AppState` contains no row-index selection state; unknown or invisible IDs
   clear selection, while refreshes preserve a visible selected ID and choose
   the first visible item only when there is no valid selection.
2. Deleting a selected item chooses the item now occupying its position, or
   the previous item when it was last; empty snapshots clear selection.
3. GTK rows carry their database IDs and selection/activation/keyboard paths
   dispatch the same ID-based state action without an index side table.
4. Preview and contextual commands resolve the same authoritative selected
   item at dispatch time. No-selection and protected-item availability rules
   are represented by testable helpers.
5. Pin/star mutations update the visible in-memory snapshot immediately.
6. Tests cover refresh, filtering, deletion, movement, row-ID dispatch,
   command resolution, and contextual availability.
