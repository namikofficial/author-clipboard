# Feature Brief: Global Shortcut & COSMIC Integration

> Global shortcut registration and COSMIC applet integration for instant picker access.

---

## Problem Statement

Users need instant access to their clipboard history from any application. The picker must appear immediately when triggered and work across all workspaces and monitors.

## Proposed Solution

Register a global shortcut (Super+V) that triggers the picker via IPC. The COSMIC applet uses layer-shell for proper positioning and focus management.

## Goals

- Global shortcut works in all applications
- Picker appears in < 100ms
- Works across multi-monitor setups
- Proper focus handling (Escape returns focus)

## Non-Goals

- Shortcut configuration UI (requires COSMIC runtime API)
- Waybar module (deferred)

---

**Created**: Phase 2 Complete
**Status**: Implemented v0.5.0