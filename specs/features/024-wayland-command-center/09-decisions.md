# Decisions: Wayland Clipboard Command Center

## D001 — Sensitive encryption migration

**Status**: Accepted  
**Date**: 2026-07-12

New profiles enable `encrypt_sensitive` by default. Existing configuration
files that explicitly contain the key continue to use their stored value.
Existing configuration files created before the key existed (and therefore
missing it) retain the historical `false` behavior when loaded.

This distinguishes a genuinely new profile from a legacy partial config and
avoids silently changing storage behavior for existing users. Saving a loaded
legacy config writes the resolved value explicitly, making the migration
stable on subsequent launches.

## D002 — Feature numbering consolidation

**Status**: Pending

The roadmap calls this product phase 25, while implementation specs 024–027
already exist on `dev`. The command-center requirements will be reconciled
with feature 027 before final review so the repository does not retain two
unrelated feature directories with the same numeric identifier.
