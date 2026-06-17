# Decisions: Phase 15 Denylist Completions

> Deviations from the original PROJECT_PLAN.md description and the reasoning
> behind non-obvious technical choices.

---

## DECISION-001 — `app_denylist` ships forward-compatible only

**Context**: PROJECT_PLAN.md Phase 15 promises "Clipboard ignore rules by
source application (where Wayland allows)". The wlr-data-control-unstable-v1
protocol does **not** carry source-app metadata on data offers — the
compositor delivers MIME types and raw bytes only. There is no way for the
daemon to know which application copied the content under wlr-data-control.
The ext-data-control-v1 protocol (newer) does carry a `source` app_id field,
but no compositor the project targets exposes it today.

**Decision**: We still add the `app_denylist` config and the `is_app_denied`
matcher. The daemon calls the matcher in the capture path with
`source_app: None`. The wiring is exercised by unit tests and will activate
the moment any compositor starts supplying source-app info (ext-data-control-v1
or future protocols).

**Why ship it now**:
1. The config surface is small (one `Vec<String>` field) and non-breaking.
2. The matcher logic is the part most likely to have edge cases (case
   sensitivity, path vs basename, Unicode); shipping it now lets us test it.
3. When the protocol gap closes, no further code change is needed.

**Alternatives considered**:
- **Drop the feature until source-app is available.** Rejected: the matcher
  has independent value as a query-time filter via the existing `app:`
  filter chip, and shipping the config + tests lets us catch design bugs
  early.
- **Implement via compositor-specific protocols (e.g. Hyprland IPC).**
  Out of scope for this spec; tracked as a follow-up under Phase 16 (X11/host
  integration work).

**Status**: Active. Follow-up: track ext-data-control-v1 adoption by COSMIC,
Hyprland, and Sway; file an issue when any of them exposes source-app info.

---

## DECISION-002 — Use `regex` (full crate), not `regex-lite`

**Context**: Two popular Rust regex crates exist. `regex-lite` is a smaller
subset that's easier on build time and binary size; `regex` is the canonical
implementation with full Unicode support, better error messages, and a
stable API.

**Decision**: Use `regex = "1"` (full crate).

**Why**:
- Already accept multi-second compile times (`libcosmic`, `wayland-*`).
- Full Unicode handling matters for clipboard content (multi-byte chars).
- Better error messages when a user's config regex is invalid.

**Trade-off**: ~300KB more in the final binary, ~1-3s more build time. Acceptable.

---

## DECISION-003 — Lazy regex cache via `OnceLock`

**Context**: `is_content_denied` is called on every clipboard event in the
hot path. Compiling a regex per call would be wasteful.

**Decision**: Compile once on first use via `std::sync::OnceLock`. No mutex
needed because the daemon's event loop is single-threaded for clipboard
dispatch.

**Why not `LazyLock`?** `LazyLock` requires `const` initializers in some
contexts; `OnceLock::get_or_init` with a closure is more flexible for our
borrow situation (we need to capture `&self`).

---

## DECISION-004 — Invalid regex warns once and matches nothing

**Context**: A user typo in a regex (`[unclosed`) must not silently match
everything (false negatives) or block startup (false positive).

**Decision**: Compile errors are stored as `None` in the cache; `is_match`
returns `false`. The first time an invalid pattern is observed, a single
`tracing::warn!` is emitted naming the bad pattern. Subsequent calls
quietly return `false`.

**Why not fail-closed (deny on invalid)?**: A typo'd rule shouldn't
unexpectedly block all matching content — that would silently disable the
user's safety net. Better to make the typo visible via logs and let the
user fix it.

---

**Last Updated**: Phase 15 completion (June 2026)
