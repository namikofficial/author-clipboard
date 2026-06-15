# Review Checklist: Distribution Packages & Release Artifacts (Phase 18)

> Pre-merge review. Each item is checked off when the reviewer confirms it.

---

## Spec Coverage

- [x] Brief, requirements, design, task plan, and test plan exist.
- [x] Each requirement (`FR-001`..`FR-013`) maps to one or more tasks in `06-task-plan.md`.
- [x] Out-of-scope items are explicitly listed in `01-requirements.md` and `05-technical-design.md`.

## Implementation Quality

- [x] `crates/applet/Cargo.toml` deb assets are workspace-relative.
- [x] `.deb` includes applet, daemon, ctl, AND hypr-picker.
- [x] `release.yml` builds the full workspace, builds the `.deb`, generates `SHA256SUMS`, and conditionally signs.
- [x] `ci.yml` builds and inspects the `.deb` on pull requests.
- [x] `ci.yml` builds the Arch package and verifies `.SRCINFO` inside `archlinux:latest`.
- [x] Ubuntu jobs build a pinned gtk4-layer-shell version unavailable from apt.
- [x] Arch jobs invoke `makepkg` through an unprivileged builder account.
- [x] `release.yml` runs only for explicit version tags and never pushes a version-bump commit.
- [x] Release validation rejects tag/package version drift before publication.
- [x] Flatpak manifest is at `packaging/flatpak/...` and references the Freedesktop 23.08 runtime.
- [x] AppImage build script is in `packaging/appimage/` with `build.sh` + `AppRun` + desktop.
- [x] `flake.nix` exposes `packages.<system>.default` and `apps.<system>.default`.
- [x] `default.nix` provides a non-flake fallback.
- [x] `docs/RELEASING.md`, `docs/COSMIC_STORE.md`, `docs/AUR.md`, `docs/FLATPAK.md` exist and are linked from `docs/PACKAGING.md`.
- [x] `justfile` has recipes for every new packaging form.
- [x] `PROJECT_PLAN.md` Phase 18 checkboxes reflect actual completion.

## Quality Gates

- [x] `just fmt-check` passes.
- [x] `just lint` passes (no new warnings on applet, shared, daemon, ctl, mcp-server).
- [x] `cargo test` for shared crate passes.
- [x] PR Rust, Debian, and Arch jobs are green for the current head SHA.

## Security & Privacy

- [x] GPG signing is opt-in; secrets are referenced via `${{ secrets.GPG_* }}` and never echoed.
- [x] The `.deb` postinst does **not** call `systemctl --user` non-interactively; it prints a hint.
- [x] The Flatpak manifest declares only the minimum required permissions (Wayland socket, xdg-data).
- [x] The AppImage build script downloads `appimagetool` with a pinned URL; checksum verification is documented but not enforced (best-effort).

## Documentation

- [x] `docs/PACKAGING.md` covers all current install paths.
- [x] `docs/RELEASING.md` is runnable: a maintainer can follow it end-to-end.
- [x] `docs/COSMIC_STORE.md` and `docs/AUR.md` describe external actions that this repo cannot perform automatically.

## Deviations from Plan

- [x] The temporary release-on-`main` and automatic patch-bump design was
  rejected because it published before an explicit maintainer action and wrote
  unreviewed commits back to the stable branch.

---

**Last Updated**: 2026-06-15
