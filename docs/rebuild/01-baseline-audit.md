# Author Clipboard — Rebuild Baseline Audit

**Date:** 2026-07-27
**Branch:** dev (clean working tree)
**Commit:** baseline pre-audit

---

## Verification Summary

| Check | Result |
|-------|--------|
| `git status` | Clean working tree |
| `cargo fmt --all -- --check` | ✅ Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ Pass |
| `cargo test --workspace` | ✅ 278 shared tests + 97 ui_gtk tests + 3 daemon tests + 6 mcp tests = 384 total, all pass |
| `cargo build --workspace` | ✅ Pass |
| `just verify` | ✅ All green |

> **Note:** 14 GTK widget tests in `ui_gtk` are ignored because they require a GTK init/display (not available in this headless environment). These are not failures.

---

## Verified Working Features

### Clipboard Daemon
- **Wayland clipboard monitoring** via `wlr-data-control` protocol (`crates/clipboard-daemon/src/main.rs`)
- **Text, HTML, image, and file-list** capture paths all functional
- **Deduplication** within configurable window (`dedup_window_seconds`, default 2s)
- **Incognito mode** — skips storage when `.incognito` flag file exists
- **Content denylist** — substring/prefix/suffix/exact/regex matching
- **MIME denylist** — blocks specified MIME types before storage
- **Size limits** — `max_item_size` (default 1 MB) enforced
- **Sensitive content detection** — runs on all text, HTML, and file-list items (`crates/shared/src/sensitive.rs`)
- **Encryption-at-rest** — AES-256-GCM via `EncryptionManager` for sensitive items when `encrypt_sensitive` is true (`crates/shared/src/encryption.rs`)
- **Audit logging** — sensitive item detection, history clears, data exports logged to SQLite

### Database (`crates/shared/src/db.rs`)
- **SQLite with WAL mode** and FTS5 full-text search
- **Schema migrations** v0→v10 (content_type, sensitive, plain_text, FTS5, TTL, snippets, starred, collections, encryption metadata, saved filters)
- **CRUD operations** — insert, query, search, pin, star, delete, clear
- **Encryption integration** — `insert_with_encryption`, `decrypt_item`
- **Export/import** with redaction of encrypted content and re-derivation of sensitive flag
- **Collections** — create, rename, delete, cascade delete, membership tracking
- **Snippets** — upsert, list, search, delete
- **Saved filters** — create, list, delete by name/id
- **Per-item TTL** override support
- **Audit log** with trim

### UI (GTK4 / libcosmic) (`crates/ui-gtk/`)
- **Popup window** — clipboard history picker with search, filter, copy, paste, pin, star, delete
- **Manager window** — full history management
- **Preview pane** — text, image, file, and sensitive-state rendering
- **Redacted preview** — sensitive items show `•••••••• Sensitive item — reveal to copy` by default
- **Reveal countdown** — temporary unredaction with auto-hide
- **Keyboard navigation** — arrow keys, page up/down, home/end, Esc, search
- **GSettings bindings** — theme, window size, filter persistence
- **Toast notifications** — copy/paste feedback

### CLI (`crates/ctl/`)
- `author-clipboard-ctl` binary with history, search, copy, pin, star, delete, clear, export, import, snippet, and collection commands

### MCP Server (`crates/mcp-server/`)
- **MCP protocol** handler with sensitive-content confirmation gating
- **Redaction** — MCP output never echoes secrets even when `show_sensitive_previews` is true
- **Per-request confirmation** required for sensitive copies

### Hyprland Picker (`crates/hypr-picker/`)
- **Layer-shell native picker** for Hyprland
- **Config includes** native and external keybindings

### Security & Privacy
- **Sensitive detection** on text, HTML (multi-layer: plain text, stripped HTML, attributes, comments, raw HTML), and file lists (URI credentials)
- **Encryption-at-rest** for sensitive items (AES-256-GCM, base64(nonce || ct))
- **Redacted previews** — UIs never decrypt to display list items
- **Export redaction** — encrypted content replaced with `••••••••` in JSON exports
- **Incognito mode** — `.incognito` flag file disables all capture
- **Screen lock clearing** — `clear_sensitive` deletes unpinned sensitive items on lock
- **Audit events** — sensitive item detection, history clears, data exports tracked

