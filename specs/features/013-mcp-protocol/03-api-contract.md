# API Contract: MCP Protocol Server

> Complete MCP tool, resource, and prompt definitions for the clipboard MCP server.

---

## MCP Server Configuration

### stdio Transport (Codex, OpenCode local)

```json
// Codex: ~/.config/codex.json
{
  "mcpServers": {
    "author-clipboard": {
      "command": "author-clipboard-mcp",
      "args": ["--transport", "stdio"]
    }
  }
}
```

```json
// OpenCode: ~/.config/opencode/config.json
{
  "mcp": {
    "author-clipboard": {
      "type": "local",
      "command": ["author-clipboard-mcp", "--transport", "stdio"],
      "enabled": true
    }
  }
}
```

HTTP transport is not implemented. The shipped server accepts local stdio only.

---

## Tools

### clipboard.search

Search clipboard history with filtering and pagination.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Search query (full-text search)"
    },
    "limit": {
      "type": "number",
      "description": "Maximum results to return (default: 50, max: 200)"
    },
    "offset": {
      "type": "number",
      "description": "Offset for pagination"
    },
    "content_type": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Filter by content type: text, image, html, files"
    },
    "pinned": {
      "type": "boolean",
      "description": "Filter by pinned state"
    },
    "sensitive": {
      "type": "boolean",
      "description": "Filter by sensitive flag"
    },
    "source_app": {
      "type": "string",
      "description": "Filter by source application (e.g., kitty, firefox)"
    },
    "age_max_seconds": {
      "type": "number",
      "description": "Show only items from last N seconds"
    }
  },
  "required": ["query"]
}
```

**Output**:
```json
{
  "items": [
    {
      "id": 42,
      "content_hash": "0xabc123",
      "preview": "AWS_ACCESS_KEY_ID=••••••••",
      "mime_type": "text/plain",
      "content_type": "text",
      "timestamp": "2026-06-08T10:30:00Z",
      "pinned": false,
      "source_app": "kitty",
      "sensitive": true
    }
  ],
  "total": 5,
  "has_more": false
}
```

---

### clipboard.get

Get a single clipboard item by ID.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "number",
      "description": "Clipboard item ID"
    },
    "include_content": {
      "type": "boolean",
      "description": "Include full content (default: false, masked by default)"
    },
    "confirm_sensitive": {
      "type": "boolean",
      "description": "Required per request when include_content exposes a sensitive item"
    }
  },
  "required": ["id"]
}
```

Search, resources, and prompt payloads always pass through the MCP redaction
boundary regardless of `picker.show_sensitive_previews`. Confirmation is never
remembered between requests. Errors use stable uppercase `code` values and do
not echo clipboard content.

**Output**:
```json
{
  "item": {
    "id": 42,
    "content_hash": "0xabc123",
    "content": "••••••••",  // masked unless include_content=true
    "mime_type": "text/plain",
    "content_type": "text",
    "timestamp": "2026-06-08T10:30:00Z",
    "pinned": false,
    "source_app": "kitty",
    "sensitive": true,
    "plain_text": "••••••••",
    "preview": "AWS_ACCESS_KEY_ID=••••••••"
  }
}
```

---

### clipboard.copy

Copy an item to the Wayland clipboard.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "number",
      "description": "Clipboard item ID to copy"
    },
    "mode": {
      "type": "string",
      "enum": ["copy", "quick_paste", "copy_plain_text", "copy_redacted"],
      "description": "Copy mode (default: copy)"
    },
    "confirm_sensitive": {
      "type": "boolean",
      "description": "Required if item is sensitive"
    }
  },
  "required": ["id"]
}
```

**Output**:
```json
{
  "success": true,
  "mime_type": "text/plain",
  "behavior": "copied",
  "sensitive_confirmed": false
}
```

**Error Response** (sensitive without confirmation):
```json
{
  "error": {
    "code": "SENSITIVE_CONFIRMATION_REQUIRED",
    "message": "This item contains sensitive data. Set confirm_sensitive=true to copy."
  }
}
```

---

### clipboard.pin

Pin an item so it won't be auto-deleted.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "number",
      "description": "Clipboard item ID to pin"
    }
  },
  "required": ["id"]
}
```

**Output**:
```json
{
  "success": true,
  "id": 42,
  "pinned": true
}
```

---

### clipboard.unpin

Unpin an item.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "number",
      "description": "Clipboard item ID to unpin"
    }
  },
  "required": ["id"]
}
```

**Output**:
```json
{
  "success": true,
  "id": 42,
  "pinned": false
}
```

---

### clipboard.delete

Delete a single clipboard item.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "number",
      "description": "Clipboard item ID to delete"
    },
    "confirm": {
      "type": "boolean",
      "description": "Must be true to confirm deletion"
    }
  },
  "required": ["id", "confirm"]
}
```

**Output**:
```json
{
  "success": true,
  "deleted_id": 42
}
```

**Error Response** (confirm not set):
```json
{
  "error": {
    "code": "CONFIRMATION_REQUIRED",
    "message": "Set confirm=true to delete item."
  }
}
```

---

### clipboard.clear_unpinned

Delete all non-pinned items.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "confirm": {
      "type": "boolean",
      "description": "Must be true to confirm"
    }
  },
  "required": ["confirm"]
}
```

**Output**:
```json
{
  "success": true,
  "deleted_count": 138
}
```

---

### clipboard.list_snippets

List all user-defined snippets.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {}
}
```

