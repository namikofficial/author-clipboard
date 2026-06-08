# Feature Brief: Dotfiles Production Integration

> Integrate author-clipboard into the production Hyprland/dotfiles environment at https://github.com/namikofficial/dotfiles for daily use.

---

## Problem Statement

The dotfiles repo already has a clipboard workflow using cliphist with rofi. author-clipboard is intended to replace this, but:
- Existing keybinds reference cliphist scripts
- Daemon startup is handled by custom scripts
- The environment has specific expectations around keyboard flow and visual style
- There's an existing AI helper workflow that should integrate with clipboard

## Proposed Solution

Two-stage migration:
1. **Stage 1**: Replace cliphist internals with author-clipboard, keep keybinds
2. **Stage 2**: Update keybinds to first-party author-clipboard commands

This minimizes breakage risk on the production machine.

## Goals

- author-clipboard replaces cliphist as the clipboard backend
- Existing Hyprland keybinds (Super+Ctrl+V, Super+Shift+V) continue working
- rofi remains the default menu backend (matches existing setup)
- daemon health checks via dev-health integration
- Settings hub integration for clipboard configuration
- AI helper workflow enhanced with clipboard context

## Non-Goals

- Removing cliphist package (may be needed by other tools)
- Supporting non-Hyprland compositors in dotfiles
- Waybar integration (separate feature)

## Stakeholders

Primary: namik (production user)
Secondary: anyone adopting author-clipboard with similar Hyprland/dotfiles setup

---

**Created**: Phase 15 (Post-Research)
**Status**: Draft