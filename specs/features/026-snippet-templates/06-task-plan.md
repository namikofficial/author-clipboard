# Task Plan: Snippet Token Replacement & Preview

> Atomic, independently verifiable tasks. Each has a single goal and one
> verification command.

---

## TASK-001 — Add `template` module skeleton
**Goal**: New file `crates/shared/src/template.rs` with `RenderContext`,
`render`, and `render_now` signatures, plus 5 sanity tests.

**Affected files**: `crates/shared/src/template.rs`, `crates/shared/src/lib.rs`.

**Verify**: `cargo test -p author-clipboard-shared --lib template::tests::test_render_passthrough` passes.

---

## TASK-002 — Implement built-in variable table
**Goal**: Resolve `date`, `time`, `datetime`, `iso_*`, `year/month/day`,
`hour/minute/second`, `unix`, `uuid`, `random:N`, `cursor`, `clipboard`,
`user`, `hostname` against a `RenderContext`. Add 10 tests.

**Verify**: `cargo test -p author-clipboard-shared --lib template::tests::test_render_builtin` passes (all 18 tests total).

---

## TASK-003 — Escape and unknown-variable handling
**Goal**: `$$` → `$`, `\$` → `$`, unclosed `${` literal, unknown
`${name}` literal, empty `${}` literal. Add tests.

**Verify**: `cargo test -p author-clipboard-shared --lib template::tests::test_render_escape` passes.

---

## TASK-004 — Add `RenderSnippet` IPC command + daemon handler
**Goal**: New `IpcCommand::RenderSnippet { id: i64 }` variant.
Implement the daemon handler with the `RenderContext` it builds from
app state. Add a small `db.get_snippet(id)` helper.

**Affected files**: `crates/shared/src/ipc.rs`, `crates/shared/src/db.rs`, `crates/clipboard-daemon/src/main.rs`.

**Verify**: `cargo build -p author-clipboard-daemon` succeeds.

---

## TASK-005 — Picker preview (shared)
**Goal**: No change to `PickerEntry`. Add a helper
`fn rendered_preview(entry: &PickerEntry) -> String` next to
`snippet_preview` that returns the template-rendered text for
snippet entries and the existing subtitle (or empty string) for
others. UI sites will call this.

**Affected files**: `crates/shared/src/picker.rs`.

**Verify**: `cargo test -p author-clipboard-shared --lib picker::tests::test_snippet_preview` passes.

---

## TASK-006 — CLI `expand-snippet` subcommand
**Goal**: `author-clipboard-ctl expand-snippet <name|id>` with
`--stdout` and `--cursor-offset` flags.

**Affected files**: `crates/ctl/src/main.rs`, `crates/ctl/Cargo.toml` (if any new deps — none expected).

**Verify**: `cargo build -p author-clipboard-ctl` succeeds; `author-clipboard-ctl expand-snippet --help` shows the new flags.

---

## TASK-007 — UI-gtk snippet preview row
**Goal**: Snippets page shows a read-only preview label below the
content entry; updates on `content_entry` change.

**Affected files**: `crates/ui-gtk/src/pages/snippets.rs`.

**Verify**: `cargo build -p author-clipboard-ui-gtk` succeeds; `cargo clippy -p author-clipboard-ui-gtk -- -D warnings` clean.

---

## TASK-008 — Update PROJECT_PLAN.md
**Goal**: Mark the Phase 15 snippet-templates checkbox done with a one-line
link to this spec.

**Affected files**: `PROJECT_PLAN.md`.

**Verify**: `grep -n 'snippet-templates' PROJECT_PLAN.md` shows the spec link.

---

## TASK-009 — Run `just verify`
**Goal**: Ensure fmt + clippy + test + build are green.

**Verify**: `just verify` exits 0.

---

## TASK-010 — Conventional commits
**Goal**: One commit:
`feat(snippets): add token replacement engine, picker preview, and expand-snippet cli`

**Verify**: `git log --oneline -2` shows the commit; `just verify` still green.

---

**Last Updated**: Phase 15 completion (June 2026)
