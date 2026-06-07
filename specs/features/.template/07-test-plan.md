# Test Plan: {feature-name}

> Test strategy and test cases.

---

## Test Strategy

| Test Type | Location | Coverage Target |
|-----------|----------|-----------------|
| Unit tests | Same file as code | 80%+ public APIs |
| Integration tests | `tests/` in crate | Critical paths |
| Manual tests | N/A | UI/UX validation |

---

## Unit Tests

### Test: content_processor_handles_new_type

```rust
#[test]
fn content_processor_handles_new_type() {
    let content = b"test content";
    let result = process_content(content, ContentType::NewType);
    assert!(result.is_ok());
}
```

### Test: sensitive_detection_{pattern}

```rust
#[test]
fn sensitive_detection_api_key() {
    let content = "ghp_xxxxxxxxxxxx";
    assert!(is_sensitive(content));
}
```

---

## Integration Tests

### Test: full_capture_restore_cycle

```rust
#[tokio::test]
async fn full_capture_restore_cycle() {
    // 1. Simulate clipboard capture
    // 2. Store item
    // 3. Restore item
    // 4. Verify content matches
}
```

---

## Manual Test Checklist

- [ ] New feature works as expected
- [ ] Keyboard navigation works
- [ ] No regressions in existing features
- [ ] Edge cases handled gracefully

---

## Test Data

| Type | Example | Expected Result |
|------|---------|-----------------|
| Normal text | "Hello, world!" | Stored as text |
| Sensitive | "sk-xxxxx" | Flagged as sensitive |
| Large content | 1MB+ data | Respected max_item_size |

---

**Last Updated**: {date}