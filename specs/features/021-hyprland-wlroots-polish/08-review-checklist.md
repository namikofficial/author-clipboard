# Review Checklist: Hyprland-native UX & wlroots Polish

> Pre-merge review criteria for Phase 19 polish.

---

## Code Quality

- [ ] `just verify` passes (fmt → clippy → test → build)
- [ ] No clippy warnings (workspace `pedantic` lint)
- [ ] Rustfmt applied
- [ ] `shellcheck` clean on `contrib/waybar/clipboard.sh`
- [ ] No new `unwrap()` / `expect()` without justification in shared code
- [ ] Public items in `shared::picker` have rustdoc (already in place)

## Functionality

- [ ] `author-clipboard-ctl status --json` returns the documented payload
- [ ] `status --json` works when daemon is down (graceful degradation)
- [ ] `status --json` masks `last_preview` when `sensitive_last == true`
- [ ] Waybar module renders text / tooltip / class / alt
- [ ] Waybar `class: down` styling fires when daemon is down
- [ ] AUR PKGBUILD installs all four binaries + service + desktop + icon
- [ ] `packaging/arch/.SRCINFO` is in sync with PKGBUILD
- [ ] `flake.nix` exposes `default`, `applet`, `daemon`, `ctl`,
      `hypr-picker`, `apps.default`, `devShells.default`
- [ ] `docs/HYPRLAND.md` has a `## Demo` section
- [ ] `README.md` references the new sections

## Documentation

- [ ] `docs/HYPRLAND.md` describes Waybar module installation
- [ ] `contrib/waybar/README.md` documents the install / use flow
- [ ] `PROJECT_PLAN.md` marks Phase 19 complete
- [ ] Spec files in `specs/features/021-hyprland-wlroots-polish/` are
      up to date with shipped behavior

## Security

- [ ] Waybar script only reads status — no clipboard writes
- [ ] `last_preview` is masked when item is sensitive
- [ ] IPC socket path unchanged (private, not world-writable)

## Process

- [ ] One task = one commit (or one PR)
- [ ] Commit messages follow Conventional Commits
- [ ] No unrelated refactors

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
