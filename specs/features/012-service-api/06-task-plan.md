# Task Plan: Service API Normalization

> Atomic, independently verifiable tasks for normalizing the service API.

---

## Task Dependencies

```
T001 (ipc types) --> T002 (daemon handlers) --> T003 (cli routing) --> T004 (subscription system) --> T005 (integration tests)
                --> T006 (mcp foundation)
```

---

## T001: Expand IPC Types

**Goal**: Add IpcRequest/IpcResponse envelopes and expand IpcCommand with all operations

**Files to Edit**:
- `crates/shared/src/ipc.rs`

**Implementation**:
- Add `IpcRequest` struct with version, cmd, args, request_id
- Add `IpcResponse` struct with version, ok, data, error
- Add `IpcErrorDetail` struct with code, message, min_version
- Expand `IpcCommand` enum with all query/mutation commands
- Add `FilterOptions`, `CopyMode`, `SubscriptionEvent` types
- Add error code constants

**Verification**:
```bash
cargo test -p author-clipboard-shared -- ipc
just verify
```

**Rollback Risk**: Low — adding new types only

---

## T002: Implement Daemon IPC Command Handlers

**Goal**: Implement all IPC command handlers in the daemon

**Files to Edit**:
- `crates/clipboard-daemon/src/main.rs`

**Implementation**:
- Add version validation in accept loop
- Implement `handle_history()`, `handle_get_item()`, `handle_search()`
- Implement `handle_copy()`, `handle_pin()`, `handle_unpin()`, `handle_delete()`
- Implement `handle_clear_unpinned()`, `handle_clear_all()`
- Implement `handle_list_snippets()`, `handle_upsert_snippet()`, `handle_delete_snippet()`
- Implement `handle_get_config()`, `handle_update_config()`
- Return structured JSON responses for all commands
- Add audit logging for all mutations

**Verification**:
```bash
cargo test -p author-clipboard-daemon
just verify
```

**Rollback Risk**: Medium — changes daemon behavior significantly

---

## T003: Route CLI Through IPC

**Goal**: Make CLI route all operations through daemon IPC instead of direct DB access

**Files to Edit**:
- `crates/ctl/src/main.rs`

**Implementation**:
- Change all command handlers to use IpcClient
- Add `--json` flag (default for machine output)
- Add `--pretty` flag for human-readable output
- Add `--daemon-only` flag to fail when daemon is down
- Remove direct `Database::open()` calls from CLI
- Remove direct database query code from CLI handlers
- Add fallback message when daemon not reachable

**Verification**:
```bash
cargo test -p author-clipboard-ctl
cargo build -p author-clipboard-ctl
author-clipboard-ctl status --daemon-only
```

**Rollback Risk**: Medium — changes CLI behavior significantly

---

## T004: Implement Live Update Subscription System

**Goal**: Add subscription system for live updates to connected clients

**Files to Edit**:
- `crates/clipboard-daemon/src/main.rs`

**Implementation**:
- Add `Subscription` struct with id, client_id, events, channel
- Add `subscriptions` HashMap to DaemonState
- Implement `handle_subscribe()` and `handle_unsubscribe()`
- Add `broadcast()` method for sending events to subscribers
- Call broadcast() after every state mutation (insert, delete, pin, etc.)
- Handle subscription cleanup when client disconnects

**Verification**:
```bash
# Manual test: open applet, run CLI command, observe live update
just run &
author-clipboard-ctl copy 1
# applet should update
```

**Rollback Risk**: Low — additive feature

---

## T005: Integration Tests for Service API

**Goal**: Add integration tests verifying full CLI-to-daemon round-trip

**Files to Edit**:
- `crates/clipboard-daemon/src/integration_tests.rs` (new file)

**Implementation**:
- Test history command returns structured JSON
- Test copy command triggers clipboard write
- Test clear command deletes unpinned items
- Test pin/unpin commands update item state
- Test search command returns filtered results
- Test config commands get/update configuration
- Test daemon not running returns clear error
- Test version negotiation (send old version, get min_version error)

**Verification**:
```bash
cargo test --all
just verify
```

**Rollback Risk**: N/A — tests only

---

## T006: Prepare MCP Foundation

**Goal**: Ensure IPC command set covers all MCP tool requirements

**Files to Edit**:
- `crates/shared/src/ipc.rs`
- `crates/clipboard-daemon/src/main.rs`

**Implementation**:
- Verify all MCP tools have corresponding IPC commands
- Add any missing commands for MCP completeness
- Document command-to-MCP-tool mapping

**Verification**:
```bash
# See feature 013-mcp-protocol for MCP tool mapping
```

**Rollback Risk**: Low — ensures MCP compatibility

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Completed | IpcCommand expanded with collections, stars, ToggleStar, GetCollectionItems, AddToCollection, RemoveFromCollection |
| T002 | Completed | CLI routes through daemon IPC via IpcClient::send_command |
| T003 | Completed | All query commands support FilterOptions with content_type, pinned, sensitive, source_app, age filters |
| T004 | Completed | CopyMode enum with Copy, QuickPaste, CopyPlainText, CopyRedacted |
| T005 | Completed | Service normalizes all operations through daemon |
| T006 | Completed | Daemon handles all mutations, CLI is read-only client |

---

**Last Updated**: Phase 16