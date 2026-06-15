# Release Runbook

This is the maintainer runbook for cutting a release of author-clipboard. The
release process is built around [Conventional Commits](https://www.conventionalcommits.org/)
and `git-cliff`, and is fully automated by `.github/workflows/release.yml`.

## Quick Path

1. Ensure `main` is green and you have a clean working tree.
2. Decide the new version. Update it in `Cargo.toml`,
   `packaging/arch/PKGBUILD`, the generated `packaging/arch/.SRCINFO`, and
   other pinned packaging manifests such as Flatpak/Nix.
3. Regenerate and verify Arch metadata:

   ```bash
   cd packaging/arch
   makepkg --printsrcinfo > .SRCINFO
   cd ../..
   just aur-check
   ```
4. (Optional) Set the maintainer GPG key in repo secrets (see
   [Signing](#signing-releases-with-gpg) below).
5. Merge the version-preparation commit only after PR CI is green, then tag and
   push:

   ```bash
   git switch main
   git pull --rebase
   just release 0.6.0       # updates CHANGELOG, commits, tags v0.6.0
   git push && git push --tags
   ```

6. The `Release` workflow runs only for the explicit `vX.Y.Z` tag:
   - Reject a tag that differs from Cargo, PKGBUILD, or `.SRCINFO`
   - Format, lint, test
   - Build the full workspace
   - Build and inspect the `.deb`
   - Build the Arch package
   - Generate `SHA256SUMS`
   - Sign `SHA256SUMS` (if GPG secrets are set)
   - Bundle PKGBUILD / `.SRCINFO` for AUR
   - Publish the GitHub Release with all artifacts

7. After CI completes, **verify** the release:

   ```bash
   gh release download v0.6.0
   sha256sum -c SHA256SUMS
   # If SHA256SUMS.asc is present:
   gpg --verify SHA256SUMS.asc SHA256SUMS
   ```

8. Submit to the COSMIC store (see [`docs/COSMIC_STORE.md`](COSMIC_STORE.md)).
9. Push the PKGBUILD to the AUR (see [`docs/AUR.md`](AUR.md)).

Merging or pushing to `main` never creates a release. The workflow also never
bumps versions or pushes commits back to `main`.

## What Goes Into a Release

| Artifact | Producer | Filename |
|----------|----------|----------|
| Debian package | `cargo deb` against `crates/applet` | `author-clipboard_<ver>-1_amd64.deb` |
| Linux tarball | `tar -C target/release -cJf` | `author-clipboard-<ver>-linux-x86_64.tar.xz` |
| Checksums | `sha256sum` | `SHA256SUMS` |
| GPG signature | `gpg --armor --detach-sign SHA256SUMS` | `SHA256SUMS.asc` (optional) |
| AppStream metainfo | copied from `data/` | `com.namikofficial.author-clipboard.metainfo.xml` |
| AUR bundle | `tar -C packaging/arch -czf` | `author-clipboard-aur-files.tar.gz` |
| Arch package | `makepkg` in `archlinux:latest` | `author-clipboard-<ver>-1-x86_64.pkg.tar.zst` |
| Release notes | `git-cliff --latest --strip header` | release body |

## Reproducible Builds

The release workflow pins:

- **Toolchain** — `dtolnay/rust-toolchain@stable` (same image across runs).
- **Deps** — `cargo build --locked` rejects a `Cargo.lock` drift.
- **Source date** — `SOURCE_DATE_EPOCH` is set to the commit timestamp
  (`git log -1 --format=%ct`), so timestamps embedded in build artifacts
  are deterministic.

Bit-for-bit identical binaries across hosts are **best-effort** because:

- `libcosmic` is a git dependency; its commit hash is pinned in
  `Cargo.lock`, but build-time metadata (e.g. paths) can still differ.
- Cargo's incremental compilation artifacts may differ across runs even
  with the same inputs.

What we **do** guarantee: builds from the same tag on the same GitHub
Actions runner image produce the same `SHA256SUMS` for the *content* of
the binaries modulo debug info layout.

## Signing Releases with GPG

Signing is **optional**. To enable it:

1. Generate (or reuse) a maintainer GPG key:

   ```bash
   gpg --full-generate-key
   # RSA, 4096 bits, name = "Namik <author-clipboard@namik.dev>"
   ```

2. Export the **private** key in ASCII-armored form:

   ```bash
   gpg --armor --export-secret-keys author-clipboard@namik.dev > gpg-private.key
   ```

3. Add two repository secrets (Settings → Secrets and variables → Actions):

   - `GPG_PRIVATE_KEY` — the contents of `gpg-private.key`.
   - `GPG_PASSPHRASE` — the key's passphrase.

4. Re-run the failed `Release` workflow (or push the tag again if you have
   not yet cut a release). The job will:

   ```bash
   echo "$GPG_PRIVATE_KEY" | gpg --batch --import
   gpg --batch --yes --pinentry-mode loopback \
       --passphrase "$GPG_PASSPHRASE" \
       --armor --detach-sign --output SHA256SUMS.asc SHA256SUMS
   ```

5. Verify on the client side:

   ```bash
   gpg --verify SHA256SUMS.asc SHA256SUMS
   ```

6. Publish your **public** key somewhere durable (e.g. a `KEYS` file in
   the repo, or upload to `keys.openpgp.org`).

### Rotating / Revoking

If the key is compromised:

- Revoke the key (`gpg --gen-revoke`).
- Publish a signed announcement in the next release notes.
- Re-issue the key and update secrets.

## Verifying a Release as a User

```bash
# 1. Download
gh release download v0.6.0

# 2. (Optional) Verify the maintainer's signature on the checksums
gpg --verify SHA256SUMS.asc SHA256SUMS

# 3. Verify the checksums
sha256sum -c SHA256SUMS

# 4. Install the .deb
sudo dpkg -i author-clipboard_0.6.0-1_amd64.deb
sudo apt --fix-broken install
```

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `cargo deb` fails: "no such file or directory" | The asset paths in `crates/applet/Cargo.toml` must be `../../target/release/...`. |
| `gpg: signing failed: Inappropriate ioctl` | The CI step uses `--pinentry-mode loopback`; locally, configure `pinentry-mode loopback` in `~/.gnupg/gpg-agent.conf`. |
| `mismatched hashes` on tarball | `SOURCE_DATE_EPOCH` not exported; re-run the release job. |
| `.SRCINFO` out of sync with `PKGBUILD` | `cd packaging/arch && makepkg --printsrcinfo > .SRCINFO` |
| Release exits during version validation | Make the tag, Cargo version, `pkgver`, and `.SRCINFO` version identical before tagging. |
| Hyprland picker missing from .deb | Check `crates/applet/Cargo.toml` `[package.metadata.deb].assets` for the `author-clipboard-hypr-picker` line. |

## See Also

- [`docs/PACKAGING.md`](PACKAGING.md) — all install paths.
- [`docs/AUR.md`](AUR.md) — pushing the PKGBUILD.
- [`docs/COSMIC_STORE.md`](COSMIC_STORE.md) — store submission.
- [`docs/FLATPAK.md`](FLATPAK.md) — Flatpak caveats.
- [`docs/CHANGELOG.md`](../CHANGELOG.md) — generated by `git-cliff`.
