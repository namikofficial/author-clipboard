# Domain Model: Distribution Packages & Release Artifacts (Phase 18)

> Files, formats, and the build-time data model that the Phase 18 packaging work produces.

---

## New / Modified Artifacts

| Path | Type | Source / Generated | Purpose |
|------|------|---------------------|---------|
| `crates/applet/Cargo.toml` | Edited | Source | Fix `package.metadata.deb.assets` paths to be workspace-relative; add hypr-picker binary. |
| `.github/workflows/release.yml` | Edited | Source | Build full workspace, build `.deb`, generate SHA256SUMS, optional GPG sign, attach all artifacts. |
| `.github/workflows/ci.yml` | Edited | Source | Add an Arch Linux job that runs `makepkg --nocheck` against `packaging/arch/PKGBUILD`. |
| `packaging/flatpak/com.namikofficial.author-clipboard.yml` | New | Source | Flatpak manifest for `flatpak-builder`. |
| `packaging/appimage/build.sh` | New | Source | Builds an AppImage from the release binaries. |
| `packaging/appimage/AppRun.in` | New | Source | AppImage entrypoint template. |
| `packaging/appkit/AppImage.desktop.in` | New | Source | AppImage desktop entry template. |
| `flake.nix` | New | Source | Nix flake. |
| `flake.lock` | Generated | Source | Locked input versions for the Nix flake. |
| `default.nix` | New | Source | Fallback for non-flakes users. |
| `docs/RELEASING.md` | New | Source | Maintainer runbook: tag, sign, verify, publish. |
| `docs/COSMIC_STORE.md` | New | Source | AppStore submission checklist. |
| `docs/AUR.md` | New | Source | AUR push runbook. |
| `docs/FLATPAK.md` | New | Source | Flatpak caveats + build command. |
| `justfile` | Edited | Source | New `release-*`, `flatpak-*`, `appimage-*`, `nix-*` recipes. |
| `PROJECT_PLAN.md` | Edited | Source | Tick off the Phase 18 deliverables that are now done. |

## Build Outputs (released on tag)

| Artifact | Producer | Filename pattern |
|----------|----------|-------------------|
| Debian package | `cargo deb -p author-clipboard-applet --no-build` | `author-clipboard_<version>-1_amd64.deb` |
| Linux binary tarball | `tar -C target/release -cJf ... author-clipboard author-clipboard-daemon author-clipboard-ctl author-clipboard-hypr-picker` | `author-clipboard-<version>-linux-x86_64.tar.xz` |
| Checksum file | `sha256sum` over all artifacts | `SHA256SUMS` |
| GPG signature | `gpg --armor --detach-sign SHA256SUMS` (only if secrets are set) | `SHA256SUMS.asc` |
| Arch PKGBUILD + SRCINFO | `cp packaging/arch/*` | `PKGBUILD`, `.SRCINFO` |
| AUR bundle | `tar -C packaging/arch -czf ...` | `author-clipboard-aur-files.tar.gz` |
| Release notes | `git-cliff --latest --strip header` | `RELEASE_NOTES.md` (in release body) |

## State

No new runtime state. All artifacts are reproducible from the source tree at the tagged commit. `SOURCE_DATE_EPOCH` is set in CI to the commit timestamp to support deterministic builds.

## Versioning

Version is sourced from `[workspace.package].version` in the root `Cargo.toml`. All packaging files (`PKGBUILD`, `flake.nix`, metainfo, `.SRCINFO`) must agree at release time. The release workflow fails if any of these drift from the workspace version.

---

**Last Updated**: 2026-06-08
