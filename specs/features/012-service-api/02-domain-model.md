# Domain Model: Service API

> Data structures, state, and relationships for the normalized service API.

---

## Data Structures

### New Types

```rust
// In shared/src/ipc.rs

/// Versioned IPC request envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub version: String,        // e.g., "1.0"
    pub cmd: String,            // e.g., "history"
    pub args: Value,            // JSON object with command arguments
    pub request_id: Option<u64>, // For tracing/correlating requests
}

/// Versioned IPC response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub version: String,
    pub ok: bool,
    pub data: Option<Value>,
    pub error: Option<IpcErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcErrorDetail {
    pub code: String,           // e.g., "DAEMON_NOT_RUNNING"
    pub message: String,
    pub min_version: Option<String>, // For UNKNOWN_COMMAND errors
}

/// Unified command enum (replaces current IpcMessage)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum IpcCommand {
    // Visibility
    Toggle,
    Show,
    Hide,
    ShowAt { x: i32, y: i32 },

    // Health
    Ping,
    Status,

    // Query
    History { limit: usize, offset: Option<usize>, filters: Option<FilterOptions> },
    GetItem { id: i64 },
    Search { query: String, limit: Option<usize>, filters: Option<FilterOptions> },
    GetStats,
    GetAuditLog { limit: Option<usize> },

    // Mutations
    Copy { id: i64, mode: CopyMode },
    Pin { id: i64 },
    Unpin { id: i64 },
    Delete { id: i64 },
    ClearUnpinned,
    ClearAll,
    Import { items: Vec<ClipboardItem> },

    // Snippets
    ListSnippets,
    UpsertSnippet { name: String, content: String },
    DeleteSnippet { id: i64 },

    // Config
    GetConfig,
    UpdateConfig { config: ConfigUpdate },

    // Subscriptions
    Subscribe { events: Vec<SubscriptionEvent> },
    Unsubscribe { subscription_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CopyMode {
    Copy,                // Write to clipboard
    QuickPaste,          // Write to clipboard and type it
    CopyPlainText,       // Strip formatting
    CopyRedacted,        // Replace sensitive data with •••
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub content_type: Option<Vec<ContentType>>,
    pub pinned: Option<bool>,
    pub sensitive: Option<bool>,
    pub source_app: Option<String>,
    pub age_min_seconds: Option<u64>,
    pub age_max_seconds: Option<u64>,
    pub search_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionEvent {
    ItemAdded,
    ItemUpdated,
    ItemDeleted,
    PinToggled,
    HistoryCleared,
    ConfigChanged,
    IncognitoChanged,
}
```

### Changes to Existing Types

The `ClipboardItem` type remains unchanged. The `Config` type gains an optional `min_api_version` field:

```rust
// shared/src/config.rs
pub struct Config {
    // ... existing fields ...
    /// Minimum API version to advertise to clients
    #[serde(default = "default_min_api_version")]
    pub min_api_version: String,
}

fn default_min_api_version() -> String {
    "1.0".to_string()
}
```

---

## State Machine

### Daemon State

```
[Starting] --> [Init DB] --> [Bind IPC] --> [Connect Wayland] --> [Running]
                                    |                                      |
                                    v                                      v
                              [IPC Ready]                           [Capturing]
                                    |                                      |
                                    v                                      v
                              [Accepting Commands] <---------------- [Main Loop]
```

### Client Connection State

```
[Disconnected] --> [Connecting] --> [Authenticating] --> [Subscribed] --> [Ready]
                                           |                                    |
                                           v                                    v
                                      [Error]                             [Reconnecting]
```

### Live Update Flow

```
[CLI/APplet/MCP] --> [IPC Request] --> [Daemon Processes] --> [State Change]
                                                           |
                                                           v
                                                      [Broadcast to All Subscribers]
```

---

## Database Changes

### New Tables

```sql
-- Subscriptions for live updates
CREATE TABLE ipc_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL,
    events TEXT NOT NULL,           -- JSON array of SubscriptionEvent
    created_at TEXT NOT NULL,
    last_ping TEXT NOT NULL
);

-- API version tracking
CREATE TABLE api_version (
    version TEXT NOT NULL PRIMARY KEY,
    min_version TEXT NOT NULL,
    introduced_at TEXT NOT NULL
);
```

### No Schema Migrations Needed

This feature does not change the database schema. All data structures are in-memory or use existing tables.

---

## IPC Protocol Changes

### New Request/Response Format

**Request**:
```json
{
  "version": "1.0",
  "cmd": "history",
  "args": {
    "limit": 50,
    "offset": 0,
    "filters": {
      "content_type": ["text", "html"],
      "pinned": false,
      "age_max_seconds": 86400
    }
  },
  "request_id": 12345
}
```

**Response (success)**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "items": [...],
    "total": 150,
    "offset": 0,
    "limit": 50,
    "has_more": true
  },
  "error": null
}
```

**Response (error)**:
```json
{
  "version": "1.0",
  "ok": false,
  "data": null,
  "error": {
    "code": "ITEM_NOT_FOUND",
    "message": "No clipboard item with id 999",
    "min_version": null
  }
}
```

### Error Codes

| Code | HTTP-like | Meaning |
|------|-----------|---------|
| `DAEMON_NOT_RUNNING` | 503 | Cannot connect to daemon |
| `DAEMON_ERROR` | 500 | Internal daemon error |
| `INVALID_REQUEST` | 400 | Malformed request JSON |
| `INVALID_ARG` | 400 | Valid JSON but invalid arguments |
| `UNKNOWN_COMMAND` | 404 | Command not recognized |
| `UNSUPPORTED_VERSION` | 400 | Version too old, min_version included |
| `ITEM_NOT_FOUND` | 404 | Requested item doesn't exist |
| `PERMISSION_DENIED` | 403 | Operation not permitted |
| `CONFLICT` | 409 | State conflict (e.g., already pinned) |

---

## CLI Changes

### Before (Direct DB Access)

```bash
author-clipboard-ctl history 10
# Output: "[1] first item... [2] second item..."
```

### After (IPC Routing)

```bash
author-clipboard-ctl history --limit 10 --json
# Output: {"items": [{"id": 1, "content": "..."}], "total": 10}
```

**Key changes**:
- All CLI commands now go through IPC
- `--json` flag for machine-readable output (default for most commands)
- `--pretty` flag for human-readable output
- Daemon returns structured data, CLI formats it

### New CLI Flags

| Flag | Commands | Description |
|------|----------|-------------|
| `--json` | all | Output machine-readable JSON |
| `--pretty` | all | Output formatted human-readable text |
| `--daemon-only` | all | Fail if daemon not reachable (vs fallback to direct DB) |
| `--filter TYPE` | history, search | Filter by content type |
| `--pinned` | history, search | Show only pinned items |
| `--age SECONDS` | history, search | Show items from last N seconds |

---

**Last Updated**: Phase 15