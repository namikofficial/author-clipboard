# Test Plan: Distribution Packages & Release Artifacts (Phase 18)

> How we verify that Phase 18 work does what the spec promises.

---

## Manual Verification

| Check | Command | Expected |
|-------|---------|----------|
| `.deb` contains all binaries | `dpkg-deb -c target/debian/author-clipboard_*.deb \| grep usr/bin/author-clipboard` | Four binaries listed: applet, daemon, ctl, hypr-picker. |
| `.deb` metadata is valid | `dpkg-deb -I target/debian/author-clipboard_*.deb` | Maintainer, homepage, license, extended description populated. |
| `cargo deb` build is reproducible | `cargo deb -p author-clipboard-applet --no-build` twice | Both runs succeed with the same metadata; binary content may differ if libcosmic is rebuilt. |
| Arch PKGBUILD builds | `makepkg --nocheck --nodeps` inside `archlinux:latest` | Build completes; package written. |
| `.SRCINFO` is in sync | `makepkg --printsrcinfo > .SRCINFO.new && diff -u packaging/arch/.SRCINFO .SRCINFO.new` | Empty diff. |
| Flatpak manifest is valid YAML | `python3 -c "import yaml; yaml.safe_load(open('packaging/flatpak/com.namikofficial.author-clipboard.yml'))"` | No exception. |
| AppImage script is syntactically valid | `bash -n packaging/appimage/build.sh` | No errors. |
| Nix flake parses | `nix flake check --no-build` (when nix is installed) | No errors. |
| `just` lists new recipes | `just --list` | New `release-*`, `flatpak-*`, `appimage-*`, `nix-*` recipes appear. |
| CI yaml is valid | `yq '.jobs' .github/workflows/ci.yml` | Both `check` and `arch-pkg` jobs are defined. |
| Release yaml is valid | `yq '.jobs' .github/workflows/release.yml` | `release` job exists with deb + tar.xz + sums. |

## CI Verification (post-merge)

1. `ci.yml` runs on the PR; `arch-pkg` job is green.
2. Push a `v0.5.1-rc1` tag to a fork; `release.yml` runs.
3. Inspect the draft release on the fork; verify all artifacts are present.
4. If `GPG_PRIVATE_KEY` is configured, verify `gpg --verify SHA256SUMS.asc SHA256SUMS` succeeds.

## Out-of-Scope (Not Tested)

- AUR push (manual, requires AUR account).
- COSMIC store submission (external portal).
- Real AppImage run inside a live Wayland session (covered by manual QA in `docs/LOCAL_TESTING.md`).
- Real Flatpak install (covered by manual QA in `docs/FLATPAK.md`).

---

**Last Updated**: 2026-06-08
