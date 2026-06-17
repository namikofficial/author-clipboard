# Task Plan: Phase 15 Denylist Completions

> Atomic, independently verifiable tasks. Each has a single goal and one
> verification command. Tasks are executed in order.

---

## TASK-001 — Add `regex` workspace dependency
**Goal**: Make `regex = "1"` available to crates via `.workspace = true`.

**Affected files**: `Cargo.toml`.

**Verify**: `grep -A1 '^regex ' Cargo.toml` shows the new entry under `[workspace.dependencies]`.

---

## TASK-002 — Add `regex` dep to shared crate
**Goal**: Wire `regex` into `crates/shared/Cargo.toml`.

**Affected files**: `crates/shared/Cargo.toml`.

**Verify**: `grep regex crates/shared/Cargo.toml` returns a hit.

---

## TASK-003 — Add `Regex` variant + tests
**Goal**: Extend `ContentPatternMode` with `Regex`; add lazy cache; add
3 regex tests.

**Affected files**: `crates/shared/src/config.rs`.

**Verify**: `cargo test -p author-clipboard-shared --lib test_content_denylist_regex` passes.

---

## TASK-004 — Add `app_denylist` field + `is_app_denied` + tests
**Goal**: Add config field, matcher method, and 4 tests; update
`test_config_roundtrip` to include the new field.

**Affected files**: `crates/shared/src/config.rs`.

**Verify**: `cargo test -p author-clipboard-shared --lib test_app_denylist` passes.

---

## TASK-005 — Wire daemon to call both matchers
**Goal**: Insert `is_app_denied` call in each capture branch (text, html,
files). Add `app_denylist` and `content_pattern_mode` to the `GetConfig`
IPC response.

**Affected files**: `crates/clipboard-daemon/src/main.rs`.

**Verify**: `cargo build -p author-clipboard-daemon` succeeds with no new warnings.

---

## TASK-006 — Run `just verify`
**Goal**: Ensure fmt + clippy + test + build are green across the workspace.

**Verify**: `just verify` exits 0.

---

## TASK-007 — Update PROJECT_PLAN.md
**Goal**: Mark Phase 15 regex + source-app denylist items done; add a one-line
note that source-app filtering is currently a no-op due to a wlr-data-control
limitation, with a pointer to `specs/features/025-phase15-denylist/09-decisions.md`.

**Affected files**: `PROJECT_PLAN.md`.

**Verify**: `grep -n '\[x\]' PROJECT_PLAN.md | wc -l` count increases by 2.

---

## TASK-008 — Commit per conventional commits
**Goal**: Split into 2 commits:
1. `feat(shared): add regex content-denylist mode and app_denylist config`
   (covers TASKS 001–005)
2. `docs(plan): mark Phase 15 regex + app denylist done; note Wayland limitation`
   (covers TASK-007; amends to TASK 001–005 in the same commit if preferred)

Use the `conventional-commit` skill for the message format.

**Verify**: `git log --oneline -3` shows the new commits and `just verify` is still green.

---

**Last Updated**: Phase 15 completion (June 2026)
