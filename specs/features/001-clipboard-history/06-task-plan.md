# Task Plan: Clipboard History

> Atomic, independently verifiable tasks for the clipboard history feature.

---

## T001: Database Schema

**Goal**: Ensure database schema supports all required fields

**Files to Edit**:
- `crates/shared/src/db.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- db_schema
```

---

## T002: Capture Flow

**Goal**: Implement full clipboard capture via Wayland

**Files to Edit**:
- `crates/clipboard-daemon/src/main.rs`

**Verification**:
```bash
# Manual test: copy text in a terminal, verify it appears in picker
```

---

## T003: Search Implementation

**Goal**: Implement FTS5 search with LIKE fallback

**Files to Edit**:
- `crates/shared/src/db.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- search
```

---

## T004: Pin/Unpin

**Goal**: Implement pin/unpin functionality

**Files to Edit**:
- `crates/shared/src/db.rs`
- `crates/shared/src/ipc.rs`
- `crates/ctl/src/main.rs`
- `crates/applet/src/main.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- pin
```

---

## T005: Delete

**Goal**: Implement single-item deletion

**Files to Edit**:
- `crates/shared/src/db.rs`
- `crates/shared/src/ipc.rs`

**Verification**:
```bash
cargo test -p author-clipboard-shared -- delete
```

---

## T006: Cleanup Task

**Goal**: Implement TTL-based cleanup and max_items enforcement

**Files to Edit**:
- `crates/clipboard-daemon/src/main.rs`

**Verification**:
```bash
# Wait for cleanup interval, verify expired items are deleted
```

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Complete | Part of existing implementation |
| T002 | Complete | Part of existing implementation |
| T003 | Complete | Part of existing implementation |
| T004 | Complete | Part of existing implementation |
| T005 | Complete | Part of existing implementation |
| T006 | Complete | Part of existing implementation |

**Note**: This feature is implemented in v0.5.0. This task plan documents the implementation for reference.

---

**Last Updated**: Phase 15 (Updated from draft)