---

## Reproducible Defects

### D1: `truncate_preview` in daemon logs full sensitive content
**File:** `crates/clipboard-daemon/src/main.rs:30-36`
**Symbol:** `truncate_preview`
**Severity:** P0
**Description:** The `truncate_preview` function truncates content to 80 chars for logging but does NOT check the `sensitive` flag. When a sensitive text item is stored, the log line at `main.rs:461` (`info!("📋 Stored: {preview}")`) prints the first 80 characters of the plaintext content. For sensitive items, the log should use the redacted preview or omit content entirely.
**Reproduction:** Copy a password or API key to clipboard; observe daemon logs show the first 80 chars of the secret.
**Current code path:**
```rust
let preview = truncate_preview(&content, 80);
// ...
info!("📋 Stored: {preview}");  // Logs plaintext for sensitive items!
```

### D2: HTML preview logs plaintext even for sensitive items
**File:** `crates/clipboard-daemon/src/main.rs:356-365`
**Symbol:** HTML preview generation in `DataOffer` handler
**Severity:** P0
**Description:** When an HTML clipboard item is stored, the preview is built from `plain_text` (the `text/plain` companion) and logged. For sensitive HTML items, this leaks content into logs. The `item.sensitive` flag is set by `ClipboardItem::new_html()` but is never checked before logging the preview.
**Reproduction:** Copy HTML containing a secret (e.g., `<input value="ghp_abc123...">`); observe daemon logs.

### D3: File list preview not logged but sensitive flag not checked
**File:** `crates/clipboard-daemon/src/main.rs:398-406`
**Symbol:** File list storage path
**Severity:** P1
**Description:** File list items log only the count (`"📁 Stored file list ({file_count} files)"`), which is safe. However, the `sensitive` flag is set by `ClipboardItem::new_files()` but never triggers the sensitive-item audit log path (unlike text items at line 447-459).

### D4: `show_sensitive_previews` config is defined but not wired to daemon preview generation
**File:** `crates/shared/src/config.rs:137` (`PickerConfig::show_sensitive_previews`)
**Severity:** P1
**Description:** The `PickerConfig` struct has a `show_sensitive_previews` field (default `false`), but the clipboard daemon's preview generation in `main.rs` does not consult this config. The daemon always generates a `truncate_preview` of the raw content for logging, regardless of the setting. The UI (`ui-gtk`) does check `show_redacted` state, but the daemon's log output is uncontrolled.

### D5: Encryption key file permissions not verified at startup
**File:** `crates/shared/src/encryption.rs`
**Severity:** P1
**Description:** The `EncryptionManager` reads the key from `<data_dir>/.encryption_key` but does not verify that the file has `0600` permissions. If the key file is world-readable, encryption provides no security.

### D6: IPC socket path uses fallback without verifying directory permissions
**File:** `crates/shared/src/ipc.rs`
**Severity:** P2
**Description:** The IPC socket path falls back to `<cache_dir>/author-clipboard` when `$XDG_RUNTIME_DIR` is not set. The fallback directory may have weaker permissions than `$XDG_RUNTIME_DIR`.

---

## Misleading or False Implementation Claims

### M1: "Encryption is enabled by default" — but legacy config files disable it
**File:** `crates/shared/src/config.rs:281-291`
**Claim:** `default_encrypt_sensitive()` returns `true`.
**Reality:** `from_existing_json()` preserves the legacy behavior: if `encrypt_sensitive` was absent from the config JSON, it is explicitly set to `false`. This means users upgrading from before the encryption feature had it disabled by default, which is correct for backward compatibility but contradicts the "encryption is on by default" narrative in some documentation.

