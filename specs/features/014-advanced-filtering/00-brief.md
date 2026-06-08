# Feature Brief: Advanced Filtering & Saved Searches

> Composable filter chips, saved searches, and a powerful search grammar that replaces the current simple search with a unified, premium experience.

---

## Problem Statement

Current search is a simple text match. Users need:
- Filter by content type (text, image, html, files)
- Filter by age (today, this week, older)
- Filter by source application (kitty, firefox, etc.)
- Filter by sensitivity, pinned state, starred state
- Composable filters that can be combined
- Saved searches for common queries ("API keys copied today", "recent code blocks")

## Proposed Solution

A unified search grammar with chip-based UI that supports:
- Plain text search
- Composable filter chips (type:, age:, app:, pinned:, sensitive:, starred:)
- Saved searches (named queries stored in config)
- Real-time filter preview as you type
- Search suggestions based on history

## Goals

- Single search box handles both text search and filters
- Chip-based UI for filter building (click to add, click to remove)
- Saved searches accessible from a dropdown
- Support for complex queries: `type:text age:today app:kitty sensitive:false`
- Autocomplete for filter values
- Search history for quick access to recent queries

## Non-Goals

- Full regex support (keep it simple)
- Natural language query parsing
- Collaborative/shared searches
- Search result ranking customization

## Stakeholders

All users who need to find specific clipboard items quickly, especially developers with large histories.

---

**Created**: Phase 15 (Post-Research)
**Status**: Draft