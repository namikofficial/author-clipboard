# Feature Brief: Database & CI Hardening

> FTS5 full-text search, per-item TTL, dedup controls, WAL mode, and GitHub Actions CI pipeline.

---

## Problem Statement

The database needs production-grade features: reliable search, crash safety, configurable retention, and proper CI to catch issues.

## Proposed Solution

SQLite FTS5 for search, WAL mode for crash safety, per-item TTL for retention control, dedup window configuration, and GitHub Actions CI pipeline.

## Goals

- FTS5 virtual table with LIKE fallback
- WAL mode for concurrent access
- Per-item TTL via `expires_at` column
- Configurable `dedup_window_seconds`
- CI: fmt → clippy → test → build

---

**Created**: Phase 12 Complete
**Status**: Implemented v0.5.0