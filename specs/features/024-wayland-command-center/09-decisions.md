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
# Foundation Decisions (T002–T005)

## D-024-004: Keep derived index compatibility during ID migration

`selected_id` is authoritative. `selected_index` remains temporarily as a
derived compatibility field for preview and keyboard code, and is updated only
through selection helpers. This limits one foundation change from destabilizing
unrelated GTK presentation work. It is a documented deviation from immediately
removing index state.

## D-024-005: Use a pure keyed reconciliation plan

The current GTK dependency supports list models, but introducing a custom
GObject item subclass while multiple UI features are landing adds unnecessary
binding complexity. The foundation uses a pure ID-keyed reconciliation model,
allowing retained row reuse and deterministic inserts/removes/moves. A future
increment may adapt the same snapshot contract to `gio::ListStore`.

## D-024-006: Bridge capture refresh with a revision file monitor

The existing request/response Unix-socket client does not hold long-lived event
connections, despite the daemon already broadcasting mutation events internally.
For this increment the daemon publishes a monotonic `.history_revision` file and
GTK uses `gio::FileMonitor` to trigger an IPC snapshot refresh. This is explicit,
edge-triggered signaling and removes the arbitrary 200 ms delayed reload. The
History/Search responses also expose the revision so a future persistent IPC
subscription can replace the file bridge without changing snapshot semantics.

## D-024-007: MCP redaction is independent of UI configuration

Every MCP search, resource, safe-get, and generated prompt applies a final
recursive redaction pass. The daemon's UI preview setting is not trusted as an
MCP authorization signal. Full sensitive get/copy and destructive tools require
boolean confirmation on that individual request; confirmation is not cached.

## D-024-013: Shared workflow primitives

Transforms are pure and return non-content-bearing errors. The existing
`${name}` template engine stays canonical; strict `{name}` command-center
syntax is an adapter. Export uses a versioned envelope, redacts sensitive and
encrypted history by default, and gates full output on explicit confirmation.

## D-024-012: Ignore-next-copy sentinel

Ignore-next-copy is armed through IPC and represented by an application-owned
sentinel in the data directory. The capture path atomically removes it before
storage, so one daemon process consumes one eligible capture exactly once.

Capture rules are evaluated in configuration order and the first matching
enabled rule wins. Supported storage actions are `ignore` and
`force_sensitive`; `tag` is represented and validated but reported unsupported
until clipboard items have a persisted tag field. Invalid matchers fail config
validation instead of silently broadening capture.
