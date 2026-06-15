# Feature Brief: Hardening & Polish Pass (Phases 1–16)

> Make author-clipboard a polished, secure, maintainable Linux/Wayland clipboard manager with cleaner internal architecture, better UI logic, and a better end-user experience — without a massive rewrite.

**Created**: 2026-06-08
**Status**: Approved for incremental implementation
**Branch**: `dev`
**Last Released**: v0.3.1
**Workspace Version**: 0.5.0 (unreleased dev work)

---

## Problem Statement

The project is feature-rich but the metadata, security posture, and internal architecture are not all aligned:

- `Cargo.toml` workspace is at `0.5.0`, last release is `v0.3.1`, `CHANGELOG.md` only documents up to `0.3.1`, `SECURITY.md` only supports `0.3.x` — so a new user cannot tell what is stable.
- `SECURITY.md` claims sensitive detection and encryption are fully implemented and enforced everywhere, but constructors like `new_html()` and `new_files()` do not run sensitive detection, and `encrypt_sensitive` is opt-in while the security table implies it is always active.
- The `applet` crate's `main.rs` is ~2900 lines mixing direct DB calls, IPC calls, local filtering, and widget code — no real UI/view-model boundary.
- The picker lacks debounced search, matched-term highlight, rich filters, keyboard shortcuts, and redacted sensitive previews.
- The CLI has `doctor` but it is shallow; no `--json`/`--storage`/`--security` modes and no `repair` commands.
- Image/file blobs are not cleaned up on item deletion, TTL expiry, or max-items cleanup; there is no orphan scan or repair.
- Audit events are inconsistent across copy/quick-paste/preview/clear and may log sensitive content.
- CI runs `cargo test` but not `cargo audit` / `cargo deny`; docs do not clearly state what is encrypted, indexed, or stored.

The goal is incremental, reviewable commits that close those gaps without breaking the existing public surface.

---

## Proposed Solution

A 16-phase incremental pass, executed in the order listed in the request, each phase shipped in small commits:

1. **Stabilize project/release trust** — align versions, mark `dev` as pre-release, fix `SECURITY.md` supported versions, refresh `CHANGELOG.md` and `README.md`, add badges, verify release workflow.
2. **Security correctness before more features** — run sensitive detection on text/HTML/URI/files/import paths, fix `new_html()`/`new_files()` constructors, add tests.
3. **Real encryption-at-rest flow** — store `encrypted`/`encryption_version`/`sensitive`/`redacted_preview` metadata, encrypt sensitive content before insert, decrypt at preview/copy boundary, ensure no plaintext leaks to logs/UI/search/export, add tests.
4. **Refactor UI logic** — introduce `ClipboardListItemView`, `ClipboardPreviewView`, `ClipboardAction`, `ClipboardFilterState`, `ClipboardSearchState`, `ClipboardUiError`; route picker/applet/cli actions through the same policy path; add action result states (`idle`/`loading`/`success`/`failed`/`permission required`/`sensitive confirmation required`/`daemon unavailable`).
5. **Better picker/search UX** — debounced search, matched-term highlight, filters (all/text/links/code/images/files/pinned/starred/sensitive), keyboard shortcuts, redacted sensitive previews, preview panel.
6. **Better applet/tray UX** — quick recents, open picker, pause/resume, clear actions, status display (daemon conn / capture paused / db err / key missing / wl unavailable / config err), no sensitive leaks in menu labels.
7. **Better first-run and settings UX** — safe defaults, onboarding hints, full settings surface, broken-config handling.
8. **Database/query correctness** — `get_history_page` / `search_page` / `count_*` / `update_item_flags` / `clear(scope)`; SQL-backed pinned/starred/sensitive filters; pagination tests; large-dataset smoke.
9. **Clipboard/Wayland robustness** — robust MIME fallback order, store related representations, recover from compositor restart / owner change / empty / unsupported / huge / duplicate events.
10. **Better diagnostics and `doctor`** — full check list with `--json` / `--storage` / `--security`; `repair --orphaned-files` / `--vacuum`.
11. **Storage lifecycle and cleanup** — delete/TTL/max-items clean blobs, startup orphan scan, `doctor --storage` reports orphans, `repair --orphaned-files` removes them, tests.
12. **Better CLI UX** — `status` / `history` / `search` / `copy` / `paste` / `delete` / `pin` / `star` / `clear [--sensitive]` / `pause` / `resume` / `doctor` / `config path|validate`; `--json`, non-zero exit codes, redaction, `--yes` for destructive actions.
13. **UI polish** — spacing, typography, badges, empty/loading/error states, shortcut hints, redacted previews.
14. **Consistent audit events** — full event set (capture/copy/quick-paste/preview/sensitive-copy/sensitive-preview/delete/pin/unpin/star/unstar/clear-history/clear-sensitive/pause/resume/config/key/err/start/stop/doctor), no plaintext, no secrets, no full private paths.
15. **CI/test hardening** — `fmt --check` / `check --locked` / `clippy -D warnings` / `test` / `audit` / `deny` on `dev`/`main`/PRs; test groups for config, migrations, encryption, sensitive detect, search/filter, IPC policy, CLI, storage, audit.
16. **Documentation polish** — README quickstart, install, usage, shortcuts, config, privacy model, threat model, troubleshooting, `doctor` guide, packaging, contribution; explicit "what is encrypted / what is indexed / where stored / how to delete / how to pause / how to skip apps / how to report security".

