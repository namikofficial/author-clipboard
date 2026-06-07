# Task Plan: Deduplication Fix

> Atomic, independently verifiable tasks for fixing the deduplication implementation.

---

## T001: Add sha2 dependency

**Goal**: Add sha2 crate to workspace dependencies

**Files to Edit**:
- `Cargo.toml`

**Implementation**:
- Add `sha2 = "0.10"` to workspace dependencies

**Verification**:
```bash
cargo search sha2
cargo update
```

**Rollback Risk**: Low — adding dependency only

---

## T002: Update hash functions to SHA-256

**Goal**: Change hash_content and hash_bytes to use SHA-256

**Files to Edit**:
- `crates/shared/src/types.rs`

**Implementation**:
- Import sha2::{Sha256, Digest}
- Update hash_content to use Sha256::new()
- Update hash_bytes to use Sha256::new()
- Add test for known SHA-256 value

**Verification**:
```bash
cargo test -p author-clipboard-shared -- hash
just verify
```

**Rollback Risk**: Low — changing implementation, tests verify correctness

---

## T003: Update insert_or_bump to check dedup window

**Goal**: Change insert_or_bump to enforce dedup_window_seconds

**Files to Edit**:
- `crates/shared/src/db.rs`

**Implementation**:
- Change insert_or_bump signature to accept dedup_window_seconds parameter
- Check has_recent_duplicate before bumping
- Update callers in daemon

**Verification**:
```bash
cargo test -p author-clipboard-shared -- insert_or_bump
cargo test -p author-clipboard-daemon
just verify
```

**Rollback Risk**: Medium — changes insert behavior

---

## T004: Update decision log

**Goal**: Document the actual implementation in decisions.md

**Files to Edit**:
- `specs/features/001-clipboard-history/09-decisions.md`

**Implementation**:
- Add decision noting that SHA-256 is now implemented
- Note that DefaultHasher was used in initial implementation

**Verification**:
```bash
# Just documentation update
```

**Rollback Risk**: None — documentation only

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Completed | hash_content function uses sha2::Sha256 |
| T002 | Completed | dedup window enforced in insert_or_bump |
| T003 | Completed | content_hash column uses TEXT not INTEGER |
| T004 | Completed | Tests pass with just verify |

---

**Last Updated**: Phase 16