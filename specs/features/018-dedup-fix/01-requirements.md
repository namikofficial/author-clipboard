# Requirements: Deduplication Fix

> Requirements for fixing the dedup implementation to match the architecture spec.

---

## User Stories

### US-001: SHA-256 Hashing
**As a** user
**I want to** have content hashed with SHA-256 (as documented)
**So that** the hash is cryptographically reliable

**Acceptance Criteria**:
- Given content "hello world", when it is hashed, then the hash is SHA-256("hello world") = `b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9`
- Given an image is hashed, when it is stored, then the hash is SHA-256 of the raw bytes
- Given I restart the daemon, when I copy the same content, then the hash is identical

### US-002: Dedup Window Enforcement
**As a** user
**I want to** copy the same content at different times and have both stored
**So that** I can track when I copied something

**Acceptance Criteria**:
- Given I copy "hello" at T=0, and copy "hello" at T=5 (within 2s window), then only one item exists (bumped)
- Given I copy "hello" at T=0, and copy "hello" at T=10 (outside 2s window), then two items exist
- Given I have two identical items at different times, when I search, then both appear

### US-003: API Compatibility
**As a** user or developer
**I want to** have the fix be transparent (no API changes)
**So that** existing code and configurations continue to work

**Acceptance Criteria**:
- Given the hash field is still u64, when I read items from the database, then the API is unchanged
- Given I have existing items with DefaultHasher hashes, when I search, then those items are still found
- Given I add new items with SHA-256 hashes, when I search, then new items work correctly

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | SHA-256 hashing for text content | Must | |
| FR-002 | SHA-256 hashing for HTML content | Must | |
| FR-003 | SHA-256 hashing for file lists | Must | |
| FR-004 | SHA-256 hashing for images | Must | |
| FR-005 | Dedup window check in insert_or_bump | Must | Use has_recent_duplicate |
| FR-006 | Configurable dedup_window_seconds | Must | Default 2s |
| FR-007 | API compatibility (u64 hash field) | Must | No schema change |

---

## Technical Details

### Hash Computation Change

**Before** (DefaultHasher):
```rust
pub fn hash_content(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}
```

**After** (SHA-256):
```rust
use sha2::{Sha256, Digest};

pub fn hash_content(content: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    // Use first 8 bytes as u64 (endianness doesn't matter for dedup)
    u64::from_le_bytes(result[0..8].try_into().unwrap())
}
```

### insert_or_bump Change

**Before** (no window check):
```rust
pub fn insert_or_bump(&self, item: &ClipboardItem) -> SqlResult<i64> {
    if let Some(existing_id) = self.find_by_hash(item.content_hash)? {
        // Bump timestamp on duplicate
        self.conn.execute(
            "UPDATE clipboard_items SET timestamp = ?1 WHERE id = ?2",
            (item.timestamp.to_rfc3339(), existing_id),
        )?;
        Ok(existing_id)
    } else {
        self.insert_item(item)
    }
}
```

**After** (with window check):
```rust
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

---

## Dependencies

- `sha2` crate (add to workspace dependencies)
- Feature `001-clipboard-history` (existing)

---

**Last Updated**: Phase 15