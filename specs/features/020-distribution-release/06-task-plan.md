# Task Plan: Distribution Packages & Release Artifacts (Phase 18)

> Atomic, independently verifiable tasks. Each task lists the goal, files touched, and the verification command.

---

## T-001: Fix applet deb asset paths and add hypr-picker to the .deb
**Goal**: The `package.metadata.deb.assets` paths in `crates/applet/Cargo.toml` resolve from the workspace root and include `author-clipboard-hypr-picker`.
**Files**: `crates/applet/Cargo.toml`
**Verify**:
```bash
cargo build --release --workspace
cargo install cargo-deb --locked
cargo deb -p author-clipboard-applet --no-build
ls -la target/debian/author-clipboard_*.deb
dpkg-deb -c target/debian/author-clipboard_*.deb | grep -E "usr/bin/author-clipboard"
```

## T-002: Add Arch PKGBUILD validation to CI
**Goal**: `ci.yml` includes a job that runs `makepkg --nocheck --nodeps` on `packaging/arch/PKGBUILD` inside `archlinux:latest`, regenerates `.SRCINFO` via `makepkg --printsrcinfo`, and fails if `.SRCINFO` drifts.
**Files**: `.github/workflows/ci.yml`
**Verify**:
```bash
# Local sim (best-effort)
docker run --rm -v "$PWD":/work -w /work archlinux:latest \
  bash -lc 'pacman -Syu --noconfirm base-devel && makepkg --nocheck --nodeps'
```

## T-003: Improve release.yml
**Goal**: Build the full workspace, build the `.deb`, generate `SHA256SUMS`, conditionally sign with GPG, and attach `*.deb`, `*-linux-x86_64.tar.xz`, `SHA256SUMS` (and `.asc` if signed), plus the AUR bundle.
**Files**: `.github/workflows/release.yml`
**Verify**:
```bash
# Local sim: read the workflow and ensure all steps are present
yq '.jobs.release.steps[].name' .github/workflows/release.yml
```

## T-004: Add Flatpak manifest
**Goal**: `packaging/flatpak/com.namikofficial.author-clipboard.yml` builds author-clipboard against the Freedesktop 23.08 runtime and grants Wayland socket access.
**Files**: `packaging/flatpak/com.namikofficial.author-clipboard.yml`, `docs/FLATPAK.md`
**Verify**:
```bash
flatpak-builder --help > /dev/null 2>&1 && \
  flatpak-builder --user --sandbox --force-clean build-dir \
  packaging/flatpak/com.namikofficial.author-clipboard.yml || \
  echo "flatpak-builder not installed; manifest authored for human review"
```

## T-005: Add AppImage build script
**Goal**: `packaging/appimage/build.sh` produces a runnable AppImage from the workspace's release build.
**Files**: `packaging/appimage/build.sh`, `packaging/appimage/AppRun`, `packaging/appimage/author-clipboard.desktop`, `packaging/appimage/README.md`
**Verify**:
```bash
bash -n packaging/appimage/build.sh
```

## T-006: Add Nix flake and non-flake default.nix
**Goal**: `flake.nix` exposes `packages.<system>.default` and `apps.<system>.default`; `default.nix` provides a fallback for users without flakes enabled.
**Files**: `flake.nix`, `default.nix`, `flake.lock` (generated)
**Verify**:
```bash
command -v nix >/dev/null 2>&1 && nix flake check --no-build || echo "nix not installed; flake authored for human review"
```

## T-007: Add RELEASING.md
**Goal**: `docs/RELEASING.md` documents the maintainer flow: prep commit, set `SOURCE_DATE_EPOCH`, tag, push, sign, verify.
**Files**: `docs/RELEASING.md`

## T-008: Add COSMIC_STORE.md, AUR.md, FLATPAK.md
**Goal**: Provide a runbook per submission surface.
**Files**: `docs/COSMIC_STORE.md`, `docs/AUR.md`, `docs/FLATPAK.md`

## T-009: Update justfile with new packaging recipes
**Goal**: Add `just deb`, `just release-archive`, `just release-checksums`, `just release-sign`, `just flatpak-build`, `just appimage-build`, `just nix-check`, `just nix-build` recipes.
**Files**: `justfile`
**Verify**:
```bash
just --list
```

## T-010: Update docs/PACKAGING.md
**Goal**: Reference the new packaging forms (Flatpak, AppImage, Nix) and link to RELEASING.md, AUR.md, COSMIC_STORE.md, FLATPAK.md.
**Files**: `docs/PACKAGING.md`

## T-011: Update PROJECT_PLAN.md Phase 18 checkboxes
**Goal**: Reflect what is now done.
**Files**: `PROJECT_PLAN.md`

## T-012: Run `just fmt-check` and `just lint`
**Goal**: No regressions.
**Files**: (none — verification only)
**Verify**:
```bash
just fmt-check
just lint
```

---

**Last Updated**: 2026-06-08
