# Feature Brief: Quick Paste

> Type selected text directly into applications using wtype/ydotool.

---

## Problem Statement

Copying to clipboard and then manually pasting is a two-step process. Users want a single action to select an item and have it typed into the active application.

## Proposed Solution

Detect available quick-paste tools (wtype preferred, ydotool fallback), provide opt-in toggle in settings, show appropriate UI indicators, and handle security warnings for input permissions.

## Goals

- Single-action paste that types text directly
- Backend detection (wtype/ydotool/wl-copy)
- Security warnings for input permissions
- Works across applications

---

**Created**: Phase 5 Complete
**Status**: Implemented v0.5.0