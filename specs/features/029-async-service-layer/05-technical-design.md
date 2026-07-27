# Technical design

`ui-gtk` owns a `ServiceHandle` backed by a Tokio runtime and a dedicated worker. The worker owns all IPC clients and transport state. GTK code sends `ServiceRequest` values through a channel; result callbacks are attached to the default GLib main context. No GTK object crosses the worker boundary.

The shared IPC response gains an optional request ID. The daemon writes one response per request and the client validates version, response ID, and payload shape. The service reconnects after connection/receive failures. Each operation is bounded by connect, write, and response timeouts.

Search generations are allocated on the GTK side and carried in the request. The worker may replace queued searches; the GTK result handler applies only the latest generation.

All current direct database reads from GTK pages are replaced by service operations. The daemon remains the owner of SQLite access.
