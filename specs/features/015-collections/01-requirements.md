# Requirements: Collections, Pinning, and Starring

> Requirements for the three-tier organization system.

---

## User Stories

### US-001: Pin Items Forever
**As a** user
**I want to** pin items so they are never auto-deleted
**So that** I can preserve important content

**Acceptance Criteria**:
- Given an item, when I press Ctrl+P, then the item is pinned and shows a pin icon
- Given a pinned item, when cleanup runs, then the item is not deleted
- Given a pinned item, when I restart the daemon, then the item is still pinned

### US-002: Star Items for Priority
**As a** user
**I want to** star items so they appear higher in recent lists
**So that** I can quickly find important items

**Acceptance Criteria**:
- Given an item, when I press Ctrl+Shift+S, then the item is starred
- Given a starred item, when I view the history, then starred items appear before non-starred items of the same age
- Given a starred item, when I search, then starred items are ranked higher in results

### US-003: Create Collections
**As a** user
**I want to** create named collections to group related items
**So that** I can organize my clipboard by project or topic

**Acceptance Criteria**:
- Given I open the Collections tab, when I click "New Collection", then I am prompted for a name
- Given I enter "deploy-commands", when I click Create, then a new collection is created
- Given I have collections, when I open the picker, then I see a Collections section with all collections

### US-004: Add Items to Collections
**As a** user
**I want to** add items to collections via keyboard shortcut
**So that** I can organize items without using the mouse

**Acceptance Criteria**:
- Given an item is selected, when I press Ctrl+Shift+C, then a collection picker appears
- Given I select "prompts" from the picker, then the item is added to the prompts collection
- Given an item is in multiple collections, when I view each collection, then I see the item in both

### US-005: View Collection Contents
**As a** user
**I want to** open a collection and see all its items
**So that** I can browse items by topic

**Acceptance Criteria**:
- Given I click on a collection, when the view opens, then I see all items in that collection
- Given an item in a collection, when I delete it from the main history, then it is also removed from the collection
- Given a collection with 50+ items, when I open it, then items are paginated with virtual scrolling

### US-006: Quick Access to Pinned and Starred
**As a** user
**I want to** quickly access my pinned and starred items
**So that** I can find them without searching

**Acceptance Criteria**:
- Given I open the picker, when I press Ctrl+Shift+P, then only pinned items are shown
- Given I open the picker, when I press Ctrl+Shift+A, then only starred items are shown
- Given I press Ctrl+Shift+P again, then all items are shown

---

## Three-Tier Organization Model

### Pin (Never Auto-Purge)

- Items marked `pinned=true` are never deleted by cleanup
- Shown in dedicated "Pinned" section at top of history
- Manual unpin required to allow deletion
- Pin icon: 📌

### Star (Priority Ranking)

- Items marked `starred=true` rank higher in all views
- Shown with star icon in lists
- Can be starred/unstarred freely
- Does NOT prevent auto-deletion
- Star icon: ⭐

### Collection (Named Grouping)

- Items can belong to zero or more collections
- Collection is a named group with persistent items
- Can be created, renamed, deleted
- Deleting a collection does NOT delete its items
- Collection icon: 📁

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | Pin/Unpin items | Must | Keyboard: Ctrl+P |
| FR-002 | Star/Unstar items | Must | Keyboard: Ctrl+Shift+S |
| FR-003 | Create collection | Must | Via UI or CLI |
| FR-004 | Delete collection | Must | Items remain in history |
| FR-005 | Add item to collection | Must | Keyboard: Ctrl+Shift+C |
| FR-006 | Remove item from collection | Must | |
| FR-007 | View collection contents | Must | |
| FR-008 | Rename collection | Must | |
| FR-009 | Pinned section in picker | Must | |
| FR-010 | Starred items ranked higher | Must | |
| FR-011 | Collection count badge | Must | Show number of items |
| FR-012 | Drag-and-drop to collections | Should | |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | Collection list load | < 50ms | |
| NFR-002 | Add to collection | < 100ms | |
| NFR-003 | 100 collections supported | Must | |
| NFR-004 | 1000 items per collection | Must | |

---

## Edge Cases

| Case | Handling |
|------|----------|
| Delete item in collection | Remove from collection, keep in history |
| Delete last item in collection | Collection remains (empty) |
| Delete collection with 100 items | Items remain in history |
| Star then unstar same item | Star state toggles |
| Pin then star same item | Both states active |

---

## Out of Scope

- Collaborative/shared collections
- Auto-suggest collections based on content
- Collection-based retention rules
- Nested collections (flat only)

---

## Dependencies

- Feature `012-service-api` (required)
- Feature `016-world-class-ux` (UI implementation)

---

**Last Updated**: Phase 15