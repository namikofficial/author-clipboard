# Async GTK service layer

Move daemon communication and daemon-backed data access out of GTK callbacks so delayed, unavailable, or restarting daemon processes cannot freeze or corrupt the UI.

The feature introduces one typed asynchronous service boundary for history, search, status, copy, mutations, snippets, and collections. GTK callbacks enqueue work and receive results only on the GTK main context.
