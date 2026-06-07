# Feature Brief: Clipboard History

> Persistent clipboard history with SQLite storage, full-text search, pin/delete, and auto-cleanup.

---

## Problem Statement

Users lose clipboard content when applications close or content is overwritten. There is no persistent history to search or retrieve previously copied items.

## Proposed Solution

A daemon monitors the Wayland clipboard and stores all content in a local SQLite database. Users can search, pin, and restore items through a picker UI.

## Goals

- Never lose clipboard data across app closures
- Instant access via global shortcut
- Rich content support (text, images, HTML, files)
- Search and filter history
- Pin important items

## Non-Goals

- Cloud sync (deferred to Phase 17)
- OCR for images (deferred to Phase 15)
- X11 support (deferred to Phase 16)

## Stakeholders

All COSMIC/Hyprland/Sway users who want persistent clipboard history.

---

**Created**: Phase 1 Complete (May 2026)
**Status**: Implemented v0.5.0