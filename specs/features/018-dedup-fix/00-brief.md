# Feature Brief: Deduplication Fix

> Fix the critical correctness bug where `insert_or_bump` uses `DefaultHasher` instead of SHA-256, and the dedup window is not properly enforced.

---

## Problem Statement

The architecture document and decisions specify SHA-256 for content hashing and a `dedup_window_seconds` time window for deduplication. The current implementation:

1. Uses `DefaultHasher` (fast but not cryptographic) instead of SHA-256
2. `insert_or_bump` bumps any item with matching hash regardless of time window
3. `has_recent_duplicate` checks the time window but is not used by the main insertion path

This is a spec violation and a correctness bug.

## Proposed Solution

1. Replace `DefaultHasher` with SHA-256 (using `sha2` crate)
2. Change `insert_or_bump` to use `has_recent_duplicate` before bumping
3. Ensure dedup window is enforced at insert time

## Goals

- SHA-256 hash computed for all content types
- Dedup window enforced: identical content within window bumps, outside window creates new entry
- API remains backward compatible (same `content_hash` field, different computation)
- Database migration not needed (hash value stored, computation change is transparent)

## Non-Goals

- Changing the hash field type (u64 is fine, just change how it's computed)
- Adding new database columns
- Supporting multiple hash algorithms

## Stakeholders

All users, as this affects core clipboard behavior.

---

**Created**: Phase 15 (Post-Research)
**Status**: Draft