**Output**:
```json
{
  "snippets": [
    {
      "id": 1,
      "name": "greeting",
      "content": "Hello, world!",
      "updated_at": "2026-06-08T10:00:00Z"
    }
  ]
}
```

---

### clipboard.upsert_snippet

Create or update a snippet.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "Snippet name (unique identifier)"
    },
    "content": {
      "type": "string",
      "description": "Snippet content"
    }
  },
  "required": ["name", "content"]
}
```

**Output**:
```json
{
  "success": true,
  "snippet": {
    "id": 1,
    "name": "greeting",
    "content": "Hello, world!",
    "updated_at": "2026-06-08T10:30:00Z"
  }
}
```

---

### clipboard.delete_snippet

Delete a snippet.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "number",
      "description": "Snippet ID to delete"
    },
    "confirm": {
      "type": "boolean",
      "description": "Must be true to confirm"
    }
  },
  "required": ["id", "confirm"]
}
```

**Output**:
```json
{
  "success": true,
  "deleted_id": 1
}
```

---

### clipboard.export

Export clipboard history as JSON.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "format": {
      "type": "string",
      "enum": ["json"],
      "description": "Export format (default: json)"
    },
    "limit": {
      "type": "number",
      "description": "Maximum items to export (default: all)"
    }
  }
}
```

**Output**:
```json
{
  "exported_count": 150,
  "format": "json",
  "data": "[...]"  // JSON string of ClipboardItem array
}
```

---

### clipboard.stats

Get database statistics.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {}
}
```

**Output**:
```json
{
  "total_items": 150,
  "pinned_items": 12,
  "total_size_bytes": 1048576,
  "oldest_item": "2026-06-01T10:00:00Z",
  "newest_item": "2026-06-08T10:30:00Z"
}
```

---

## Resources

### clipboard://recent

List recent clipboard items.

**URI**: `clipboard://recent?limit=50&offset=0&content_type=text&pinned=false`

**Read Response**:
```json
{
  "items": [...],
  "total": 150,
  "limit": 50,
  "offset": 0
}
```

---

### clipboard://item/{id}

Get a specific clipboard item.

**URI Template**: `clipboard://item/{id}`

**Read Response**:
```json
{
  "item": {...}
}
```

---

### clipboard://pins

Get all pinned items.

**URI**: `clipboard://pins`

**Read Response**:
```json
{
  "items": [...],
  "total": 12
}
```

---

### clipboard://snippets

Get all snippets.

**URI**: `clipboard://snippets`

**Read Response**:
```json
{
  "snippets": [...]
}
```

---

### clipboard://stats

Get database statistics.

**URI**: `clipboard://stats`

**Read Response**:
```json
{
  "total_items": 150,
  "pinned_items": 12,
  "total_size_bytes": 1048576,
  "capture_rate_per_hour": 45.2
}
```

---

### clipboard://audit/recent

Get recent audit events.

**URI**: `clipboard://audit/recent?limit=50`

**Read Response**:
```json
{
  "events": [
    {
      "id": 1000,
      "event_kind": "item_copied",
      "details": "{\"id\": 42, \"sensitive\": true}",
      "timestamp": "2026-06-08T10:30:00Z"
    }
  ]
}
```

---

## Prompts

### clipboard:summarize_recent

Summarize recent clipboard items.

**Arguments**:
- `limit` (optional, number): Number of items to summarize (default: 10)
- `content_type` (optional, string): Filter by content type

**Template**:
```
Summarize the last {limit} clipboard items. For each item, note:
- Content type (text/image/html/files)
- Whether it was pinned
- Whether it contained sensitive data
- A brief preview of the content

Format as a concise list.
```

---

### clipboard:promote_to_snippet

Create a snippet from recent clipboard history.

**Arguments**: None

**Template**:
```
Look at the most recent clipboard item that would make a good snippet.
Create a new snippet with:
- name: a descriptive name based on the content
- content: the clipboard item's content

Ask the user to confirm the name and content before saving.
```

---

### clipboard:find_pattern

Find clipboard items matching a pattern.

**Arguments**:
- `pattern` (required, string): Search pattern

**Template**:
```
Search clipboard history for items matching: {pattern}

Return a list of matching items with:
- ID
- Content preview (first 100 chars)
- Timestamp
- Pinned status

If no matches found, suggest related searches.
```

---

### clipboard:redact_sensitive

Redact sensitive data from clipboard items.

**Arguments**:
- `id` (required, number): Item ID to redact

**Template**:
```
Take the clipboard item with ID {id} and create a redacted copy.
Replace all sensitive patterns (API keys, passwords, tokens) with •••.
Return the redacted content that can be safely shared.
```

---

## Server Instructions

The MCP server should return the following instructions during initialization:

```
Server Instructions for author-clipboard:

- Prefer masked previews in search results to protect sensitive data
- Always confirm destructive operations (delete, clear) with the user
- Use list results instead of full content to minimize token cost
- For sensitive items, set confirm_sensitive=true when copying
- Pin important items to prevent auto-deletion
- Search supports full-text queries with filters for type, age, and source app
```

---

**Last Updated**: Phase 15
