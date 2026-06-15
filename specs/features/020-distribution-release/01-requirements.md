# Requirements: Distribution Packages & Release Artifacts (Phase 18)

> Functional and non-functional requirements for Phase 18 packaging and release work.

---

## User Stories

### US-001: Debian/Ubuntu user installs from a published artifact
**As a** Debian/Ubuntu/Pop!_OS user
**I want to** download a `.deb` from the GitHub release
**So that** I can install author-clipboard without compiling it

**Acceptance Criteria**:
- Given a tagged release, when CI runs, then a `.deb` is attached to the release.
- The `.deb` contains the applet, daemon, ctl, AND hypr-picker binaries.
- The `.deb` postinst prompts the user to enable the systemd user service.
- `dpkg -i` followed by `apt --fix-broken install` resolves missing runtime deps.
- `apt-cache show author-clipboard` shows the maintainer, homepage, and license.

### US-002: Arch user installs from the AUR (or builds from PKGBUILD)
**As an** Arch user
**I want to** use `makepkg`/`yay` against an AUR-style package
**So that** I get all four binaries and a tracked upstream version

**Acceptance Criteria**:
- The PKGBUILD in `packaging/arch/PKGBUILD` builds cleanly on a current Arch container.
- The PKGBUILD installs applet, daemon, ctl, AND hypr-picker.
- The PKGBUILD's `source` URL tracks the current release tag.
- `.SRCINFO` is in sync with the PKGBUILD (CI verifies via `makepkg --printsrcinfo`).
- A CI job fails the build if PKGBUILD or `.SRCINFO` are out of sync.

### US-003: Flatpak user installs from Flathub or a sideload build
**As a** Flatpak user
**I want to** install author-clipboard with a manifest that grants Wayland clipboard access
**So that** I can use the applet on Fedora Silverblue, Bazzite, or COSMIC Atomic

**Acceptance Criteria**:
- A Flatpak manifest exists at `packaging/flatpak/com.namikofficial.author-clipboard.yml`.
- The manifest declares the required Wayland portals.
- A short `docs/FLATPAK.md` explains the sandbox caveats (clipboard via xdg-desktop-portal + wlr-data-control).
- `flatpak-builder` can build the manifest on a Linux host with the required SDK.

### US-004: AppImage user runs the binary without installing
**As a** user on a system where I cannot install packages
**I want to** download an `.AppImage`
**So that** I can run author-clipboard from my home directory

**Acceptance Criteria**:
- A build script `packaging/appimage/build.sh` produces a runnable AppImage.
- The AppImage bundles the applet, daemon, ctl, and the desktop + icon files.
- The AppImage runs `AppRun` and launches the applet.

### US-005: NixOS user installs via a flake
**As a** NixOS user
**I want to** `nix run github:namikofficial/author-clipboard` or add it to my system flake
**So that** I get a working installation without writing a derivation

**Acceptance Criteria**:
- `flake.nix` exists at the repo root.
- `nix flake check` succeeds (or at least the package evaluates).
- `nix build` produces the applet, daemon, ctl, and hypr-picker in `bin/`.
- A NixOS module is provided that optionally enables the systemd user service.
- `default.nix` provides a fallback for non-flakes users.

### US-006: Maintainer cuts a signed release
**As a** maintainer
**I want to** push a tag, have CI build and sign artifacts
**So that** users can verify provenance

**Acceptance Criteria**:
- CI builds all workspace binaries on a tag push.
- CI attaches `.deb`, `*-linux-x86_64.tar.xz`, `SHA256SUMS`, and (when secret is present) `.sig` to the release.
- GPG signing is conditional on `GPG_PRIVATE_KEY` and `GPG_PASSPHRASE` secrets being set.
- `docs/RELEASING.md` documents the tag-and-push flow plus how to set up signing locally.
- The release uses `git-cliff` to generate release notes.
- A merge or direct push to `main` never creates a release or mutates version files.
- The release tag must exactly match `[workspace.package].version` and the
  versions recorded by the Arch package metadata.

### US-007: Downstream/COSMIC store reviewer verifies the package
**As a** COSMIC store reviewer (or downstream packager)
**I want to** see AppStream metadata, screenshot placeholders, and an OARS rating
**So that** the submission is reviewable

