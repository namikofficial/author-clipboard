# Brief: Unify GTK Selection, Actions, and Visible State

Make the GTK UI use `AppState.selected_id` as its only stored clipboard
selection. Visible rows, keyboard movement, previews, refresh reconciliation,
and contextual commands must resolve through stable database IDs.

This slice is limited to the GTK UI state, widgets, controllers, popup/page
wiring, tests, and rebuild documentation. It does not change the public IPC
schema.
