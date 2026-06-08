# Test Plan: Clipboard History

> Test strategy and test cases for the clipboard history feature.

---

## Test Strategy

| Test Type | Location | Coverage Target |
|-----------|----------|-----------------|
| Unit tests | Same file as code | 80%+ public APIs |
| Integration tests | `tests/` in crate | Critical paths |
| Manual tests | N/A | UI/UX validation |

---

## Unit Tests

### Hash Functions

```rust
#[test]
fn test_hash_content_deterministic() {
    let content = "hello world";
    let hash1 = hash_content(content);
    let hash2 = hash_content(content);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_hash_content_sha256() {
    // Known SHA-256 value
    let hash = hash_content("hello world");
    let expected = 0xb94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9u64;
    assert_eq!(hash, expected);
}

#[test]
fn test_hash_bytes() {
    let data = b"hello world";
    let hash1 = hash_bytes(data);
    let hash2 = hash_bytes(data);
    assert_eq!(hash1, hash2);
}
```

### Database Operations

```rust
#[test]
fn test_insert_and_query() {
    let db = Database::open_in_memory().unwrap();
    let item = ClipboardItem::new_text("test".to_string());
    let id = db.insert_item(&item).unwrap();
    let retrieved = db.get_by_id(id).unwrap().unwrap();
    assert_eq!(retrieved.content, "test");
}

#[test]
fn test_dedup_within_window() {
    let db = Database::open_in_memory().unwrap();
    let item1 = ClipboardItem::new_text("duplicate".to_string());
    db.insert_or_bump(&item1, 2).unwrap();  // 2s window

    // Within window: should bump
    let item2 = ClipboardItem::new_text("duplicate".to_string());
    let id2 = db.insert_or_bump(&item2, 2).unwrap();
    assert_eq!(db.get_recent(10).unwrap().len(), 1);
}

#[test]
fn test_search() {
    let db = Database::open_in_memory().unwrap();
    db.insert_item(&ClipboardItem::new_text("hello world".to_string())).unwrap();
    let results = db.search("hello", 10).unwrap();
    assert_eq!(results.len(), 1);
}
```

### Sensitivity Detection

```rust
#[test]
fn test_sensitive_api_key() {
    let result = check_sensitivity("ghp_xxxxxxxxxxxx");
    assert!(result.is_sensitive);
}

#[test]
fn test_sensitive_password() {
    let result = check_sensitivity("password=secret123");
    assert!(result.is_sensitive);
}

#[test]
fn test_not_sensitive_normal_text() {
    let result = check_sensitivity("hello world");
    assert!(!result.is_sensitive);
}
```

---

## Integration Tests

### Full Capture Flow

```rust
#[tokio::test]
async fn test_capture_and_restore() {
    // 1. Start daemon
    // 2. Simulate clipboard copy via Wayland
    // 3. Verify item in database
    // 4. Restore item
    // 5. Verify clipboard content
}
```

### Dedup Behavior

```rust
#[tokio::test]
async fn test_dedup_within_window() {
    // 1. Copy "test" at T=0
    // 2. Copy "test" at T=1 (within 2s window)
    // 3. Verify only 1 item
}

#[tokio::test]
async fn test_dedup_outside_window() {
    // 1. Copy "test" at T=0
    // 2. Copy "test" at T=3 (outside 2s window)
    // 3. Verify 2 items
}
```

---

## Manual Test Checklist

- [ ] Copy text from terminal, verify appears in picker
- [ ] Copy image, verify thumbnail appears
- [ ] Copy HTML, verify plain text is searchable
- [ ] Copy sensitive content, verify detection
- [ ] Pin item, verify it survives cleanup
- [ ] Delete item, verify removal
- [ ] Search for text, verify results
- [ ] Restart daemon, verify history persists

---

**Last Updated**: Phase 15 (Updated from draft)