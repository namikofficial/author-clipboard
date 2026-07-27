# Async service layer

## Problem

GTK previously called synchronous IPC and opened the daemon database from callbacks. Slow queries or an unavailable daemon could block the main loop, while failures often appeared as empty lists.

## Architecture

GTK uses a cloneable `ServiceHandle` implementing the typed `ClipboardService` boundary. Requests are enqueued to a worker that owns a Tokio runtime and all socket activity. Results return through GLib main-context futures; GTK widgets and `Rc<RefCell<_>>` state never cross the worker boundary.

`ServiceError` distinguishes offline, timeout, protocol, validation, database, daemon, and worker failures. Transport budgets are 500 ms for connection, 500 ms for writes, and 2 s for responses.

Requests carry monotonically increasing IDs. The daemon echoes IDs and writes versioned responses. The client rejects mismatched IDs or protocol versions and maps malformed messages to protocol errors. A later request reconnects after a daemon restart.

History/search requests carry a generation. Results are applied only when their generation is still current, preventing delayed older queries from overwriting newer results. Successful empty results remain distinct from failures, which show an explicit error state.

Clipboard refreshes, copy and action-rail mutations, status, settings clear, snippets, and collections use the service boundary. SQLite remains daemon-owned.

## Verification

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
just verify
```
