# Domain Model: MCP Protocol Server

> Data structures, state, and relationships for the MCP server implementation.

---

## Data Structures

### New Crate: mcp-server

```
crates/
└── mcp-server/           # New crate for MCP server
    ├── src/
    │   ├── main.rs       # Entry point, transport setup
    │   ├── server.rs     # MCP server implementation
    │   ├── tools.rs      # Tool definitions
    │   ├── resources.rs  # Resource definitions
    │   ├── prompts.rs    # Prompt templates
    │   └── transport.rs  # stdio/HTTP transport
    └── Cargo.toml
```

### Tool Definitions

```rust
// In mcp-server/src/tools.rs

use serde::{Deserialize, Serialize};

/// Tool: clipboard.search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTool {
    pub name: String,
    pub description: String,
    pub input_schema: SearchInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    pub query: String,
    pub limit: Option<usize>,      // default 50, max 200
    pub offset: Option<usize>,
    pub content_type: Option<Vec<String>>, // ["text", "image", "html", "files"]
    pub pinned: Option<bool>,
    pub sensitive: Option<bool>,
    pub source_app: Option<String>,
    pub age_max_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOutput {
    pub items: Vec<ClipboardItemPreview>,
    pub total: usize,
    pub has_more: bool,
}

/// Tool: clipboard.get
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTool {
    pub name: String,
    pub description: String,
    pub input_schema: GetInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInput {
    pub id: i64,
    pub include_content: Option<bool>, // default false, require confirmation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOutput {
    pub item: ClipboardItemFull,
}

/// Tool: clipboard.copy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTool {
    pub name: String,
    pub description: String,
    pub input_schema: CopyInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyInput {
    pub id: i64,
    pub mode: CopyMode,
    pub confirm_sensitive: Option<bool>, // required if sensitive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyOutput {
    pub success: bool,
    pub mime_type: String,
    pub behavior: String,
}

/// Tool: clipboard.pin / clipboard.unpin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinTool {
    pub name: String,
    pub description: String,
    pub input_schema: PinInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinInput {
    pub id: i64,
}

/// Tool: clipboard.delete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTool {
    pub name: String,
    pub description: String,
    pub input_schema: DeleteInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteInput {
    pub id: i64,
    pub confirm: bool, // must be true
}

/// Tool: clipboard.clear_unpinned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearUnpinnedTool {
    pub name: String,
    pub description: String,
    pub input_schema: ClearUnpinnedInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearUnpinnedInput {
    pub confirm: bool, // must be true
}

/// Tool: clipboard.list_snippets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSnippetsTool {
    pub name: String,
    pub description: String,
    pub input_schema: EmptyInput,
}

/// Tool: clipboard.upsert_snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSnippetTool {
    pub name: String,
    pub description: String,
    pub input_schema: UpsertSnippetInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSnippetInput {
    pub name: String,
    pub content: String,
}

/// Tool: clipboard.delete_snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSnippetTool {
    pub name: String,
    pub description: String,
    pub input_schema: DeleteSnippetInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSnippetInput {
    pub id: i64,
    pub confirm: bool,
}

/// Tool: clipboard.export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTool {
    pub name: String,
    pub description: String,
    pub input_schema: ExportInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInput {
    pub format: Option<String>, // "json" (default)
    pub limit: Option<usize>,
}

/// Tool: clipboard.stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsTool {
    pub name: String,
    pub description: String,
    pub input_schema: EmptyInput,
}
```

### Resource Definitions

```rust
// In mcp-server/src/resources.rs

/// Resource: clipboard://recent
/// Read recent clipboard items
pub struct RecentResource {
    pub uri: String,           // "clipboard://recent?limit=50&offset=0"
    pub name: String,
    pub description: String,
    pub mime_type: String,     // "application/json"
}

/// Resource: clipboard://item/{id}
/// Read a specific clipboard item
pub struct ItemResource {
    pub uri_template: String,   // "clipboard://item/{id}"
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// Resource: clipboard://pins
/// Read all pinned items
pub struct PinsResource {
    pub uri: String,           // "clipboard://pins"
    pub name: String,
    pub description: String,
}

/// Resource: clipboard://snippets
/// Read all snippets
pub struct SnippetsResource {
    pub uri: String,           // "clipboard://snippets"
    pub name: String,
    pub description: String,
}

/// Resource: clipboard://stats
/// Read database statistics
pub struct StatsResource {
    pub uri: String,           // "clipboard://stats"
    pub name: String,
    pub description: String,
}

/// Resource: clipboard://audit/recent
/// Read recent audit events
pub struct AuditResource {
    pub uri: String,           // "clipboard://audit/recent?limit=50"
    pub name: String,
    pub description: String,
}

/// Resource: clipboard://collections/{name}
/// Read items in a named collection
pub struct CollectionResource {
    pub uri_template: String,   // "clipboard://collections/{name}"
    pub name: String,
    pub description: String,
}
```

### Prompt Definitions

```rust
// In mcp-server/src/prompts.rs

/// Prompt: summarize_recent
/// Summarize recent clipboard items
pub struct SummarizeRecentPrompt {
    pub name: String,           // "clipboard:summarize_recent"
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Prompt: promote_to_snippet
/// Create a snippet from current clipboard
pub struct PromoteToSnippetPrompt {
    pub name: String,           // "clipboard:promote_to_snippet"
    pub description: String,
}

/// Prompt: find_pattern
/// Find clipboard items matching a pattern
pub struct FindPatternPrompt {
    pub name: String,           // "clipboard:find_pattern"
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt: redact_sensitive
/// Redact sensitive data from clipboard items
pub struct RedactSensitivePrompt {
    pub name: String,           // "clipboard:redact_sensitive"
    pub description: String,
}
```

---

## State Machine

### MCP Server Lifecycle

```
[Starting] --> [Loading Config] --> [Connecting to Daemon] --> [Ready]
                                              |
                                              v
                                        [Connection Failed]
                                              |
                                              v
                                      [Retry with Backoff]
```

### Tool Invocation Flow

```
[AI Agent] --> [MCP Client] --> [stdio/HTTP Transport]
                                  |
                                  v
                            [Parse Request]
                                  |
                                  v
                            [Route to Tool Handler]
                                  |
                                  v
                            [IPC to Daemon]
                                  |
                                  v
                            [Return Result/Error]
```

---

## Configuration

### MCP Server Configuration

```json
// ~/.config/author-clipboard/mcp.json
{
  "version": "1.0",
  "transport": "stdio",           // "stdio" or "http"
  "http": {
    "host": "127.0.0.1",
    "port": 8765,
    "auth": null                  // future: { "type": "bearer", "token": "..." }
  },
  "defaults": {
    "limit": 50,
    "show_sensitive_previews": false,
    "confirm_destructive": true
  },
  "allowed_tools": [
    "clipboard.search",
    "clipboard.get",
    "clipboard.copy",
    "clipboard.pin",
    "clipboard.unpin",
    "clipboard.list_snippets",
    "clipboard.upsert_snippet",
    "clipboard.stats"
  ],
  "denied_tools": [
    "clipboard.delete",
    "clipboard.clear_unpinned",
    "clipboard.delete_snippet"
  ]
}
```

---

**Last Updated**: Phase 15