# Feature Brief: Unified GTK4 UI

> One UI library, one widget tree, two windowing modes (popup + manager),
> one visual language. Replaces the libcosmic applet and the parallel
> hypr-picker with a single GTK4 + libadwaita codebase.

---

## Problem

The project ships three UIs (`applet`, `hypr-picker`, `ctl picker`),
two of which (applet popup + applet manager + hypr-picker) are parallel
implementations of the same widget set. They drift: keyboard shortcuts
differ, filter UIs differ, settings pages differ, icons differ, and the
applet's "full window" mode (used when launched from CLI) is a
520×700 libcosmic app with no proper window chrome, no headerbar, and
broken focus handling for Esc.

The result: a beautiful, privacy-focused backend wrapped in three
slightly-broken UIs that fight each other.

## Proposed Solution

Build `crates/ui-gtk/` — a single GTK4 + libadwaita UI library.
Ship two binaries from it:

- `author-clipboard` — popup by default (`--popup`), manager when
  launched from `.desktop` or with `--manager`. Same `App` struct.
- `author-clipboard-hypr-picker` — popup, calls into the same
  `ui_gtk::run_popup(...)` entry point.

The third UI (`ctl picker`) is a thin shim over `shared::picker::build_external_rows`
and stays as-is. The picker module is upgraded to match the new filter
set so the three UIs feel like one product.

## Goals

1. One widget tree, three entry points.
2. Esc always closes the popup (bug US-001).
3. Search never auto-focuses on popup open (bug US-002).
4. CLI launch opens a real manager window, not a broken 520×700 pane
   (bug US-003).
5. Cute, branded, cohesive visual identity — soft radii, custom
   icon set, micro-animations, dark/light adaptive.
6. Same keyboard shortcuts across popup and manager.
7. `just verify` green at the end.

## Non-Goals

- Replacing the external `wofi/rofi/fuzzel` picker.
- Adding a new DE's native toolkit (KDE, GNOME, Windows).
- Rewriting the daemon, IPC protocol, or DB schema.
- New clipboard features; this is a UI unification.

## Stakeholders

Users on COSMIC, Hyprland, Sway, and any wlroots compositor that
supports GTK4 layer-shell.

---

**Created**: Phase 22 (2026-06-12)
**Status**: Approved — ready for implementation
