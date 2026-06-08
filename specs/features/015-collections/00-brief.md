# Feature Brief: Collections, Pinning, and Starring

> A premium clipboard organization system with three distinct concepts: pin (never auto-purge), star (rank higher in recents), and collection (named boards for grouping).

---

## Problem Statement

Current "pin" is the only organization tool and it conflates two different needs:
1. "Keep this item forever" (pin = never auto-purge)
2. "This is important, show it higher" (star = rank higher)
3. "Group these related items together" (collection = named board)

Users also want named collections like "deploy commands", "DB queries", "prompts", "links" to organize their clipboard history.

## Proposed Solution

Split the current "pinned" concept into three distinct features:
- **Pin**: Item is never auto-deleted, shown in a dedicated Pinned section
- **Star**: Item ranks higher in recents, shown with a star indicator
- **Collection**: Items are grouped into named boards/collections

Each operates independently. An item can be pinned, starred, in a collection, all three, or none.

## Goals

- Pin/Unpin items to prevent auto-deletion
- Star/Unstar items to boost their ranking
- Create, manage, and populate named collections
- Collections are persistent and survive restarts
- Quick-access to pinned items and starred items
- Collection view shows all items in that collection

## Non-Goals

- Sharing collections between users
- Automatic collection suggestions based on content
- Collection-based auto-deletion rules

## Stakeholders

Users who want to organize their clipboard history beyond simple chronological listing, especially developers with frequently used snippets, commands, and configurations.

---

**Created**: Phase 15 (Post-Research)
**Status**: Draft