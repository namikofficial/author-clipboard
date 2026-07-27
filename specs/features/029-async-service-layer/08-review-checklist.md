# Review checklist

- [ ] No GTK callback performs synchronous IPC or opens the daemon database.
- [ ] Worker-owned transport has bounded connect, write, and response waits.
- [ ] Request IDs and protocol versions are validated.
- [ ] Stale searches cannot update visible state.
- [ ] Offline, timeout, protocol, validation, and database errors are visible.
- [ ] Restarting the daemon is recoverable.
- [ ] Mock-service tests cover all required validation scenarios.
- [ ] Workspace tests, clippy, and `just verify` pass.
