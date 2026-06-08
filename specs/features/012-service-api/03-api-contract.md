# API Contract: Service API

> IPC protocol and API definitions for the normalized service API.

---

## Protocol Overview

- **Transport**: Unix domain socket at `$XDG_RUNTIME_DIR/author-clipboard.sock`
- **Format**: JSON lines (one JSON object per message, newline-delimited)
- **Protocol Version**: 1.0 (advertised in every request/response)
- **Request/Response**: Every request gets exactly one response

---

## Connection Flow

1. Client connects to Unix socket
2. Client sends requests; server responds in order
3. Client can send multiple requests (pipelining supported)
4. Connection stays open until client closes or server dies

---

## IPC Commands

### Visibility Commands

#### Toggle
Show picker if hidden, hide if shown.

**Request**:
```json
{"version": "1.0", "cmd": "Toggle", "args": {}, "request_id": 1}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"visible": true}, "error": null}
```

#### Show
Show the picker at last position.

**Request**:
```json
{"version": "1.0", "cmd": "Show", "args": {}, "request_id": 2}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"visible": true}, "error": null}
```

#### Hide
Hide the picker.

**Request**:
```json
{"version": "1.0", "cmd": "Hide", "args": {}, "request_id": 3}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"visible": false}, "error": null}
```

#### ShowAt
Show picker at specific coordinates.

**Request**:
```json
{"version": "1.0", "cmd": "ShowAt", "args": {"x": 100, "y": 200}, "request_id": 4}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"visible": true, "x": 100, "y": 200}, "error": null}
```

---

### Health Commands

#### Ping
Health check. Daemon responds with current state.

**Request**:
```json
{"version": "1.0", "cmd": "Ping", "args": {}, "request_id": 5}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"status": "ok", "uptime_seconds": 3600}, "error": null}
```

#### Status
Get detailed daemon status.

**Request**:
```json
{"version": "1.0", "cmd": "Status", "args": {}, "request_id": 6}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "daemon_version": "1.0.0",
    "visible": false,
    "item_count": 150,
    "pinned_count": 12,
    "incognito": false,
    "database_size_bytes": 1048576,
    "capture_active": true
  },
  "error": null
}
```

---

### Query Commands

#### History
Get recent clipboard items with optional filtering and pagination.

**Request**:
```json
{
  "version": "1.0",
  "cmd": "History",
  "args": {
    "limit": 50,
    "offset": 0,
    "filters": {
      "content_type": ["text", "html"],
      "pinned": null,
      "sensitive": null,
      "source_app": null,
      "age_min_seconds": null,
      "age_max_seconds": null,
      "search_query": null
    }
  },
  "request_id": 7
}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "items": [
      {
        "id": 1,
        "content_hash": "0x1234abcd",
        "content": "••••••••",
        "mime_type": "text/plain",
        "content_type": "text",
        "timestamp": "2026-06-08T10:30:00Z",
        "pinned": false,
        "source_app": "kitty",
        "sensitive": true,
        "plain_text": "••••••••",
        "preview": "ghp_•••••••••••••••"
      }
    ],
    "total": 150,
    "offset": 0,
    "limit": 50,
    "has_more": true
  },
  "error": null
}
```

**Notes**:
- `content` is masked if sensitive and `show_sensitive_previews` is false
- `plain_text` is always masked for sensitive items
- `preview` is a truncated, masked preview suitable for list display

#### GetItem
Get a single item by ID with full content (respects sensitivity).

**Request**:
```json
{"version": "1.0", "cmd": "GetItem", "args": {"id": 42}, "request_id": 8}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "id": 42,
    "content_hash": "0x5678efgh",
    "content": "actual content here",
    "mime_type": "text/plain",
    "content_type": "text",
    "timestamp": "2026-06-08T10:30:00Z",
    "pinned": true,
    "source_app": "kitty",
    "sensitive": false,
    "plain_text": "actual content here",
    "preview": "actual content here"
  },
  "error": null
}
```

**Error** (if sensitive and not allowed):
```json
{
  "version": "1.0",
  "ok": false,
  "data": null,
  "error": {
    "code": "SENSITIVE_CONTENT",
    "message": "Item contains sensitive content. Use Copy mode with confirmation.",
    "min_version": null
  }
}
```

#### Search
Full-text search with filtering.

**Request**:
```json
{
  "version": "1.0",
  "cmd": "Search",
  "args": {
    "query": "password",
    "limit": 20,
    "filters": {
      "content_type": null,
      "pinned": null,
      "sensitive": null,
      "source_app": null,
      "age_min_seconds": null,
      "age_max_seconds": null,
      "search_query": null
    }
  },
  "request_id": 9
}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "items": [...],
    "total": 5,
    "offset": 0,
    "limit": 20,
    "has_more": false
  },
  "error": null
}
```

#### GetStats
Get database statistics.

**Request**:
```json
{"version": "1.0", "cmd": "GetStats", "args": {}, "request_id": 10}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "total_items": 150,
    "pinned_items": 12,
    "total_size_bytes": 1048576,
    "oldest_item": "2026-06-01T10:00:00Z",
    "newest_item": "2026-06-08T10:30:00Z",
    "capture_rate_per_hour": 45.2
  },
  "error": null
}
```

#### GetAuditLog
Get recent audit events.

