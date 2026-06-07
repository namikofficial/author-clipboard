# Feature Brief: Hyprland-native UX & wlroots Polish

> Waybar/Wayle status module, AUR + Nix flakes, and Hyprland demo content for
> the Hyprland/wlroots polish work that closes out Phase 19.

---

## Problem Statement

Phase 19 (`Hyprland-native UX & wlroots polish`) shipped a first-party Hyprland
picker (`author-clipboard-hypr-picker`), an external menu picker, shared
`shared::picker` logic, a Hyprland config generator, layer-shell popup mode,
and the `picker` config section. To make the Hyprland/wlroots story feel
first-class without overclaiming native COSMIC parity, the project also needs:

- A lightweight status indicator that a Hyprland user can drop into a bar
  (Waybar / Wayle / ags / any bar that reads JSON or runs a script) so the
  picker actually shows up in their normal workflow.
- Distribution artifacts that make installing the picker on Hyprland-target
  systems (Arch via AUR, NixOS via the flake) one command.
- Discoverable demo material so users can see the picker in action before
  installing.

## Proposed Solution

Three small additions layered on top of the Phase 19 work already shipped:

1. A Waybar/Wayle module under `contrib/waybar/` (script + JSON config) that
   reads a single command — `author-clipboard-ctl status --json` — and renders
   the daemon's running state, the last-captured item's type, and a count of
   pinned / unpinned items.
2. Polish the existing AUR PKGBUILD + `.SRCINFO` template in
   `packaging/arch/` and the existing `flake.nix` / `default.nix` so they build
   all four binaries (daemon, applet, ctl, hypr-picker) and the GTK4 / layer
   shell runtime dependencies, with CI validation that `.SRCINFO` is in sync
   with the PKGBUILD.
3. A `Demo` section in `docs/HYPRLAND.md` that documents the recommended
   screencast flow and provides a deterministic text-based "demo" (a recorded
   shell session) for users who want to evaluate without watching a GIF.

## Goals

- Drop-in Waybar / Wayle module that works without changes to the daemon.
- `author-clipboard-ctl status --json` exposing structured data for any
  bar / panel.
- AUR PKGBUILD + `.SRCINFO` ready to push; CI fails if they drift.
- Nix flake exposes `daemon`, `applet`, `ctl`, `hypr-picker`, and a dev
  shell.
- HYPRLAND.md has a "Demo" section describing a reproducible flow and a
  static demo (text transcript / ASCII layout).
- Phase 19 marked complete in `PROJECT_PLAN.md`.

## Non-Goals

- Replacing the COSMIC applet (libcosmic stays the primary UI on COSMIC).
- A real animated GIF / video file (the sandboxed build environment cannot
  capture screen content; the demo is the documented flow + transcript).
- Multi-protocol transport (X11) — still Phase 16.
- Snippets, OCR, sync — still later phases.

## Stakeholders

- Hyprland users installing via AUR.
- NixOS users installing via the flake.
- Waybar / Wayle / ags / any bar users who want a clipboard indicator.
- Sway users (same module works; bar configuration is composable).
- Maintainers cutting the next release (`v0.6.0`).

---

**Created**: 2026-06-08
**Status**: Draft (Phase 19 polish)
