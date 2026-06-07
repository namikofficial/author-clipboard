# Requirements: MCP Protocol Server

> Requirements for implementing a Model Context Protocol server for clipboard integration.

---

## User Stories

### US-001: Search Clipboard from AI Agent
**As a** developer using Codex
**I want to** search my clipboard history for API patterns or configuration snippets
**So that** I can incorporate recent context into code generation

**Acceptance Criteria**:
- Given I am in a Codex session, when I invoke the clipboard.search tool with query="AWS config", then I receive a list of matching clipboard items with masked previews
- Given I am in an OpenCode session, when I use the clipboard resource to browse recent items, then I see items with source app, timestamp, and type indicators

### US-002: Copy Item to Clipboard from AI Agent
**As a** developer using Codex
**I want to** copy a specific clipboard item using its ID
**So that** I can prepare context for pasting into my editor

**Acceptance Criteria**:
- Given a clipboard item ID, when I invoke clipboard.copy with mode="copy", then the item is written to the Wayland clipboard
- Given a sensitive item, when I invoke clipboard.copy without confirmation, then the tool returns an error requiring user confirmation

### US-003: Manage Snippets via AI Agent
**As a** developer
**I want to** list, create, update, and delete snippets through MCP tools
**So that** I can maintain my snippet library without switching contexts

**Acceptance Criteria**:
- Given I invoke clipboard.list_snippets, then I receive all snippets with names and updated timestamps
- Given I invoke clipboard.upsert_snippet with name and content, then the snippet is created or updated
- Given I invoke clipboard.delete_snippet with an ID, then the snippet is deleted

### US-004: Pin and Organize via AI Agent
**As a** developer
**I want to** pin important clipboard items through MCP
**So that** I can preserve critical snippets without manual UI interaction

**Acceptance Criteria**:
- Given an item ID, when I invoke clipboard.pin, then the item is marked as pinned
- Given an item ID, when I invoke clipboard.unpin, then the item is unpinned
- Given I invoke clipboard.get_pins, then I receive all pinned items

### US-005: Browse Clipboard as Resources
**As a** developer using OpenCode
**I want to** browse clipboard items as URI-addressable resources
**So that** I can include clipboard context in my workspace without tool invocations

**Acceptance Criteria**:
- Given the resource URI `clipboard://recent?limit=50`, when I read it, then I receive a list of recent items with full metadata
- Given the resource URI `clipboard://item/42`, when I read it, then I receive the full item (respecting sensitivity)
- Given the resource URI `clipboard://snippets`, when I read it, then I receive all snippets

### US-006: Use Prompt Templates
**As a** developer
**I want to** use prompt templates for common clipboard workflows
**So that** I can quickly execute complex operations without building tool calls manually

**Acceptance Criteria**:
- Given the prompt "summarize recent clipboard items", when I invoke it, then I receive a summary of the last 10 items
- Given the prompt "promote current clipboard to snippet", when I invoke it, then I am guided to create a snippet from the current clipboard

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | stdio transport for local MCP | Must | Primary Codex/OpenCode transport |
| FR-002 | HTTP Streamable transport | Should | For remote OpenCode configuration |
| FR-003 | clipboard.search tool | Must | With filters and pagination |
| FR-004 | clipboard.get tool | Must | Fetch single item by ID |
| FR-005 | clipboard.copy tool | Must | With Copy/QuickPaste/CopyPlainText modes |
| FR-006 | clipboard.pin / clipboard.unpin | Must | |
| FR-007 | clipboard.delete tool | Must | Destructive, requires confirmation |
| FR-008 | clipboard.clear_unpinned tool | Must | Destructive, requires confirmation |
| FR-009 | clipboard.list_snippets tool | Must | |
| FR-010 | clipboard.upsert_snippet tool | Must | |
| FR-011 | clipboard.delete_snippet tool | Must | |
| FR-012 | clipboard.export tool | Must | |
| FR-013 | clipboard.stats tool | Must | |
| FR-014 | clipboard:// recent resource | Must | |
| FR-015 | clipboard://item/{id} resource | Must | |
| FR-016 | clipboard://pins resource | Must | |
| FR-017 | clipboard://snippets resource | Must | |
| FR-018 | clipboard://stats resource | Must | |
| FR-019 | Prompt templates | Should | |
| FR-020 | Server instructions | Must | Warn about sensitive handling |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | Tool response latency | < 100ms | For search/list operations |
| NFR-002 | MCP server startup | < 500ms | |
| NFR-003 | Memory footprint | < 20MB | |
| NFR-004 | Token cost optimization | Minimal | Prefer list results over full content |

---

## Edge Cases

| Case | Handling |
|------|----------|
| Sensitive item in search results | Return masked preview, not full content |
| Daemon not running | Return clear error "Daemon not running. Start author-clipboard-daemon" |
| Item not found | Return error code ITEM_NOT_FOUND |
| Destructive tool without confirmation | Return error requiring --confirm flag |
| Large result set | Paginate with limit/offset, default limit 50 |
| Empty search results | Return empty array, not error |

---

## Out of Scope

- OAuth authentication for remote MCP (Phase 13)
- TLS for HTTP transport
- Real-time clipboard capture monitoring via MCP
- Full MCP protocol implementation (use sdk crate)

---

## Dependencies

- Feature `012-service-api` (required - MCP sits above daemon IPC)
- Feature `018-dedup-fix` (ensures consistent hashing)

---

**Last Updated**: Phase 15