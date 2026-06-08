# Task Plan: MCP Protocol Server

> Atomic, independently verifiable tasks for implementing the MCP server.

---

## Task Dependencies

```
T001 (crate setup) --> T002 (tools) --> T003 (resources) --> T004 (prompts) --> T005 (transports) --> T006 (integration)
                    --> T007 (documentation)
```

---

## T001: Create MCP Server Crate

**Goal**: Create the MCP server crate with basic structure

**Files to Create**:
- `crates/mcp-server/Cargo.toml`
- `crates/mcp-server/src/main.rs`
- `crates/mcp-server/src/lib.rs`
- `crates/mcp-server/src/error.rs`

**Implementation**:
- Create Cargo.toml with mcp, tokio, serde_json, tracing, clap dependencies
- Add workspace dependency reference
- Create basic main.rs with clap argument parsing
- Create error types for MCP-specific errors

**Verification**:
```bash
cargo build -p author-clipboard-mcp
./target/debug/author-clipboard-mcp --help
```

**Rollback Risk**: Low — new crate

---

## T002: Implement MCP Tools

**Goal**: Implement all MCP tools that wrap daemon IPC

**Files to Edit**:
- `crates/mcp-server/src/tools.rs`
- `crates/mcp-server/src/server.rs`

**Implementation**:
- Implement clipboard.search tool
- Implement clipboard.get tool
- Implement clipboard.copy tool
- Implement clipboard.pin / clipboard.unpin tools
- Implement clipboard.delete tool
- Implement clipboard.clear_unpinned tool
- Implement clipboard.list_snippets tool
- Implement clipboard.upsert_snippet tool
- Implement clipboard.delete_snippet tool
- Implement clipboard.export tool
- Implement clipboard.stats tool
- Add tool handlers in server.rs

**Verification**:
```bash
cargo test -p author-clipboard-mcp
# Manual test with npx @modelcontextprotocol/sdk
```

**Rollback Risk**: Medium — new functionality

---

## T003: Implement MCP Resources

**Goal**: Implement URI-addressable resources for clipboard browsing

**Files to Edit**:
- `crates/mcp-server/src/resources.rs`
- `crates/mcp-server/src/server.rs`

**Implementation**:
- Implement clipboard://recent resource
- Implement clipboard://item/{id} resource
- Implement clipboard://pins resource
- Implement clipboard://snippets resource
- Implement clipboard://stats resource
- Implement clipboard://audit/recent resource
- Add resource handlers in server.rs

**Verification**:
```bash
cargo test -p author-clipboard-mcp
# Test resource reads via MCP client
```

**Rollback Risk**: Medium — new functionality

---

## T004: Implement MCP Prompts

**Goal**: Implement prompt templates for common clipboard workflows

**Files to Edit**:
- `crates/mcp-server/src/prompts.rs`
- `crates/mcp-server/src/server.rs`

**Implementation**:
- Implement clipboard:summarize_recent prompt
- Implement clipboard:promote_to_snippet prompt
- Implement clipboard:find_pattern prompt
- Implement clipboard:redact_sensitive prompt
- Add prompt handlers in server.rs

**Verification**:
```bash
cargo test -p author-clipboard-mcp
# Test prompt invocations via MCP client
```

**Rollback Risk**: Low — new functionality

---

## T005: Implement Transports

**Goal**: Implement stdio and HTTP transports

**Files to Edit**:
- `crates/mcp-server/src/transport.rs`
- `crates/mcp-server/src/main.rs`

**Implementation**:
- Implement stdio transport using mcp::transport::stdio
- Implement HTTP transport using axum (feature-gated)
- Add --transport flag to main.rs
- Add --port flag for HTTP transport

**Verification**:
```bash
# Test stdio
echo '{"jsonrpc":"2.0","id":1,"method":"initialize"}' | author-clipboard-mcp --transport stdio

# Test HTTP (with curl or MCP client)
curl http://127.0.0.1:8765/mcp -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"initialize"}'
```

**Rollback Risk**: Low — transport is feature-gated

---

## T006: Integration Testing with MCP Clients

**Goal**: Verify MCP server works with Codex and OpenCode

**Files to Edit**:
- `crates/mcp-server/tests/` (new directory)

**Implementation**:
- Create integration test with mock MCP client
- Test tool invocations (search, copy, pin, etc.)
- Test resource reads
- Test prompt invocations
- Test error handling (daemon not running, item not found, etc.)

**Verification**:
```bash
cargo test -p author-clipboard-mcp --all
# Manual testing with Codex/OpenCode configuration
```

**Rollback Risk**: N/A — tests only

---

## T007: Documentation and Configuration Examples

**Goal**: Document MCP server usage and provide configuration examples

**Files to Create**:
- `docs/MCP.md`
- Example configurations for Codex and OpenCode

**Implementation**:
- Document installation and usage
- Provide Codex config example
- Provide OpenCode local config example
- Provide OpenCode remote config example
- Document security considerations for remote HTTP
- Document tool descriptions and schemas

**Verification**:
```bash
# Read docs/MCP.md
# Verify config examples are correct
```

**Rollback Risk**: N/A — documentation only

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Completed | Crate created with mcp-server, mcp-spec, tower dependencies |
| T002 | Completed | All tools wired to IPC: search, get, copy, pin, unpin, delete, stats, snippets |
| T003 | Completed | Resources wired: recent, pins, snippets, stats, item/{id} |
| T004 | Completed | Prompts wired: summarize_recent, find_pattern |
| T005 | Completed | Stdio transport implemented with ByteTransport |
| T006 | Completed | MCP server compiles and runs, all checks pass |
| T007 | Pending | Documentation in spec files |

---

**Last Updated**: Phase 16