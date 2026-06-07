# Task Plan: {feature-name}

> Atomic, independently verifiable tasks.

---

## Task Dependencies

```
T001 ─┬─ T003 ── T005
      │
T002 ─┴─ T004 ── T005
```

---

## T001: {task title}

**Goal**: {what this task accomplishes}

**Files to Edit**:
- `crates/shared/src/types.rs`

**Implementation**:
- Add `NewType` struct
- Add `impl NewType` with methods

**Verification**:
```bash
cargo test -p author-clipboard-shared -- new_type
just verify
```

**Rollback Risk**: Low — adding new code only

---

## T002: {task title}

**Goal**: {what this task accomplishes}

**Files to Edit**:
- `crates/shared/src/db/migrations.rs`

**Implementation**:
- Add migration for new table

**Verification**:
```bash
cargo test -p author-clipboard-shared -- migration
```

**Rollback Risk**: Medium — schema change

---

## T003: {task title}

**Goal**: {what this task accomplishes}

**Files to Edit**:
- `crates/clipboard-daemon/src/processor.rs`

**Implementation**:
- Handle new content type

**Verification**:
```bash
cargo test -p author-clipboard-daemon -- processor
just verify
```

**Rollback Risk**: Low

---

## T004: {task title}

**Goal**: {what this task accomplishes}

**Files to Edit**:
- `crates/applet/src/ui.rs`

**Implementation**:
- Add UI for new feature

**Verification**:
```bash
cargo test -p author-clipboard-applet
just verify
```

**Rollback Risk**: Medium — UI changes

---

## T005: Integration Test

**Goal**: End-to-end verification

**Files to Edit**: None (integration test only)

**Implementation**:
- Test full flow: capture → store → restore

**Verification**:
```bash
cargo test --all
just verify
```

**Rollback Risk**: N/A

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | | |
| T002 | | |
| T003 | | |
| T004 | | |
| T005 | | |

---

**Last Updated**: {date}