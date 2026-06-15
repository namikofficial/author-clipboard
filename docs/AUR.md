# AUR Submission Guide

This document describes how to push the Arch PKGBUILD to the AUR. The PKGBUILD
lives in `packaging/arch/PKGBUILD` and is built and validated in CI on every
PR (see `.github/workflows/ci.yml` → `arch-pkg` job).

## Prerequisites

- An AUR account (https://aur.archlinux.org/register).
- An SSH public key registered with your AUR account.
- `git`, `makepkg`, and `pacman` (Arch Linux or container).

## One-time Setup

```bash
git clone ssh://aur@aur.archlinux.org/author-clipboard.git aur-author-clipboard
cd aur-author-clipboard
git config user.name  "Your Name"
git config user.email "you@example.com"
```

## Cutting a New AUR Release

```bash
# 1. Update the version inside packaging/arch/PKGBUILD
$EDITOR ../packaging/arch/PKGBUILD
# 2. Regenerate .SRCINFO from PKGBUILD
makepkg --printsrcinfo > ../packaging/arch/.SRCINFO
# 3. Smoke-test the build (dependencies include gtk4-layer-shell)
makepkg --noconfirm --syncdeps --cleanbuild
# 4. Commit & push
git add PKGBUILD .SRCINFO
git commit -m "upgpkg: author-clipboard 0.6.0"
git push
```

`upgpkg:` is the AUR convention for "this is a version bump of an existing
package." The first push uses `git push --set-upstream origin master` (the
AUR's default branch is `master`, not `main`).

## Validating Locally

```bash
# Re-derive .SRCINFO and ensure the file is byte-identical
cd packaging/arch
makepkg --printsrcinfo > .SRCINFO.new
diff -u .SRCINFO .SRCINFO.new && rm .SRCINFO.new

# Build without install (CI preinstalls dependencies and does this)
makepkg --noconfirm --cleanbuild --skipinteg
```

## Sources of Truth

- The AUR's PKGBUILD is a **mirror** of `packaging/arch/PKGBUILD`. They must
  stay in lockstep. CI fails the PR if `.SRCINFO` is out of sync.
- The AUR does not store release binaries — it builds from the source tarball
  on each install. The release workflow tags `v<version>` and the
  PKGBUILD's `source` URL points at `archive/v<version>.tar.gz`.

## After First Submission

- Add a comment on the AUR page pointing to the upstream project
  (https://github.com/namikofficial/author-clipboard).
- Subscribe to upstream issues so you get notified of new releases.

## See Also

- [`docs/RELEASING.md`](RELEASING.md) — overall release flow.
- [`docs/PACKAGING.md`](PACKAGING.md) — all install paths.
