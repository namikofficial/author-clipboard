# Feature Brief: Distribution Packages & Release Artifacts (Phase 18)

> Make installation simple across Linux distributions and ensure reproducible, signed release artifacts.

---

## Problem Statement

Today author-clipboard ships as a Cargo workspace with `.deb` metadata and an Arch PKGBUILD template, but the artifacts are not always correct, the release pipeline does not cover all distributions, and the project lacks portable packaging formats (Flatpak, AppImage, Nix) and signed releases. Users on Pop!_OS, Ubuntu, Fedora, Arch, NixOS, and the COSMIC App Store have to fall back to building from source, which is friction-heavy on a wlroots-only project.

## Proposed Solution

A coordinated distribution pipeline that:

1. **Fixes the existing `.deb` artifact** so the applet's `package.metadata.deb` asset paths resolve correctly (they currently point to `target/release/...` from the applet crate, but cargo-deb resolves relative to `CARGO_MANIFEST_DIR`).
2. **Expands the release workflow** to build every binary in the workspace, attach them with checksums to the GitHub Release, and produce a signed `.deb` artifact.
3. **Validates the Arch PKGBUILD** on every PR via `makepkg --nocheck --nodeps`.
4. **Adds Flatpak and AppImage** manifests/build scripts aimed at wlroots-friendly sandboxing.
5. **Adds a Nix flake** so NixOS users can run `nix run` and `nix profile install` without manual builds.
6. **Documents a reproducible, signed release procedure** (RELEASING.md) and a COSMIC App Store submission checklist.

## Goals

- One `just verify`-friendly pipeline that produces signed release artifacts.
- Multi-distro install paths covered: `.deb`, AUR, Flatpak, AppImage, Nix, source.
- Reproducible build notes and GPG signing step in CI (key optional, signing conditional on `GPG_PRIVATE_KEY` secret).
- No regression in CI: existing fmt/lint/test/clippy must still pass.

## Non-Goals

- Auto-publishing to the AUR — AUR push is a manual follow-up (no AUR SSH key in this repo).
- Macroservices signing keys managed by the project — the maintainer supplies their own key.
- `cargo-bundle`-style auto-format generation for Flatpak/AppImage (we ship explicit manifests).

## Stakeholders

- **End users** on Pop!_OS/Ubuntu, Fedora, Arch, NixOS who want a one-line install.
- **Packagers** who maintain downstream repos.
- **COSMIC desktop reviewers** who need an AppStream-compliant submission.
- **Maintainers** who tag releases and need a reproducible, low-friction process.

---

**Created**: 2026-06-08
**Status**: Draft (implementation in progress)
