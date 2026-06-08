# Task Plan: Production Tooling

> Atomic, independently verifiable tasks for production tooling.

---

## T001: CLI Tool

**Goal**: Implement complete CLI with all commands

**Files to Edit**:
- `crates/ctl/src/main.rs`

**Verification**:
```bash
author-clipboard-ctl --help
author-clipboard-ctl toggle
author-clipboard-ctl ping
author-clipboard-ctl status
author-clipboard-ctl history --limit 10
```

---

## T002: Systemd Service

**Goal**: Create systemd service file

**Files to Create**:
- `packaging/systemd/author-clipboard-daemon.service`

**Verification**:
```bash
systemctl --user daemon-reload
systemctl --user enable --now author-clipboard-daemon
systemctl --user status author-clipboard-daemon
```

---

## T003: Doctor Command

**Goal**: Implement comprehensive doctor command

**Files to Edit**:
- `crates/ctl/src/main.rs`

**Verification**:
```bash
author-clipboard-ctl doctor
```

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Complete | Part of existing implementation |
| T002 | Complete | Part of existing implementation |
| T003 | Complete | Part of existing implementation |

**Note**: This feature is implemented in v0.5.0.

---

**Last Updated**: Phase 15 (Updated from draft)