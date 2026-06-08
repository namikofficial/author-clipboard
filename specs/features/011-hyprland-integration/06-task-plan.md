# Task Plan: Hyprland Integration

> Atomic, independently verifiable tasks for Hyprland integration.

---

## T001: Hypr-Picker Binary

**Goal**: Create standalone GTK4 picker binary

**Files to Create**:
- `crates/hypr-picker/Cargo.toml`
- `crates/hypr-picker/src/main.rs`

**Verification**:
```bash
cargo build -p author-clipboard-hypr-picker
./target/debug/author-clipboard-hypr-picker --help
```

---

## T002: Layer Shell Integration

**Goal**: Implement layer-shell overlay on Hyprland

**Files to Edit**:
- `crates/hypr-picker/src/main.rs`

**Verification**:
```bash
# Manual test: open picker, verify it appears as overlay
```

---

## T003: IPC Integration

**Goal**: Connect to daemon via IPC for items

**Files to Edit**:
- `crates/hypr-picker/src/main.rs`

**Verification**:
```bash
# Manual test: verify items load from daemon
```

---

## T004: Hyprland Config Generator

**Goal**: Add hyprland-config CLI command

**Files to Edit**:
- `crates/ctl/src/main.rs`

**Verification**:
```bash
author-clipboard-ctl hyprland-config
```

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Complete | Part of existing implementation |
| T002 | Complete | Part of existing implementation |
| T003 | Complete | Part of existing implementation |
| T004 | Complete | Part of existing implementation |

**Note**: This feature is implemented in v0.5.0.

---

**Last Updated**: Phase 15 (Updated from draft)