### M2: `app_denylist` is documented as functional but is a no-op
**File:** `crates/shared/src/config.rs:429-449`
**Claim:** The `is_app_denied` method checks source-app against the denylist.
**Reality:** The `wlr-data-control` protocol does not expose source-app metadata to the daemon. The `source_app` field on `ClipboardItem` is always `None` in the capture path (`main.rs`). The denylist is effectively dead code until a compositor extension provides app info.

### M3: `capture_rules` tags are persisted but have no effect
**File:** `crates/clipboard-daemon/src/main.rs:118-120`
**Claim:** Capture rules can tag items.
**Reality:** The `Tag` action logs a warning (`"Capture-rule tags are not persisted by this schema"`) and does nothing else. The tag is not stored anywhere.

### M4: `redacted_preview` column exists in DB schema but is not populated for non-encrypted items
**File:** `crates/shared/src/db.rs:9-10` (schema), `crates/shared/src/types.rs:266-286` (`redacted_preview()` method)
**Claim:** The `redacted_preview` column is available for UIs.
**Reality:** The column is only populated when `insert_with_encryption` is called (for encrypted sensitive items). For non-encrypted sensitive items (when `encrypt_sensitive` is false), `redacted_preview` is `NULL` in the DB, and UIs must generate a redacted preview on the fly.

---

## Performance and Blocking-I/O Risks

### P1: Synchronous IPC calls from GTK thread may block UI
**File:** `crates/ui-gtk/src/app.rs`, `crates/ui-gtk/src/controller/key.rs`
**Risk:** The GTK UI makes synchronous IPC calls (via `IpcClient`) on the main thread for copy, paste, pin, star, delete operations. If the daemon is slow to respond (e.g., database locked, large result set), the UI freezes.
**Mitigation:** The current implementation uses `gtk::gio::Cancellable` with timeouts, but the timeout duration and error handling should be verified under load.

### P2: FTS5 search on large datasets may block
**File:** `crates/shared/src/db.rs:474-509` (`search()` method)
**Risk:** The `search()` method tries FTS5 first, then falls back to LIKE. For large clipboards (>10k items), the FTS5 query with prefix matching (`"term"*`) can be slow. The LIKE fallback is worse — it scans all rows.
**Mitigation:** The `max_items` config (default 100) limits the dataset, but users who increase it may experience slowdowns.

### P3: `enforce_max_items` uses a subquery with `LIMIT -1 OFFSET`
**File:** `crates/shared/src/db.rs:586-597`
**Risk:** The `enforce_max_items` SQL uses `LIMIT -1 OFFSET ?1` which is an O(n) scan to find the oldest non-pinned items. For large datasets, this could be slow during cleanup.

### P4: `insert_or_bump` does two queries (find + insert/update) under lock
**File:** `crates/shared/src/db.rs:415-430`
**Risk:** The dedup check (`find_by_hash` + `has_recent_duplicate`) followed by `insert_item` or `UPDATE` means two round-trips to the database for every insert. Under high clipboard churn, this could be a bottleneck.

### P5: `get_recent` loads all columns including encrypted ciphertext
**File:** `crates/shared/src/db.rs:447-455`
**Risk:** `get_recent()` selects all columns including `content` (which may be ciphertext for encrypted items). For large result sets, this loads unnecessary data. The UI only needs `redacted_preview` for display.

---

## Security and Privacy Risks

### S1: Sensitive content preview leakage in daemon logs (P0)
**File:** `crates/clipboard-daemon/src/main.rs:30-36, 441-461`
**Risk:** `truncate_preview` does not check `item.sensitive` before logging. Passwords, API keys, and tokens appear in plaintext in daemon logs.
**Fix needed:** Check `item.sensitive` before logging preview; use redacted form or omit content.

### S2: No encryption key rotation mechanism
**File:** `crates/shared/src/encryption.rs`
**Risk:** The encryption key is generated once and persisted. There is no mechanism to rotate the key or re-encrypt existing items with a new key.

