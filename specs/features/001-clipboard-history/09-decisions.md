# Decisions: Clipboard History

> Key decisions made during Phase 1 implementation.

---

## D001: SQLite with WAL Mode

**Date**: Phase 1
**Status**: Accepted

**Context**:
Needed a persistent storage solution that supports concurrent access, crash safety, and fast queries.

**Decision**:
Use `rusqlite` with `PRAGMA journal_mode=WAL` for crash-safe concurrent access.

**Consequences**:
- Positive: Crash-safe, concurrent reads during writes
- Positive: Good performance for our use case
- Negative: Requires bundled SQLite compilation

---

## D002: FTS5 for Full-Text Search

**Date**: Phase 1
**Status**: Accepted

**Context**:
Users need to search clipboard history by content.

**Decision**:
SQLite FTS5 virtual table with LIKE fallback for compatibility.

**Consequences**:
- Positive: Fast text search with ranking
- Positive: LIKE fallback handles edge cases
- Negative: FTS table adds storage overhead

---

## D003: Hash-Based Deduplication

**Date**: Phase 1
**Status**: Accepted

**Context**:
Don't want duplicate entries for the same content copied in quick succession.

**Decision**:
Compute SHA-256 hash of content. If a hash exists within `dedup_window_seconds`, bump the existing item's timestamp instead of creating a new entry.

**Consequences**:
- Positive: Prevents duplicate entries
- Positive: Simple and fast
- Negative: Content must be fully in memory to hash

---

## D004: Per-Item TTL

**Date**: Phase 1
**Status**: Accepted

**Context**:
Some items (like OTPs) should expire quickly while others should persist longer.

**Decision**:
Each item has an optional `expires_at` timestamp. Cleanup task checks and deletes expired items.

**Consequences**:
- Positive: Fine-grained retention control
- Positive: Pinned items can have separate TTL
- Negative: More complex cleanup logic

---

**Last Updated**: Phase 1 Complete