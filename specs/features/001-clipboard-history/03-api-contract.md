# API Contract: Clipboard History

> IPC protocol and CLI commands for clipboard history operations.

---

## CLI Commands (ctl crate)

### history

List recent clipboard items.

```bash
author-clipboard-ctl history [OPTIONS]

Options:
  --limit <count>     Number of items to show (default: 10)
  --json              Output machine-readable JSON
  --pretty            Output formatted text (default)
```

**Output (JSON)**:
```json
{
  "items": [
    {
      "id": 1,
      "content_hash": "0x1234abcd",
      "content": "hello world",
      "mime_type": "text/plain",
      "content_type": "text",
      "timestamp": "2026-06-08T10:30:00Z",
      "pinned": false,
      "starred": false,
      "source_app": "kitty",
      "sensitive": false,
      "plain_text": null
    }
  ],
  "total": 50,
  "offset": 0,
  "limit": 10
}
```

### copy

Copy a history item by ID.

```bash
author-clipboard-ctl copy <id> [OPTIONS]

Options:
  --mode <mode>       copy, quick-paste, copy-plain-text, copy-redacted
  --confirm           Confirm for sensitive items
```

**Output**:
```json
{
  "success": true,
  "id": 42,
  "mime_type": "text/plain",
  "behavior": "copied"
}
```

### clear

Clear all unpinned items.

```bash
author-clipboard-ctl clear [OPTIONS]

Options:
  --include-pinned    Also clear pinned items (requires --confirm)
  --confirm           Required for --include-pinned
```

**Output**:
```json
{
  "success": true,
  "deleted_count": 138
}
```

### export

Export clipboard history to JSON.

```bash
author-clipboard-ctl export [OPTIONS]

Options:
  --output <path>     Output file (default: stdout)
  --format <fmt>      json (default)
```

### search

Search clipboard history (Phase 14).

```bash
author-clipboard-ctl search <query> [OPTIONS]

Options:
  --type <type>       text, image, html, files
  --pinned            Show only pinned
  --sensitive          Show only sensitive
  --age <seconds>      Items from last N seconds
```

---

## IPC Commands (daemon)

### History

**Request**:
```json
{"cmd": "History", "args": {"limit": 50, "offset": 0, "filters": null}}
```

**Response**:
```json
{"ok": true, "data": {"items": [...], "total": 150, "offset": 0, "limit": 50, "has_more": true}}
```

### GetItem

**Request**:
```json
{"cmd": "GetItem", "args": {"id": 42}}
```

**Response**:
```json
{"ok": true, "data": {"item": {...}}}
```

### Copy

**Request**:
```json
{"cmd": "Copy", "args": {"id": 42, "mode": "Copy", "confirm_sensitive": false}}
```

**Response**:
```json
{"ok": true, "data": {"success": true, "mime_type": "text/plain", "behavior": "copied"}}
```

### Delete

**Request**:
```json
{"cmd": "Delete", "args": {"id": 42}}
```

**Response**:
```json
{"ok": true, "data": {"deleted_id": 42}}
```

### ClearUnpinned

**Request**:
```json
{"cmd": "ClearUnpinned", "args": {}}
```

**Response**:
```json
{"ok": true, "data": {"deleted_count": 138}}
```

---

## Error Codes

| Code | Meaning |
|------|---------|
| ITEM_NOT_FOUND | Requested item ID doesn't exist |
| SENSITIVE_CONFIRMATION_REQUIRED | Item is sensitive, confirm=true required |
| CONFIRMATION_REQUIRED | Destructive operation requires confirm=true |
| DAEMON_NOT_RUNNING | Cannot connect to daemon |
| INVALID_ARG | Invalid argument value |

---

**Last Updated**: Phase 15 (Updated from draft)