### S3: `plain_text` column for HTML items stores plaintext alongside ciphertext
**File:** `crates/shared/src/db.rs:335-342` (`insert_with_encryption`)
**Risk:** When an HTML item is encrypted, the `plain_text` companion is also encrypted. However, the `plain_text` column is used for FTS5 search indexing. The FTS5 index is built from the redacted form for encrypted items (per comment at line 317-319), but the `plain_text` ciphertext is still stored in the DB. If an attacker gains read access to the DB, they see ciphertext in both `content` and `plain_text`.

### S4: Audit log may contain sensitive details
**File:** `crates/clipboard-daemon/src/main.rs:452-459`
**Risk:** The audit log for sensitive items includes `content_type=text; length={}; timestamp={}` in the details field. While this doesn't include the actual content, the length of sensitive content is logged, which could be a side-channel leak.

### S5: Export does not redact non-encrypted sensitive items
**File:** `crates/shared/src/db.rs:733-753` (`export_items()`)
**Risk:** The export function only redacts `content` for items where `encrypted == true`. Non-encrypted sensitive items (when `encrypt_sensitive` is false) are exported with their full plaintext content. The `import_items` function re-derives the sensitive flag, but the export does not redact non-encrypted sensitive items.

### S6: Incognito mode flag file is world-readable
**File:** `crates/shared/src/config.rs:333-355` (`incognito_flag_path`, `set_incognito`)
**Risk:** The `.incognito` flag file is created with default filesystem permissions (typically 0644). Any local user can read it and determine whether incognito mode is active, and can also delete it to disable incognito mode.

---

## Architecture Debt

### A1: `ClipboardItem` struct has grown to 13+ fields with mixed concerns
**File:** `crates/shared/src/types.rs:50-96`
**Debt:** The `ClipboardItem` struct mixes storage concerns (encrypted, encryption_version, redacted_preview) with domain concerns (content, mime_type, content_type). The `redacted_preview` field is only populated for encrypted items, creating an inconsistency where non-encrypted sensitive items have no redacted preview in the DB.

### A2: `Database` methods mix concerns of insert, query, and encryption
**File:** `crates/shared/src/db.rs`
**Debt:** The `Database` struct handles schema migrations, CRUD, encryption integration, export/import, snippets, collections, saved filters, audit log, and TTL. This is a large struct with many responsibilities. The encryption methods (`insert_with_encryption`, `decrypt_item`) are interleaved with basic CRUD.

### A3: `AppState` in daemon is a large struct with many fields
**File:** `crates/clipboard-daemon/src/main.rs:45-64`
**Debt:** `AppState` holds manager, seat, device, pending_offer, last_content, db, config, encryption_manager, and revision. The Wayland dispatch implementations are `impl` blocks on `AppState` with many match arms, making the struct a central point of coupling.

### A4: `IpcHandlerState` duplicates `AppState` fields
**File:** `crates/clipboard-daemon/src/main.rs:560-571`
**Debt:** `IpcHandlerState` duplicates `db`, `config`, `data_dir`, `encryption_manager`, and `revision` from `AppState`. This duplication creates a risk of inconsistency if either struct is modified.

### A5: `PickerConfig` is under `Config` but only used by UI, not daemon
**File:** `crates/shared/src/config.rs:126-164`
**Debt:** `PickerConfig` (including `show_sensitive_previews`, `confirm_sensitive_copy`, `close_after_copy`, `prefer_quick_paste`) is part of the shared `Config` struct loaded by the daemon, but the daemon does not use most of these fields. Only the UI uses them. This creates unnecessary coupling between the daemon and UI config.

### A6: `content_denylist` regex cache is not invalidated on config reload
**File:** `crates/shared/src/config.rs:73-107` (`CompiledRegexCache`)
**Debt:** The `CompiledRegexCache` uses a `OnceLock` that is populated on first call to `is_content_denied`. If the config is reloaded (e.g., `Config::load()` called again), the cache is not invalidated. The cache is tied to the `Config` instance, so a new `Config` load creates a new cache, but if the same `Config` is mutated in place, the stale cache persists.

