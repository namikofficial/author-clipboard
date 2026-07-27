# Test plan

- Shared IPC: request IDs, malformed responses, missing responses, timeout boundaries, and daemon response writes.
- Service worker: delayed response, connection failure, restart/reconnect, write/response timeout, and typed error mapping.
- GTK-independent controller: generation ordering and rapid query/filter changes.
- Mock service UI integration: callbacks enqueue immediately, successful empty results remain distinct from errors, and mutations refresh asynchronously.
- Required commands: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `just verify`.