---

## Goals

- No contradictions between README, CHANGELOG, SECURITY, PROJECT_PLAN, and the workspace version.
- Sensitive detection and encryption enforced on every content type and code path.
- UI/CLI/applet share one daemon policy path; no policy logic duplicated in three places.
- Picker feels fast, keyboard-first, and clearly explains sensitive-item behavior.
- `doctor` can self-diagnose most common issues without logs.
- Orphan blobs are cleaned up; `repair` commands are safe and testable.
- CI runs the full workspace gate and matches the constitution.
- Docs match shipped behavior.

## Non-Goals

- Massive rewrite of `applet/src/main.rs`. Refactor only what is required for the new UI/view-model boundary.
- New features unrelated to hardening (e.g. cloud sync, OCR, X11) remain "Planned" in README.
- Cargo feature flags for optional GUI deps unless the build matrix already supports them.
- Rewriting the storage engine (still SQLite + FTS5).

## Stakeholders

- **End users** on COSMIC, Hyprland, Sway, and other wlroots compositors who need a stable, predictable clipboard manager.
- **Packagers** (AUR, Nix, Debian) who need version/README consistency to publish new releases.
- **Security-conscious users** who rely on the privacy model and audit-log correctness.
- **Maintainers** who need `doctor` to triage GitHub issues.

## Out of Scope

- Any change to the project constitution beyond what is required for the new UI/CLI/IPC contract.
- macOS/Windows clipboard support.
- New dependencies that are not already on the workspace dependency list, unless strictly required for a hardening goal.

---

## Suggested Commit Order

1. `chore(project): align versions, docs, and dev branch status` (Phase 1)
2. `test(security): add sensitive detection and encryption regression tests` (Phases 2–3)
3. `fix(security): detect sensitive content across text/html/uri/import paths` (Phase 2)
4. `fix(storage): encrypt sensitive content before database insert` (Phase 3)
5. `refactor(ui): introduce UI state, actions, and client boundary` (Phase 4)
6. `refactor(ipc): route picker/applet/cli actions through daemon policy` (Phase 4)
7. `fix(daemon): correct copy, quick-paste, and audit event behavior` (Phase 14)
8. `fix(db): move history/search filters and pagination into SQLite` (Phase 8)
9. `feat(picker): improve search, filters, keyboard UX, and redacted previews` (Phase 5)
10. `feat(applet): add daemon status, pause/resume, clear actions, and safe summaries` (Phase 6)
11. `feat(cli): add status, config validate, and doctor commands` (Phases 10/12)
12. `feat(storage): cleanup orphaned blobs and add storage repair` (Phase 11)
13. `ci: run full workspace checks, clippy, tests, audit, and deny` (Phase 15)
14. `docs: update privacy model, usage, config, and troubleshooting` (Phase 16)

## Quality Bar

The work is complete only when all of the following hold:

- Sensitive content is consistently detected.
- Sensitive content is encrypted at rest when configured.
- UI/CLI/applet use the same daemon policy path.
- Picker UX is fast, keyboard-first, and clear.
- Applet clearly shows app/daemon state.
- Config errors are visible and surfaced in `doctor`.
- Database pagination/filtering is correct under tests.
- Image/file storage cleanup is reliable.
- `doctor` can diagnose common problems and `repair` is safe.
- CI checks the full workspace including audit and deny.
- Docs match real behavior.

---

**Next**: see `01-requirements.md` for the per-phase acceptance criteria and `06-task-plan.md` for the atomic task breakdown.