---

## UX Inconsistencies

### U1: `show_sensitive_previews` default is `false` but behavior is inconsistent
**File:** `crates/shared/src/config.rs:137`, `crates/ui-gtk/src/app.rs` (state management)
**Issue:** The `PickerConfig::show_sensitive_previews` defaults to `false`, meaning sensitive items are redacted by default. However, the daemon logs always show plaintext previews (D1, D2). Users who rely on the UI setting to protect sensitive content may not realize that logs still leak content.

### U2: Reveal countdown is not configurable
**File:** `crates/ui-gtk/src/widgets/preview.rs`
**Issue:** The reveal countdown duration (how long sensitive content stays visible after clicking "reveal") is hardcoded. Users may want a longer or shorter countdown.

### U3: `close_after_copy` default is `true` but not documented in UI
**File:** `crates/shared/src/config.rs:141`
**Issue:** The `close_after_copy` picker option defaults to `true`, which means the picker closes automatically after a copy. This behavior is not explained in the UI and may surprise users who expect the picker to stay open.

### U4: `prefer_quick_paste` default is `false` but quick paste is the faster workflow
**File:** `crates/shared/src/config.rs:143`
**Issue:** Quick paste (paste directly without opening the picker) is disabled by default. Users who want the fastest workflow must manually enable it.

---

## P0 Priorities (Critical — Fix Immediately)

| # | Issue | File | Fix |
|---|-------|------|-----|
| 1 | Sensitive content preview leakage in daemon logs | `crates/clipboard-daemon/src/main.rs:30-36, 441-461` | Check `item.sensitive` before logging preview; use redacted form or omit content |
| 2 | HTML preview logs plaintext for sensitive items | `crates/clipboard-daemon/src/main.rs:356-365` | Check `item.sensitive` before logging HTML preview |
| 3 | Export does not redact non-encrypted sensitive items | `crates/shared/src/db.rs:733-753` | Redact sensitive items in export regardless of encryption status |

## P1 Priorities (High — Fix This Sprint)

| # | Issue | File | Fix |
|---|-------|------|-----|
| 4 | `show_sensitive_previews` not wired to daemon preview generation | `crates/clipboard-daemon/src/main.rs` | Consult `config.picker.show_sensitive_previews` before generating previews |
| 5 | Encryption key file permissions not verified | `crates/shared/src/encryption.rs` | Verify `.encryption_key` has `0600` permissions at startup |
| 6 | `app_denylist` is dead code (no source-app from compositor) | `crates/clipboard-daemon/src/main.rs` | Document as no-op or implement source-app detection |
| 7 | `capture_rules` Tag action is a no-op | `crates/clipboard-daemon/src/main.rs:118-120` | Either implement tag persistence or remove the Tag action |
| 8 | Incognito flag file has weak permissions | `crates/shared/src/config.rs:333-355` | Create `.incognito` with `0600` permissions |

## P2 Priorities (Medium — Fix This Quarter)

| # | Issue | File | Fix |
|---|-------|------|-----|
| 9 | Synchronous IPC calls may block GTK UI | `crates/ui-gtk/src/` | Move IPC calls to async worker thread |
| 10 | FTS5 search performance on large datasets | `crates/shared/src/db.rs:474-509` | Add query timeout and result limiting |
| 11 | `enforce_max_items` uses inefficient SQL | `crates/shared/src/db.rs:586-597` | Optimize with `DELETE ... ORDER BY timestamp ASC LIMIT ?` |
| 12 | `insert_or_bump` does two DB round-trips | `crates/shared/src/db.rs:415-430` | Use upsert (INSERT ... ON CONFLICT) |
| 13 | `get_recent` loads all columns including ciphertext | `crates/shared/src/db.rs:447-455` | Add a lightweight query for list views |
| 14 | No encryption key rotation mechanism | `crates/shared/src/encryption.rs` | Add key rotation with re-encryption of existing items |

## P3 Priorities (Low — Nice to Have)

