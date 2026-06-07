# Technical Design: Deduplication Fix

> Implementation approach for fixing the deduplication to use SHA-256 and respect the dedup window.

---

## Overview

The fix involves:
1. Adding `sha2` crate to workspace dependencies
2. Changing `hash_content` and `hash_bytes` to use SHA-256
3. Changing `insert_or_bump` to check dedup window before bumping

---

## Affected Files

| File | Change |
|------|--------|
| `Cargo.toml` | Add `sha2` workspace dependency |
| `crates/shared/src/types.rs` | Change hash functions to SHA-256 |
| `crates/shared/src/db.rs` | Change insert_or_bump to use dedup window |
| `crates/clipboard-daemon/src/main.rs` | Pass dedup_window_seconds to insert_or_bump |
| `specs/features/001-clipboard-history/09-decisions.md` | Update decision to reflect actual implementation |

---

## Implementation Details

### 1. Add sha2 dependency

```toml
# Cargo.toml (workspace)
[workspace.dependencies]
sha2 = "0.10"
```

### 2. Update types.rs

```rust
// In crates/shared/src/types.rs

use sha2::{Sha256, Digest};

// Change hash_content to use SHA-256
pub fn hash_content(content: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    // Use first 8 bytes as u64
    u64::from_le_bytes(result[0..8].try_into().unwrap())
}

// Change hash_bytes to use SHA-256
pub fn hash_bytes(data: &[u8]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    u64::from_le_bytes(result[0..8].try_into().unwrap())
}
```

### 3. Update db.rs

```rust
// In crates/shared/src/db.rs

// Change insert_or_bump signature to accept dedup_window_seconds
pub fn insert_or_bump(&self, item: &ClipboardItem, dedup_window_seconds: u64) -> SqlResult<i64> {
    // Check if there's a recent duplicate within the dedup window
    if let Some(existing_id) = self.find_by_hash(item.content_hash)? {
        if self.has_recent_duplicate(item.content_hash, dedup_window_seconds)? {
            // Within window: bump timestamp
            self.conn.execute(
                "UPDATE clipboard_items SET timestamp = ?1 WHERE id = ?2",
                (item.timestamp.to_rfc3339(), existing_id),
            )?;
            return Ok(existing_id);
        }
    }
    // Outside window or no duplicate: insert new item
    self.insert_item(item)
}
```

### 4. Update daemon to pass dedup_window_seconds

```rust
// In crates/clipboard-daemon/src/main.rs

// When calling insert_or_bump:
let item_id = db.insert_or_bump(&item, state.config.dedup_window_seconds)?;
```

---

## Testing

### Unit Tests

1. SHA-256 hash produces expected values
2. Same content produces same hash across restarts
3. Different content produces different hashes
4. Dedup window correctly identified (within/outside)

### Integration Tests

1. Copy same content within window -> one item (bumped)
2. Copy same content outside window -> two items
3. Copy different content -> two items

---

## Migration

### Database Compatibility

The hash field is stored as u64 in the database. Since we're changing the computation method (not the storage type), no database migration is needed. Existing items will have DefaultHasher hashes; new items will have SHA-256 hashes. This is fine because:
- Dedup only works for items with the same hash algorithm
- Old items won't dedup with new items (different hash), which is acceptable
- Over time, old items expire and are cleaned up

### Configuration Compatibility

The `dedup_window_seconds` config field is unchanged. The fix just ensures it's actually used.

---

## Security Considerations

- [x] SHA-256 is cryptographically secure (unlike DefaultHasher)
- [x] Hash output is still u64 (no sensitive data exposure)
- [x] No new attack surface introduced

---

**Last Updated**: Phase 15