# Feature Brief: Native Power-User Revamp

> Make author-clipboard a credible top-tier Linux clipboard manager for
> developers and power users: native-feeling, fast, visually rich,
> keyboard-first, automation-friendly, and practical across Hyprland,
> Sway, COSMIC, and generic Wayland sessions.

---

## Problem Statement

author-clipboard captures and restores clipboard history, but the product
still feels like a functional utility rather than a polished first-party
desktop tool. The current picker can list items and search, but it lacks the
depth users expect from mature clipboard managers:

- direct row actions for pin, star, delete, copy, quick-paste, and reveal
- a preview/inspector pane with rich content handling
- collection/project organization for developer workflows
- command/snippet ergonomics beyond simple history search
- native window behavior, close affordances, resize behavior, and compositor
  integration
- strong visual distinction between text, code, HTML, images, files, secrets,
  snippets, and developer artifacts
- a clear architecture for scaling from "clipboard list" to "power-user command
  center"

The user goal is not just parity with cliphist or a rofi menu. The app should
feel like a thoughtful Linux-first clipboard workstation that can compete with
premium third-party clipboard managers on macOS and Windows while staying
honest about Wayland limitations.

## Proposed Solution

Build a staged revamp around six product pillars:

1. **Native windowing and polish**: real close paths, resizable/floating window
   behavior, responsive layout, first-run guidance, consistent keyboard help,
   and compositor-specific install hints.
2. **Command-center UI**: split list + inspector, rich row cards, action rail,
   type-aware previews, status/health strip, and fast keyboard flows.
3. **Developer-first intelligence**: code/text detection, command/query
   filters, project/source context, snippet/template expansion, safe secret
   treatment, and repeatable paste workflows.
4. **Organization**: pinned items, starred priority, collections, saved filters,
   and named boards for prompts, commands, links, database snippets, and
   project-specific material.
5. **Automation and integrations**: stable IPC/CLI contracts, Waybar/Wayle
   surfaces, shell/editor hooks, JSON import/export, and extensible actions.
6. **Quality bar**: responsive under large histories, no secret leakage,
   verifiable specs, screenshots, smoke tests, and package/install parity.

This spec is an umbrella roadmap. It intentionally references and supersedes
the stale parts of earlier specs while preserving their useful detail.

## Goals

- Turn the Hyprland picker and manager into a polished native-feeling app.
- Add an item inspector with previews for text, code, HTML, images, files, and
  sensitive content.
- Add discoverable item actions and keyboard shortcuts.
- Implement collection/project organization and saved filters.
- Make snippets/templates feel first-class for developer reuse.
- Provide stable CLI/IPC contracts for automation and bar integrations.
- Ensure the install path sets up schemas, desktop entries, services, and
  compositor rules without manual post-fix work.
- Keep the security and privacy model explicit, visible, and testable.

## Non-Goals

- SaaS sync, accounts, billing, or cloud-backed history.
- Cross-device sync in this spec. Self-hosted encrypted sync remains a separate
  future feature.
- X11 parity in the main revamp. X11 fallback remains separate Phase 16 work.
- Shell command substitution inside snippets.
- Pretending Wayland exposes source-app metadata when it does not. Any
  source-app/project metadata must be collected through explicit integrations
  or documented best-effort signals.

## Stakeholders

- Hyprland/Sway users who want a native Wayland clipboard manager.
- COSMIC users who want a polished first-party-feeling manager.
- Developers who reuse commands, prompts, links, paths, database snippets, and
  project-specific text.
- Privacy-conscious users who need clear sensitive-data handling.
- Maintainers who need small, verifiable implementation slices.

## Relationship To Existing Specs

| Spec | Relationship |
|------|--------------|
| `015-collections` | Adopt and refresh under the organization pillar. |
| `016-world-class-ux` | Superseded at product level; keep preview/performance ideas. |
| `021-hyprland-wlroots-polish` | Extend from setup/support into native utility-window behavior. |
| `024-ui-cohesion-polish` | Keep as UI-only visual system sub-slice. |
| `026-snippet-templates` | Adopt as the snippet/template foundation. |

---

**Created**: 2026-06-19  
**Status**: Draft
