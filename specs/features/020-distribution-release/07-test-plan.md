# Test Plan: Distribution Packages & Release Artifacts (Phase 18)

> How we verify that Phase 18 work does what the spec promises.

---

## Manual Verification

| Check | Command | Expected |
|-------|---------|----------|
| `.deb` contains all binaries | `dpkg-deb -c target/debian/author-clipboard_*.deb \| grep usr/bin/author-clipboard` | Four binaries listed: applet, daemon, ctl, hypr-picker. |
| `.deb` metadata is valid | `dpkg-deb -I target/debian/author-clipboard_*.deb` | Maintainer, homepage, license, extended description populated. |
| `cargo deb` build is reproducible | `cargo deb -p author-clipboard-applet --no-build` twice | Both runs succeed with the same metadata; binary content may differ if libcosmic is rebuilt. |
| Arch PKGBUILD builds | `makepkg --noconfirm --syncdeps --cleanbuild --clean` inside `archlinux:latest` | Build completes; package written without nested package-manager calls. |
| `.SRCINFO` is in sync | `makepkg --printsrcinfo > .SRCINFO.new && diff -u packaging/arch/.SRCINFO .SRCINFO.new` | Empty diff. |
| Flatpak manifest is valid YAML | `python3 -c "import yaml; yaml.safe_load(open('packaging/flatpak/com.namikofficial.author-clipboard.yml'))"` | No exception. |
| AppImage script is syntactically valid | `bash -n packaging/appimage/build.sh` | No errors. |
| Nix flake parses | `nix flake check --no-build` (when nix is installed) | No errors. |
| `just` lists new recipes | `just --list` | New `release-*`, `flatpak-*`, `appimage-*`, `nix-*` recipes appear. |
| CI yaml is valid | `yq '.jobs' .github/workflows/ci.yml` | Both `check` and `arch-pkg` jobs are defined. |
| Release yaml is valid | `yq '.jobs' .github/workflows/release.yml` | `release` job exists with deb + tar.xz + sums. |
| Release trigger is explicit | `yq '.on.push.tags' .github/workflows/release.yml` | Contains only the `v[0-9]*` tag pattern. |
| Release does not mutate `main` | `rg 'git push|Bump version' .github/workflows/release.yml` | No matches. |
| Version parity is enforced | Inspect release validation step | Tag, workspace version, PKGBUILD, and `.SRCINFO` must agree. |

## CI Verification (post-merge)

1. `ci.yml` runs on the PR; Rust, Debian, and Arch package jobs are green.
2. Push a disposable `vX.Y.Z` tag matching package metadata to a fork; `release.yml` runs.
3. Inspect the draft release on the fork; verify all artifacts are present.
4. If `GPG_PRIVATE_KEY` is configured, verify `gpg --verify SHA256SUMS.asc SHA256SUMS` succeeds.

## Out-of-Scope (Not Tested)

- AUR push (manual, requires AUR account).
- COSMIC store submission (external portal).
- Real AppImage run inside a live Wayland session (covered by manual QA in `docs/LOCAL_TESTING.md`).
- Real Flatpak install (covered by manual QA in `docs/FLATPAK.md`).

---

**Last Updated**: 2026-06-15
