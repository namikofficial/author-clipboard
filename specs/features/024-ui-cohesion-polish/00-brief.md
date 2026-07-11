# Feature Brief: UI Cohesion & Dynamic Polish

> Numbering designation: **024A**. The directory name is retained for stable
> links; `024-wayland-command-center` is **024B** and maps to roadmap phase 25.

> Make the current GTK4 UI feel intentionally designed end-to-end:
> cleaner hierarchy, tighter spacing, stronger motion language, more
> legible states, and a more premium manager/popup experience.

## Problem Statement

Feature 023 is functionally complete, but the UI still reads as a set of
correct widgets rather than a single polished product. Spacing, visual
weight, state feedback, and responsive behavior need one cohesive pass.

## Proposed Solution

Run a UI-only polish pass across the popup, manager, preview, and shared
widgets. The goal is to make the app feel deliberate, calm, and modern
without changing the clipboard workflow or the public IPC surface.

## Goals

- Unify spacing, radii, shadows, and icon sizing across the shell
- Improve visual hierarchy in popup and manager views
- Make empty, loading, hover, selected, and redacted states feel polished
- Add restrained motion so the UI feels responsive instead of static
- Preserve keyboard-first behavior and accessibility

## Out of Scope

- No IPC changes
- No storage, database, or daemon behavior changes
- No feature expansion beyond UI cohesion and polish
- No redesign of the underlying navigation model

**Status**: Planned