**Acceptance Criteria**:
- `data/com.namikofficial.author-clipboard.metainfo.xml` validates against AppStream 1.0.
- The metainfo declares `<content_rating type="oars-1.1"/>` and `<supports><internet>offline-only</internet></supports>`.
- A `docs/COSMIC_STORE.md` checklist describes screenshot/description/payload expectations.
- The metainfo `<releases>` block is updated by CI on tag (manual step documented).

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | `[package.metadata.deb].assets` paths in `crates/applet/Cargo.toml` resolve from the workspace root, not the applet crate dir. | Must | Fix `target/release/...` → `../../target/release/...` and add hypr-picker. |
| FR-002 | The `.deb` includes all four binaries plus desktop, metainfo, icon, and systemd user service. | Must | |
| FR-003 | `release.yml` builds the full workspace, generates SHA256SUMS, and attaches the `.deb` and a binary tarball. | Must | |
| FR-004 | `release.yml` signs `SHA256SUMS` with GPG when secrets are present. | Should | |
| FR-005 | `ci.yml` adds a job that validates `packaging/arch/PKGBUILD` via `makepkg` (in an Arch container). | Must | |
| FR-006 | A Flatpak manifest exists and is documented in `docs/PACKAGING.md`. | Should | |
| FR-007 | An AppImage build script exists in `packaging/appimage/`. | Should | |
| FR-008 | A Nix flake exists at the repo root and exposes `packages.<system>.default` and `apps.<system>.default`. | Should | |
| FR-009 | `docs/RELEASING.md` documents the reproducible build and GPG signing flow. | Must | |
| FR-010 | `docs/COSMIC_STORE.md` documents the submission checklist. | Should | |
| FR-011 | `docs/AUR.md` documents how to push the PKGBUILD to the AUR. | Should | |
| FR-012 | The `justfile` adds recipes: `just deb`, `just release-archive`, `just release-checksums`, `just release-sign`, `just flatpak-build`, `just appimage-build`, `just nix-check`. | Should | |
| FR-013 | `PROJECT_PLAN.md` Phase 18 checkboxes reflect actual completion. | Must | |
| FR-014 | Releases run only for explicit `vX.Y.Z` tags; pushes to `main` never publish artifacts or create version-bump commits. | Must | |
| FR-015 | PR CI builds and inspects the Debian package, builds the Arch package, and fails when `.SRCINFO` differs from `PKGBUILD`. | Must | |
| FR-016 | Release CI verifies tag/workspace/PKGBUILD/`.SRCINFO` version parity before building or publishing artifacts. | Must | |

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | CI time for the new Arch validation job | < 5 min | Use Arch Linux container, cache pacman |
| NFR-002 | Release artifact reproducibility | Bit-for-bit identical binaries when built on the same commit on the same image | Document `SOURCE_DATE_EPOCH`, locked toolchain, `--locked` |
| NFR-003 | Signed release verification | `gpg --verify SHA256SUMS.sig SHA256SUMS` succeeds with the maintainer key | |
| NFR-004 | All new packaging files are non-interactive | `yes n`/no prompts at build time | |
| NFR-005 | No regressions in `just verify` | `cargo fmt`, `cargo clippy -D warnings`, `cargo test` all pass | |

## Edge Cases

| Case | Handling |
|------|----------|
| `cargo-deb` not installed locally | `just deb-check` prints a clear install hint. |
| `makepkg` not installed locally (CI) | The Arch validation job uses `archlinux:latest`; `ci.yml` does not require makepkg on the host. |
| No GPG key configured | Release uploads `SHA256SUMS` without `.sig`; `RELEASING.md` documents how to add signing later. |
| Nix flake evaluation fails on a non-Linux runner | Documented; the flake is only expected to evaluate on Linux by default. |
| Hyprland-only build needs GTK4 + gtk4-layer-shell | `.deb` already includes them via `$auto`; the AppImage and Flatpak manifest pin them. |
| `mcp-server` binary is a new addition | Not currently shipped; documented as future scope in RELEASING.md. |
| A release tag disagrees with package metadata | Release fails before creating a GitHub Release. |
| A release artifact build fails | No GitHub Release is created; successful artifacts remain available only as workflow artifacts for diagnosis. |

## Out of Scope

- Auto-publishing to the AUR (no AUR SSH key in this repo's secrets).
- AUR-style binary delta updates.
- Cross-compiling to `aarch64` in CI in this phase (documented for a follow-up).
- rpm packaging (Fedora) — Flatpak covers Silverblue/Bazzite/COSMIC Atomic; rpm can be a follow-up.
- Snap store submission.
- Code signing beyond GPG (no Microsoft/Apple style signing needed for Linux).

## Dependencies

- cargo-deb (`cargo install cargo-deb --locked`)
- flatpak-builder (only for `just flatpak-build`)
- `appimagetool` (downloaded by the AppImage build script)
- nix (only for `just nix-check`)
- `makepkg` / Arch Linux base (CI only)
- A maintainer GPG key (signing is optional in CI)

---

**Last Updated**: 2026-06-15