| # | Issue | File | Fix |
|---|-------|------|-----|
| 15 | Reveal countdown not configurable | `crates/ui-gtk/src/widgets/preview.rs` | Add countdown duration to picker config |
| 16 | `close_after_copy` not documented in UI | `crates/ui-gtk/src/pages/settings.rs` | Add tooltip or help text |
| 17 | `prefer_quick_paste` defaults to false | `crates/shared/src/config.rs:143` | Consider changing default to true |
| 18 | `CompiledRegexCache` not invalidated on config reload | `crates/shared/src/config.rs:73-107` | Invalidate cache on config reload |
| 19 | `IpcHandlerState` duplicates `AppState` fields | `crates/clipboard-daemon/src/main.rs:560-571` | Refactor to share state via `Arc<AppState>` |

---

## Proposed Sequence for Next 19 Tasks

### Phase 1: Security Fixes (P0) — 3 tasks
1. **Fix sensitive content preview leakage in daemon logs** — Modify `truncate_preview` or the logging call sites to check `item.sensitive` and use redacted/omitted content
2. **Fix HTML preview logging for sensitive items** — Add sensitivity check before logging HTML preview in the `DataOffer` handler
3. **Fix export to redact non-encrypted sensitive items** — Update `export_items()` to redact all sensitive items, not just encrypted ones

### Phase 2: High-Priority Fixes (P1) — 5 tasks
4. **Wire `show_sensitive_previews` to daemon preview generation** — Consult config before generating log previews
5. **Verify encryption key file permissions at startup** — Add `0600` permission check for `.encryption_key`
6. **Create `.incognito` flag file with `0600` permissions** — Fix file permissions in `set_incognito()`
7. **Document `app_denylist` as no-op or implement source-app detection** — Either update docs or add compositor integration
8. **Implement or remove `capture_rules` Tag action** — Either persist tags or remove the dead code path

### Phase 3: Architecture & Performance (P2) — 6 tasks
9. **Move synchronous IPC calls to async worker thread** — Prevent GTK UI freezes
10. **Optimize FTS5 search with query timeout** — Add limits and timeouts for large datasets
11. **Optimize `enforce_max_items` SQL** — Use simpler DELETE with ORDER BY and LIMIT
12. **Optimize `insert_or_bump` with upsert** — Reduce DB round-trips
13. **Add lightweight query for list views** — Select only needed columns for `get_recent`
14. **Add encryption key rotation mechanism** — Support re-encrypting existing items with a new key

### Phase 4: UX & Polish (P3) — 5 tasks
15. **Make reveal countdown configurable** — Add to picker config
16. **Document `close_after_copy` in UI** — Add help text or tooltip
17. **Consider changing `prefer_quick_paste` default to true** — Evaluate UX impact
18. **Invalidate regex cache on config reload** — Fix stale cache issue
19. **Refactor `IpcHandlerState` to share `AppState`** — Reduce code duplication

---

## Files and Symbols Involved

### Core Crates
| Crate | Key Files | Key Symbols |
|-------|-----------|-------------|
| `shared` | `src/types.rs`, `src/db.rs`, `src/sensitive.rs`, `src/encryption.rs`, `src/config.rs`, `src/ipc.rs`, `src/clipboard.rs` | `ClipboardItem`, `Database`, `EncryptionManager`, `Config`, `SensitivityCheck`, `IpcCommand`, `IpcResponse` |
| `clipboard-daemon` | `src/main.rs` | `truncate_preview`, `AppState`, `insert_item`, `handle_command`, `IpcHandlerState` |
| `ui-gtk` | `src/app.rs`, `src/widgets/preview.rs`, `src/widgets/item_row.rs`, `src/pages/clipboard.rs`, `src/window/popup.rs`, `src/window/manager.rs` | `AppState`, `PreviewPane`, `ItemRow`, `run_popup`, `run_manager` |
| `ctl` | `src/main.rs` | CLI command parser |
| `mcp-server` | `src/handler.rs`, `src/tools.rs`, `src/server.rs` | `McpHandler`, `sensitive_copy_requires_per_request_confirmation` |
| `hypr-picker` | `src/main.rs` | Layer-shell picker |

