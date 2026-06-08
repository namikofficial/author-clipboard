# Technical Design: Clipboard History

> Implementation approach for the core clipboard history feature.

---

## Overview

Clipboard history is the core feature. Items are captured via Wayland, stored in SQLite, and made available via IPC to CLI and UI clients.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/shared/src/types.rs` | ClipboardItem struct, hash functions |
| `crates/shared/src/db.rs` | Database operations |
| `crates/shared/src/sensitive.rs` | Sensitivity detection |
| `crates/clipboard-daemon/src/main.rs` | Capture flow |
| `crates/shared/src/picker.rs` | Shared picker logic |
| `crates/ctl/src/main.rs` | CLI history command |
| `crates/applet/src/main.rs` | History tab UI |

---

## Implementation Details

### Capture Flow (daemon)

```rust
// In clipboard-daemon/src/main.rs

impl Dispatch<ZwlrDataControlDeviceV1, ()> for AppState {
    fn event(&mut self, _: &ZwlrDataControlDeviceV1, event: Event, _: &(), conn: &Connection, _: &QueueHandle<Self>) {
        match event {
            Event::Selection(offer) => {
                // 1. Check incognito
                if self.config.is_incognito() { return; }

                // 2. Read content from offer
                let content = read_offer_content(&offer, conn)?;

                // 3. Check MIME denylist
                if self.config.is_mime_denied(&mime) { return; }

                // 4. Check content denylist
                if self.config.is_content_denied(&content) { return; }

                // 5. Detect content type
                let content_type = detect_content_type(&mime, &content);

                // 6. Compute SHA-256 hash
                let hash = hash_content(&content);

                // 7. Check sensitivity
                let sensitive = is_sensitive(&content).is_sensitive;

                // 8. Create ClipboardItem
                let item = ClipboardItem::new_text(content);

                // 9. Insert (with dedup window check)
                db.insert_or_bump(&item, self.config.dedup_window_seconds)?;

                // 10. Broadcast to subscribers
                self.broadcast(ItemAdded, &item);
            }
        }
    }
}
```

### Search Flow (db.rs)

```rust
pub fn search(&self, query: &str, limit: usize) -> SqlResult<Vec<ClipboardItem>> {
    // Try FTS5 first
    let fts_query = query
        .split_whitespace()
        .map(|w| format!("\"{}\"*", w.replace('"', "")))
        .join(" ");

    if let Ok(items) = self.search_fts(&fts_query, limit) {
        if !items.is_empty() {
            return Ok(items);
        }
    }

    // Fallback to LIKE
    self.search_like(&format!("%{query}%"), limit)
}
```

### Cleanup Flow (daemon)

```rust
fn run_cleanup(state: &AppState) {
    let db = &state.db;
    let config = &state.config;

    // Delete expired items
    let before = chrono::Utc::now() - chrono::Duration::seconds(config.ttl_seconds as i64);
    let deleted = db.delete_expired(&before)?;

    // Enforce max items
    let trimmed = db.enforce_max_items(config.max_items)?;

    // Trim audit log
    db.trim_audit_log(1000)?;

    tracing::info!("Cleanup: {} expired, {} over limit", deleted, trimmed);
}
```

---

## Performance Considerations

- **Hash computation**: SHA-256 is fast enough for clipboard content
- **FTS5**: Provides fast prefix search without LIKE's table scan
- **Virtualized list**: Only renders visible items (applet/hypr-picker)
- **Connection pooling**: Single DB connection, multiple read transactions

---

## Testing

1. Unit tests for hash functions
2. Unit tests for dedup logic
3. Integration tests for capture flow
4. Performance tests with 1000+ items

---

**Last Updated**: Phase 15 (Updated from draft)