# Review Checklist: Native Power-User Revamp

> Use this checklist before marking the revamp complete or merging major slices.

---

## Product

- [ ] Picker feels like a native utility window, not a stuck overlay.
- [ ] Close button, Esc, compositor close, and copy-close all terminate cleanly.
- [ ] Main workflows are keyboard-first and mouse-discoverable.
- [ ] Item rows expose enough text and metadata to scan quickly.
- [ ] Inspector gives confidence before copy/quick-paste.
- [ ] Sensitive items are visibly protected.
- [ ] Collections/saved filters solve real developer workflows.
- [ ] Snippets/templates feel integrated, not bolted on.

## Technical

- [ ] No stale references to old `crates/applet/src/*` UI architecture in new
      implementation plans.
- [ ] IPC commands are documented and tested.
- [ ] CLI commands exist for primary UI mutations.
- [ ] DB migrations are additive and covered by tests.
- [ ] Query parser is pure and tested.
- [ ] UTF-8 truncation is char-safe.
- [ ] UI does not load heavy previews until selected.
- [ ] Package/install paths include GSettings schemas and desktop entries.

## Security

- [ ] Raw sensitive content is not logged.
- [ ] Sensitive rows/previews are redacted by default.
- [ ] Reveal is explicit and time-limited.
- [ ] Export redacts sensitive/encrypted payloads by default.
- [ ] HTML preview defaults to safe text fallback.

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all`
- [ ] `just ui-check`
- [ ] `just install`
- [ ] Native picker runtime close smoke passes.
- [ ] Unicode clipboard smoke passes.
- [ ] Screenshots refreshed with `just ui-smoke`.
- [ ] Docs updated: `README.md`, `docs/UI.md`, `docs/HYPRLAND.md`.

## Release Readiness

- [ ] `CHANGELOG.md` updated.
- [ ] AUR/deb/Nix packaging includes all installed assets.
- [ ] `PROJECT_PLAN.md` points to this spec as the current revamp plan.
- [ ] Known limitations are documented clearly.

---

**Last Updated**: 2026-06-19