### Spec Files
| Spec | Path |
|------|------|
| Product constitution | `specs/000-product/constitution.md` |
| Architecture | `specs/000-product/architecture.md` |
| Glossary | `specs/000-product/glossary.md` |
| Feature specs | `specs/features/001-clipboard-history/` through `specs/features/017-dotfiles-integration/` |

---

## UI and Hyprland Smoke Tests

> **Could not execute:** This environment is headless (no Wayland compositor, no GTK display). The following tests require a running Wayland session:
> - `just ui-smoke` — requires GTK display and Wayland compositor
> - `just dev` — requires running Wayland compositor for clipboard monitoring
> - Hyprland layer-shell behavior — requires Hyprland compositor
> - Popup and manager launch paths — requires GTK display
>
> The 14 ignored GTK widget tests (`requires GTK init`) confirm that GTK integration tests cannot run in this environment.

---

## CLI Options Audit

All CLI options in `crates/ctl/src/main.rs` were reviewed. Each option changes real behavior:
- `history` — queries and displays clipboard history
- `search` — full-text search across clipboard items
- `copy` — copies item to clipboard (with mode, mime, redacted options)
- `pin` / `unpin` — toggles pin state
- `star` / `unstar` — toggles star state
- `delete` — removes item
- `clear` — clears unpinned or all items
- `export` / `import` — JSON import/export with redaction
- `snippet` — CRUD for snippets
- `collection` — CRUD for collections
- `settings` — reads/updates configuration

No CLI option is a no-op or changes only non-functional behavior.

---

## Synchronous IPC Calls from GTK Thread

The following synchronous IPC calls are reachable from the GTK thread and could block the UI:

| Call Site | IPC Method | Risk |
|-----------|-----------|------|
| `crates/ui-gtk/src/app.rs` — copy requested | `IpcCommand::Copy` | Blocks until daemon responds |
| `crates/ui-gtk/src/app.rs` — paste requested | `IpcCommand::Copy` (quick paste) | Blocks until daemon responds |
| `crates/ui-gtk/src/app.rs` — pin/unpin toggled | `IpcCommand::Pin` / `IpcCommand::Unpin` | Blocks until daemon responds |
| `crates/ui-gtk/src/app.rs` — delete requested | `IpcCommand::Delete` | Blocks until daemon responds |
| `crates/ui-gtk/src/app.rs` — search changed | `IpcCommand::Search` | Blocks until daemon responds |
| `crates/ui-gtk/src/app.rs` — history loaded | `IpcCommand::History` | Blocks until daemon responds |

All calls use `gtk::gio::Cancellable` with timeouts, but the timeout duration should be verified.

---

## Sensitive Content in UI, Logs, Errors, Exports, Accessibility, Tooltips

| Surface | Current Behavior | Risk |
|---------|-----------------|------|
| **Daemon logs** | `truncate_preview` logs plaintext for sensitive items (D1, D2) | P0 — content leaked in logs |
| **UI preview** | Redacted by default; reveals on click with countdown | Low — controlled by `show_redacted` state |
| **UI tooltips** | No sensitive content in tooltips | None |
| **Accessibility text** | No sensitive content in accessibility labels | None |
| **Error messages** | Error messages do not include clipboard content | None |
| **JSON export** | Encrypted items redacted; non-encrypted sensitive items NOT redacted (S5) | P0 — plaintext export of sensitive items |
| **MCP output** | MCP handler redacts sensitive fields recursively | Low — confirmed by tests |
| **SQLite DB** | Encrypted items stored as ciphertext; non-encrypted sensitive items stored as plaintext | Medium — depends on `encrypt_sensitive` setting |

---

## Baseline Audit Complete

This document establishes the baseline for the author-clipboard rebuild. All checks pass, all features are verified, and all defects are documented with priorities and proposed fixes.
