# Requirements: Clipboard History

> Requirements captured from Phase 1 implementation.

---

## User Stories

### US-001: Persistent Clipboard History
**As a** user
**I want to** access clipboard items from earlier in my session
**So that** I can retrieve content that was overwritten

**Acceptance Criteria**:
- Given I have copied 20+ items, when I open the picker, then I see all items in reverse chronological order
- Given I restart the daemon, when I open the picker, then my history is preserved

### US-002: Search Clipboard History
**As a** user
**I want to** search my clipboard history by content
**So that** I can find specific items quickly

**Acceptance Criteria**:
- Given I type "password" in the search box, then I see only items containing "password"
- Given I clear the search, then I see all items again

### US-003: Pin Important Items
**As a** user
**I want to** pin items so they are not auto-deleted
**So that** important content is preserved

**Acceptance Criteria**:
- Given I pin an item, when cleanup runs, then pinned items are not deleted
- Given I restart the daemon, when I open the picker, then pinned items are still pinned

### US-004: Delete Items
**As a** user
**I want to** delete individual items
**So that** I can remove unwanted history

**Acceptance Criteria**:
- Given I select an item and press Delete, then the item is removed
- Given I press Ctrl+D, then the selected item is removed

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | SQLite storage with WAL mode | Must | Crash-safe |
| FR-002 | FTS5 full-text search | Must | With LIKE fallback |
| FR-003 | Pin/unpin items | Must | Persisted in DB |
| FR-004 | Delete single items | Must | Via UI or CLI |
| FR-005 | Auto-cleanup with max_items | Must | Configurable |
| FR-006 | TTL-based expiry | Must | Per-item TTL |
| FR-007 | Content deduplication | Must | Hash-based |
| FR-008 | Per-item size limits | Must | max_item_size config |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | Database queries | < 10ms | With indexes |
| NFR-002 | Memory footprint | < 50MB | Typical usage |
| NFR-003 | Startup time | < 200ms | Cold start |

---

## Edge Cases

| Case | Handling |
|------|----------|
| Empty clipboard | Show empty state UI |
| Very large content | Skip if > max_item_size |
| Duplicate content | Bump existing item within dedup_window |
| Expired items | Delete during cleanup interval |

---

## Out of Scope

- Cloud sync
- Multi-device sync
- OCR for images
- X11 clipboard monitoring

---

**Last Updated**: Phase 1 Complete