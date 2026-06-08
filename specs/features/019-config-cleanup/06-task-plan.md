# Task Plan: Configuration Cleanup

> Atomic, independently verifiable tasks for configuration cleanup.

---

## T001: Rename field in config.rs

**Goal**: Rename content_regex_denylist to content_denylist with serde alias

**Files to Edit**:
- `crates/shared/src/config.rs`

**Implementation**:
- Add `content_denylist` field
- Add `alias = "content_regex_denylist"` for migration
- Add `ContentPatternMode` enum
- Update `is_content_denied` to use pattern mode

**Verification**:
```bash
cargo test -p author-clipboard-shared -- config
just verify
```

---

## T002: Update is_content_denied logic

**Goal**: Support pattern mode in content matching

**Files to Edit**:
- `crates/shared/src/config.rs`

**Implementation**:
- Update is_content_denied to switch on pattern mode
- Add tests for each pattern mode

**Verification**:
```bash
cargo test -p author-clipboard-shared -- content_denylist
```

---

## T003: Update CLI config display

**Goal**: Show new field names in CLI config output

**Files to Edit**:
- `crates/ctl/src/main.rs`

**Verification**:
```bash
author-clipboard-ctl config | grep content
```

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Completed | Config content_denylist with serde alias for backward compat |
| T002 | Completed | ContentPatternMode enum: AllowList, BlockList, None |
| T003 | Completed | just verify passes |

---

**Last Updated**: Phase 16