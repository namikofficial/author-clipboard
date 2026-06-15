# Decisions: Distribution Packages & Release Artifacts

## D-001: Release only from explicit version tags

**Decision**: `.github/workflows/release.yml` runs only for `vX.Y.Z` tags.

**Reason**: Merging to `main` must not publish artifacts before packaging has
been validated. The tag is the maintainer's explicit release action.

## D-002: Do not bump versions from release CI

**Decision**: Version changes are reviewed commits made before tagging.

**Reason**: A release workflow must not mutate `main`, trigger follow-up
workflows, or leave workspace and package metadata partially updated.

## D-003: Build packages on pull requests

**Decision**: PR CI builds and inspects the Debian package, performs a real
Arch package build, and verifies `.SRCINFO` parity.

**Reason**: Parsing package files is not evidence that users can install the
resulting artifacts.

## D-004: Keep Arch dependency resolution outside `PKGBUILD` build functions

**Decision**: Declare `gtk4-layer-shell` as a dependency and let the package
manager/CI environment resolve it.

**Reason**: Nested AUR clones, `makepkg`, and `pacman` calls inside `build()`
are non-reproducible and incompatible with normal non-root package builds.

## D-005: Disable makepkg LTO for bundled SQLite

**Decision**: The Arch package sets `options=('!lto')`.

**Reason**: Arch's makepkg LTO flags produce GCC LTO objects for bundled
SQLite that Rust's linker does not consume correctly, leaving unresolved
`sqlite3_*` symbols. Cargo's normal optimized release profile remains enabled.

## D-006: Pin gtk4-layer-shell source builds on Ubuntu CI

**Decision**: Ubuntu jobs build upstream `gtk4-layer-shell` tag `v1.3.0`
instead of installing `libgtk4-layer-shell-dev`.

**Reason**: Ubuntu 24.04 does not publish that development package. Pinning the
source tag keeps runner setup reproducible and provides the pkg-config metadata
required by the Rust bindings.

## D-007: Run Arch makepkg as a dedicated builder

**Decision**: Arch workflow jobs use `runuser -u builder` after changing
workspace ownership.

**Reason**: `makepkg` intentionally rejects root execution. The explicit user
boundary matches normal Arch packaging behavior.

**Last Updated**: 2026-06-15
