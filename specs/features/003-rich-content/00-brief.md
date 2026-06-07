# Feature Brief: Rich Content Support

> Support for images, HTML, and file URI lists in clipboard history.

---

## Problem Statement

Users copy more than just text - images, formatted HTML, and file selections. These content types should be captured, stored, and restorable.

## Proposed Solution

Detect content type from MIME data, store appropriately (text as text, images as files), generate thumbnails for UI display, and restore with correct MIME type.

## Goals

- Capture and restore images with correct MIME type
- HTML formatting preserved when pasting
- File selections captured with metadata
- Thumbnails for quick visual identification

---

**Created**: Phase 3 Complete
**Status**: Implemented v0.5.0