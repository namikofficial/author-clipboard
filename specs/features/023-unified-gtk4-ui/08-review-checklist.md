# Review Checklist: Unified GTK4 UI

---

## Bug fixes

- [ ] US-001: Esc always closes popup; clears search on first press
- [ ] US-002: popup opens with list focused, not search
- [ ] US-003: CLI launches real `AdwApplicationWindow`, not a 520×700 pane

## Architecture

- [ ] `crates/ui-gtk/` exists and is the only place widget code lives
- [ ] `crates/applet/src/main.rs` ≤ 100 LOC
- [ ] `crates/hypr-picker/src/main.rs` ≤ 50 LOC
- [ ] `shared::picker` has the new `PickerFilter` enum
- [ ] `ctl picker` exposes `--filter` and uses the new enum
- [ ] `just verify` is green
- [ ] No libcosmic dep remains in the workspace

## Visual

- [ ] 14 design tokens defined in `style.css`
- [ ] 22 SVGs in `assets/icons/`
- [ ] Soft radii (12-16px) on cards
- [ ] 150ms ease-out transitions on hover/select
- [ ] Custom scrollbar (8px wide)
- [ ] Light + dark theme parity (`AdwStyleManager`)
- [ ] Empty states use `AdwStatusPage` with custom SVG

## Functionality

- [ ] All shortcuts from US-005 work in popup and manager
- [ ] `Ctrl+1..9` quick-pick works
- [ ] `?` opens shortcuts overlay
- [ ] Sensitive reveal works (manager only, 5s countdown)
- [ ] Filter survives popup→manager (GSettings)
- [ ] IPC `Copy` / `Pin` / `Delete` / `Star` / `Snippets` all work
- [ ] `super+shift+v` binding still triggers the popup

## Quality

- [ ] No `#![allow(clippy::all)]`
- [ ] No `anyhow::Error` in public APIs
- [ ] No raw sensitive data in logs
- [ ] All public items have doc comments (`///`)
- [ ] Conventional commit messages (`feat(ui):`, `refactor(ui):`)
- [ ] `pre-023-ui-rewrite` git tag exists for rollback

## Security

- [ ] Sensitive content never rendered in list (uses `redacted_preview`)
- [ ] Reveal is explicit, time-boxed, manager-only
- [ ] IPC socket permissions unchanged
- [ ] No sensitive content in any `tracing` log
- [ ] No sensitive content in any new error path
- [ ] HTML preview sandboxed (WebContext)

## Accessibility

- [ ] Every interactive widget is focusable
- [ ] Every interactive widget has a label (`AdwButtonContent` etc.)
- [ ] Tab order matches visual order
- [ ] Esc key behavior is consistent
- [ ] Color contrast meets WCAG AA in both light and dark

## Performance

- [ ] Popup cold start < 150ms
- [ ] Manager cold start < 300ms
- [ ] List scroll 60fps with 1000 items
- [ ] Memory < 80MB (manager, 1000 items)
- [ ] No re-layout storm on filter change

## Documentation

- [ ] `docs/UI.md` exists with tokens, shortcuts, widget catalog
- [ ] `README.md` has 2 inline screenshots (popup + manager)
- [ ] 6 PNGs in `docs/UI/`
- [ ] `docs/HYPRLAND.md` updated with new `author-clipboard` binary
- [ ] `CHANGELOG.md` updated under `[Unreleased]`

## Packaging

- [ ] `packaging/arch/PKGBUILD` builds (CI green)
- [ ] `packaging/debian/control` builds (CI green)
- [ ] `flake.nix` builds (CI green)
- [ ] `.SRCINFO` regenerated and committed
- [ ] Nix dev shell has `gtk4` and `glib`

## Final sign-off

- [ ] All 20 tasks marked complete in `06-task-plan.md`
- [ ] All 7 categories above have all boxes ticked
- [ ] `git tag pre-023-ui-rewrite` still exists
- [ ] `git log --oneline -1` is on the merge commit of this PR

---

**Last Updated**: 2026-06-12
