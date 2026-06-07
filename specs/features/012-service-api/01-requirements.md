# Requirements: Service API Normalization

> Requirements for making the daemon the single authoritative service for all clipboard operations.

---

## User Stories

### US-001: All CLI Operations Route Through Daemon
**As a** user
**I want to** run `author-clipboard-ctl history`, `copy`, `clear`, `export`, and `config` and have them communicate with the daemon
**So that** all operations are subject to the same policy, audit, and live-update infrastructure

**Acceptance Criteria**:
- Given the daemon is running, when I run `author-clipboard-ctl history --limit 10`, then the CLI sends an IPC request to the daemon and the daemon queries the database and returns structured JSON
- Given the daemon is running, when I run `author-clipboard-ctl copy 42`, then the daemon retrieves the item, applies masking if sensitive, and writes to the clipboard
- Given the daemon is not running, when I run any CLI command, then the CLI returns a clear error "Daemon not running" (not a database error)

### US-002: Unified Policy Enforcement
**As a** user
**I want to** have sensitive items masked, retention enforced, and deduplication applied consistently whether the operation comes from CLI, applet, or MCP
**So that** I can trust the security model regardless of how I interact with the system

**Acceptance Criteria**:
- Given an item is marked sensitive, when any client requests a preview, then the daemon returns a masked preview (e.g., "••••••••") regardless of which client
- Given `max_items` is 100, when the 101st item is inserted, then the daemon enforces the limit (not the CLI or applet)
- Given `dedup_window_seconds` is 2, when identical content is copied within 2 seconds, then the daemon bumps the existing item (not the CLI)

### US-003: Audit Trail for All Operations
**As a** user
**I want to** see an audit log entry for every state mutation, including those initiated by CLI
**So that** I can trace what happened to my clipboard history

**Acceptance Criteria**:
- Given I run `author-clipboard-ctl clear`, when the operation succeeds, then an audit log entry is created with event_kind="history_cleared" and details including the count of items cleared
- Given I run `author-clipboard-ctl copy 42`, when the item is sensitive, then the audit log records "item_copied" with sensitive=true
- Given the audit log has more than 1000 entries, when cleanup runs, then old entries are trimmed but at least 500 remain

### US-004: Live Update Notifications
**As a** user
**I want to** see the applet's clipboard list update when CLI adds a new item
**So that** I don't need to restart the applet to see new clipboard entries

**Acceptance Criteria**:
- Given the applet is open and the daemon is running, when I copy something from a terminal, then the applet's list updates within 1 second to show the new item
- Given the applet is open, when I run `author-clipboard-ctl clear`, then the applet's list updates to reflect the cleared state
- Given the applet is open, when I pin an item via CLI, then the applet shows the pin state change

### US-005: Stable Versioned Service API
**As a** developer
**I want to** have a stable IPC command surface that I can build MCP on top of
**So that** MCP tools and resources map cleanly to daemon operations

**Acceptance Criteria**:
- Given I am designing MCP tools, when I look at the IPC command list, then I see a complete, consistent command set with typed request/response
- Given the daemon is at version 1.0, when I send a v1.0 command, then it works and the response format matches the spec
- Given a new command is added in v1.1, when a v1.0 client sends it, then the daemon returns "UNSUPPORTED_COMMAND" with the minimum supported version

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | All CLI commands route through daemon IPC | Must | No direct DB access from CLI |
| FR-002 | Daemon exposes complete query/mutation API | Must | history, copy, clear, export, config, pin, unpin, delete |
| FR-003 | Structured JSON responses for all commands | Must | Not human-readable output |
| FR-004 | Audit logging for all state mutations | Must | Including CLI-initiated ones |
| FR-005 | Live update pub/sub for state changes | Must | Notify all connected clients |
| FR-006 | Policy enforcement in daemon layer | Must | Sensitivity masking, retention, dedup |
| FR-007 | Versioned IPC protocol (v1.0) | Must | Minimum version tracking |
| FR-008 | Graceful degradation when daemon is down | Must | Clear error messages |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | IPC round-trip latency | < 50ms | For query operations |
| NFR-002 | Daemon handles 100+ concurrent IPC connections | Must | Live updates require multi-client |
| NFR-003 | CLI responds within 200ms for all commands | Must | Including daemon round-trips |
| NFR-004 | No breaking changes to CLI flags | Must | Only internal routing changes |

---

## Edge Cases

| Case | Handling |
|------|----------|
| Daemon not running | CLI returns exit code 1 with "Daemon not running" message |
| Daemon crashes during operation | Transaction rollback, client receives "DAEMON_ERROR" |
| Concurrent CLI + applet operations | SQLite WAL handles concurrent reads; write serialization via mutex |
| Large result sets | Paginated responses with cursor-based pagination |
| Unknown IPC command | Return `{"ok": false, "error": "UNKNOWN_COMMAND", "min_version": "1.0"}` |

---

## Out of Scope

- Remote IPC (network-based daemon access)
- Full RPC framework with service discovery
- TLS encryption for IPC (local socket is trusted)
- Multi-daemon clustering

---

## Dependencies

- Feature `001-clipboard-history` (existing)
- Feature `018-dedup-fix` (new, fixes dedup behavior)
- Feature `019-config-cleanup` (new, renames content_regex_denylist)

---

**Last Updated**: Phase 15