**Request**:
```json
{"version": "1.0", "cmd": "GetAuditLog", "args": {"limit": 100}, "request_id": 11}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "events": [
      {
        "id": 1000,
        "event_kind": "item_copied",
        "details": "{\"id\": 42, \"sensitive\": true}",
        "timestamp": "2026-06-08T10:30:00Z"
      }
    ]
  },
  "error": null
}
```

---

### Mutation Commands

#### Copy
Copy an item to the clipboard with specified mode.

**Request**:
```json
{"version": "1.0", "cmd": "Copy", "args": {"id": 42, "mode": "Copy"}, "request_id": 12}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "id": 42,
    "mime_type": "text/plain",
    "behavior": "copied",
    "sensitive_confirmed": false
  },
  "error": null
}
```

**Copy Modes**:
- `Copy`: Write to clipboard only
- `QuickPaste`: Write to clipboard and type into active window
- `CopyPlainText`: Strip formatting before copying
- `CopyRedacted`: Replace sensitive patterns with ••• before copying

#### Pin
Pin an item so it won't be auto-deleted.

**Request**:
```json
{"version": "1.0", "cmd": "Pin", "args": {"id": 42}, "request_id": 13}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"id": 42, "pinned": true}, "error": null}
```

#### Unpin
Unpin an item.

**Request**:
```json
{"version": "1.0", "cmd": "Unpin", "args": {"id": 42}, "request_id": 14}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"id": 42, "pinned": false}, "error": null}
```

#### Delete
Delete a single item.

**Request**:
```json
{"version": "1.0", "cmd": "Delete", "args": {"id": 42}, "request_id": 15}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"deleted_id": 42}, "error": null}
```

#### ClearUnpinned
Delete all non-pinned items.

**Request**:
```json
{"version": "1.0", "cmd": "ClearUnpinned", "args": {}, "request_id": 16}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"deleted_count": 138}, "error": null}
```

#### ClearAll
Delete all items including pinned.

**Request**:
```json
{"version": "1.0", "cmd": "ClearAll", "args": {}, "request_id": 17}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"deleted_count": 150}, "error": null}
```

#### Import
Import clipboard items from JSON.

**Request**:
```json
{"version": "1.0", "cmd": "Import", "args": {"items": [...]}, "request_id": 18}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"imported_count": 50, "skipped_count": 2}, "error": null}
```

---

### Snippet Commands

#### ListSnippets
Get all snippets.

**Request**:
```json
{"version": "1.0", "cmd": "ListSnippets", "args": {}, "request_id": 19}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "snippets": [
      {"id": 1, "name": "greeting", "content": "Hello, world!", "updated_at": "2026-06-08T10:00:00Z"}
    ]
  },
  "error": null
}
```

#### UpsertSnippet
Create or update a snippet.

**Request**:
```json
{"version": "1.0", "cmd": "UpsertSnippet", "args": {"name": "greeting", "content": "Hello, world!"}, "request_id": 20}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"id": 1, "name": "greeting", "content": "Hello, world!", "updated_at": "2026-06-08T10:30:00Z"}, "error": null}
```

#### DeleteSnippet
Delete a snippet.

**Request**:
```json
{"version": "1.0", "cmd": "DeleteSnippet", "args": {"id": 1}, "request_id": 21}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"deleted_id": 1}, "error": null}
```

---

### Config Commands

#### GetConfig
Get current configuration.

**Request**:
```json
{"version": "1.0", "cmd": "GetConfig", "args": {}, "request_id": 22}
```

**Response**:
```json
{
  "version": "1.0",
  "ok": true,
  "data": {
    "max_items": 100,
    "max_item_size": 1048576,
    "ttl_seconds": 604800,
    "cleanup_interval_seconds": 300,
    "keyboard_shortcut": "Super+V",
    "encrypt_sensitive": false,
    "clear_on_lock": true,
    "dedup_window_seconds": 2,
    "mime_denylist": ["application/x-kde-cutselection"],
    "content_denylist": [],
    "picker": {
      "default_mode": "external",
      "default_source": "history",
      "max_results": 50,
      "show_sensitive_previews": false,
      "confirm_sensitive_copy": true,
      "close_after_copy": true,
      "prefer_quick_paste": false,
      "width": 720,
      "height": 520
    }
  },
  "error": null
}
```

#### UpdateConfig
Update configuration values.

**Request**:
```json
{"version": "1.0", "cmd": "UpdateConfig", "args": {"max_items": 200, "dedup_window_seconds": 5}, "request_id": 23}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"updated_keys": ["max_items", "dedup_window_seconds"]}, "error": null}
```

---

### Subscription Commands

#### Subscribe
Subscribe to live update events.

**Request**:
```json
{"version": "1.0", "cmd": "Subscribe", "args": {"events": ["ItemAdded", "ItemDeleted", "PinToggled"]}, "request_id": 24}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"subscription_id": 1}, "error": null}
```

**Push events** (daemon sends to client):
```json
{"version": "1.0", "type": "event", "event": "ItemAdded", "data": {"item": {...}}}
```

#### Unsubscribe
Unsubscribe from events.

**Request**:
```json
{"version": "1.0", "cmd": "Unsubscribe", "args": {"subscription_id": 1}, "request_id": 25}
```

**Response**:
```json
{"version": "1.0", "ok": true, "data": {"unsubscribed_id": 1}, "error": null}
```

---

**Last Updated**: Phase 15