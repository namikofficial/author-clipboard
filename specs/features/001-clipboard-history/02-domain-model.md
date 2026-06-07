# Domain Model: Clipboard History

> Data structures, state, and relationships for the core clipboard history feature.

---

## Data Structures

### ClipboardItem

```rust
// In shared/src/types.rs

pub struct ClipboardItem {
    pub id: i64,                      // Unique identifier (auto-increment)
    pub content_hash: u64,            // SHA-256 hash of content (for dedup)
    pub content: String,              // Text content or image path
    pub mime_type: String,            // MIME type (e.g., "text/plain")
    pub content_type: ContentType,    // Enum: Text, Image, Html, Files
    pub timestamp: DateTime<Utc>,    // When item was captured
    pub pinned: bool,                // Never auto-delete
    pub starred: bool,               // Priority ranking (Phase 15)
    pub source_app: Option<String>,   // Which app copied this
    pub sensitive: bool,             // Contains sensitive data
    pub plain_text: Option<String>,  // For HTML: searchable plain text
    pub ttl_override: Option<u64>,   // Per-item TTL in seconds (NULL = global)
}

pub enum ContentType {
    Text,   // Plain text
    Image,  // Binary image stored as file
    Html,   // HTML with plain_text fallback
    Files,  // File URI list
}
```

### Database Schema

```sql
CREATE TABLE clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_hash INTEGER NOT NULL,       -- SHA-256 (u64)
    content TEXT NOT NULL,              -- Text or image path
    mime_type TEXT NOT NULL,            -- MIME type
    content_type TEXT NOT NULL DEFAULT 'text',
    timestamp TEXT NOT NULL,            -- RFC3339
    pinned INTEGER NOT NULL DEFAULT 0,
    starred INTEGER NOT NULL DEFAULT 0, -- Phase 15
    source_app TEXT,
    sensitive INTEGER NOT NULL DEFAULT 0,
    plain_text TEXT,                    -- For HTML search
    ttl_override INTEGER DEFAULT NULL   -- Per-item TTL
);

-- Indexes for fast queries
CREATE INDEX idx_timestamp ON clipboard_items(timestamp DESC);
CREATE INDEX idx_content_hash ON clipboard_items(content_hash);
CREATE INDEX idx_pinned ON clipboard_items(pinned);
CREATE INDEX idx_content_type ON clipboard_items(content_type);

-- FTS5 for full-text search
CREATE VIRTUAL TABLE clipboard_fts USING fts5(
    content,
    plain_text,
    content='clipboard_items',
    content_rowid='id'
);
```

---

## State Machine

### Item Lifecycle

```
[Clipboard Capture]
        |
        v
[Content Detection] --> [Is MIME denied?] --> Yes --> [Skip]
        |
        No
        v
[Is Content Denied?] --> Yes --> [Skip]
        |
        No
        v
[Hash Content (SHA-256)]
        |
        v
[Is Duplicate in Window?] --> Yes --> [Bump timestamp]
        |
        No
        v
[Sensitivity Detection]
        |
        v
[Insert into DB]
        |
        v
[Notify Subscribers]
```

### Cleanup Lifecycle

```
[Cleanup Interval]
        |
        v
[Delete Expired Items] --> [TTL exceeded, not pinned]
        |
        v
[Enforce Max Items] --> [Delete oldest non-pinned]
        |
        v
[Trim Audit Log]
```

---

## Relationships

### ClipboardItem -> Collection (Many-to-Many via Membership)

```
ClipboardItem 1..* -- CollectionMembership -- * Collection
```

### ClipboardItem -> Snippets (Independent)

Snippets are separate from clipboard history. They can reference items but don't own them.

---

## Key Operations

### Dedup Logic

```rust
fn should_bump(item: &ClipboardItem, window_seconds: u64) -> bool {
    if let Some(existing) = db.find_by_hash(item.content_hash) {
        if db.has_recent_duplicate(item.content_hash, window_seconds) {
            return true;  // Bump existing
        }
    }
    false  // Insert new
}
```

### Search Logic

```rust
fn search(query: &str) -> Vec<ClipboardItem> {
    // 1. Try FTS5 for prefix matching
    // 2. Fallback to LIKE if FTS fails
    // 3. Order by pinned DESC, timestamp DESC
}
```

---

**Last Updated**: Phase 15 (Updated from draft)