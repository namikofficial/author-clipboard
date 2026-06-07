# Technical Design: {feature-name}

> Implementation approach and technical decisions.

---

## Overview

Brief summary of the implementation approach.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/shared/src/types.rs` | Add types |
| `crates/shared/src/db/` | Database changes |
| `crates/clipboard-daemon/src/` | Daemon changes |
| `crates/applet/src/` | UI changes |
| `crates/ctl/src/` | CLI changes |

---

## Implementation Details

### Module: module_name

```rust
// Pseudocode for key logic
fn process_content(content: &[u8]) -> Result<ProcessedContent> {
    // Step 1: detect type
    // Step 2: validate
    // Step 3: process
    // Step 4: return
}
```

---

## Security Considerations

- [ ] Sensitive data handled correctly
- [ ] No data exposure in logs
- [ ] Input validation on all boundaries
- [ ] IPC permissions checked

---

## Error Handling

| Error Condition | Handling Strategy |
|-----------------|-------------------|
| Invalid input | Return `INVALID_ARG`, log debug |
| Storage failure | Return `INTERNAL_ERROR`, log error |
| Protocol unavailable | Graceful degradation |

---

## Performance Considerations

- Database queries use indexes
- Content processing is async
- UI updates are diff-based (no full refresh)

---

## Testing Strategy

See `07-test-plan.md` for detailed test cases.

---

## Migration Strategy

If database changes are needed:
1. Add new migration
2. Write upgrade path
3. Test downgrade path
4. Document in decisions if complex

---

**Last Updated**: {date}