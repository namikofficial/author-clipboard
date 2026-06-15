# Review Checklist: Unified GTK4 UI

---

## Bug fixes

- [x] US-001: Esc always closes popup; clears search on first press
- [x] US-002: popup opens with list focused, not search
- [x] US-003: CLI launches real `AdwApplicationWindow`, not a 520×700 pane

## Architecture

- [x] `crates/ui-gtk/` exists and is the only place widget code lives
- [ ] `crates/applet/src/main.rs` ≤ 100 LOC (currently 154 — T016 target still above target)
- [ ] `crates/hypr-picker/src/main.rs` ≤ 50 LOC (currently 97 — T017 target still above target)
- [x] `shared::picker` has the new `PickerFilter` enum
- [x] `ctl picker` exposes `--filter` and uses the new enum
- [x] `just verify` is green
- [x] No libcosmic dep remains in the workspace

## Visual (manual — requires running the app)

- [ ] 14 design tokens defined in `style.css`
- [ ] 22 SVGs in `assets/icons/`
- [ ] Soft radii (12-16px) on cards
- [ ] 150ms ease-out transitions on hover/select
- [ ] Custom scrollbar (8px wide)
- [ ] Light + dark theme parity (`AdwStyleManager`)
- [ ] Empty states use `AdwStatusPage` with custom SVG

## Functionality (manual — requires running the app)

- [ ] All shortcuts from US-005 work in popup and manager
- [ ] `Ctrl+1..9` quick-pick works
- [ ] `?` opens shortcuts overlay
- [ ] Sensitive reveal works (manager only, 5s countdown)
- [x] Filter survives popup→manager (GSettings — confirmed by schema)
- [ ] IPC `Copy` / `Pin` / `Delete` / `Star` / `Snippets` all work
- [ ] `super+shift+v` binding still triggers the popup

## Quality

- [x] No `#![allow(clippy::all)]`
- [ ] No `anyhow::Error` in public APIs
- [ ] No raw sensitive data in logs (manual audit)
- [ ] All public items have doc comments (`///`)
- [x] Conventional commit messages (`feat(ui):`, `refactor(ui):`)
- [x] `pre-023-ui-rewrite` git tag exists for rollback

## Security

- [x] Sensitive content never rendered in list (uses `redacted_preview`)
- [x] Reveal is explicit, time-boxed, manager-only
- [x] IPC socket permissions unchanged
- [ ] No sensitive content in any `tracing` log (manual audit)
- [ ] No sensitive content in any new error path (manual audit)
- [x] HTML preview sandboxed (WebContext)

## Accessibility (manual — requires running the app)

- [ ] Every interactive widget is focusable
- [ ] Every interactive widget has a label (`AdwButtonContent` etc.)
- [ ] Tab order matches visual order
- [x] Esc key behavior is consistent (unit-tested)
- [ ] Color contrast meets WCAG AA in both light and dark

## Performance (manual — requires benchmarks)

- [ ] Popup cold start < 150ms
- [ ] Manager cold start < 300ms
- [ ] List scroll 60fps with 1000 items
- [ ] Memory < 80MB (manager, 1000 items)
- [ ] No re-layout storm on filter change

## Documentation

- [x] `docs/UI.md` exists with tokens, shortcuts, widget catalog
- [x] `README.md` has inline popup and manager screenshots generated from the real GTK app
- [x] 6 PNGs in `docs/UI/snapshots/` generated via `just ui-smoke`
- [x] `docs/HYPRLAND.md` updated with new `author-clipboard` binary
- [x] `CHANGELOG.md` updated under `[Unreleased]`

## Packaging (maintainer task; not automated)

- [x] `packaging/arch/PKGBUILD` builds locally through `makepkg`
- [ ] `packaging/debian/control` builds (CI green)
- [ ] `flake.nix` builds (CI green)
- [x] `.SRCINFO` regenerated and committed
- [ ] Nix dev shell has `gtk4` and `glib`

## Final sign-off

- [x] All 20 tasks marked complete in `06-task-plan.md`
- [ ] All 7 categories above have all boxes ticked
- [x] `git tag pre-023-ui-rewrite` still exists
- [ ] `git log --oneline -1` is on the merge commit of this PR

---

**Last Updated**: 2026-06-15
