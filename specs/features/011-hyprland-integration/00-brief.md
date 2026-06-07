# Feature Brief: Hyprland Integration

> External menu picker via wofi/rofi/fuzzel, first-party GTK4 layer-shell native picker, and shared picker module.

---

## Problem Statement

Hyprland users need clipboard picker access but don't have COSMIC applet support. Two picker modes are provided: external menu and first-party native picker.

## Proposed Solution

External menu picker pipes items to dmenu-style apps (wofi, fuzzel, rofi). First-party native picker is a standalone GTK4 layer-shell popup with full keyboard navigation.

## Goals

- `author-clipboard-ctl picker` with auto-detected backend
- `author-clipboard-hypr-picker` standalone GTK4 popup
- Shared `shared::picker` module for both UIs
- `author-clipboard-ctl hyprland-config` for keybind help
- Picker configuration in `config.json`

---

**Created**: Phase 19 planning
**Status**: Implemented v0.5.0