# Master Execution Plan: Complete the Unified GTK4 UI (feature 023)

> Build-ready handoff for the next phase. Each PR card below is
> self-contained: a subagent given only that card plus the boilerplate
> rules in §4 can ship the PR without further context.
>
> **PR 0 and PR 1 are already complete** (138 tests pass, `just verify`
> green, branch `feat/023-popup-bugs`, `pre-023-ui-rewrite` tag
> present). This plan covers **PRs 2 → 7** plus a 5.5 milestone.
>
> Cross-references: completion plan lives at
> `specs/features/023-unified-gtk4-ui/10-completion-plan.md` (cited as
> "CP §PR-N" below). Audit baseline: `docs/023-current-state.md`.

---

## 1. Executive summary

Feature 023 replaces the libcosmic applet with a unified GTK4 +
libadwaita UI. The skeleton is in place: `crates/ui-gtk/` has 35 files
(2,565 LOC), 138 tests pass, layer-shell popup + manager window both
launch, IPC `Copy` already carries the `mime: Option<String>` field
(PR 1), and `ApState` / `Action` / `Effect` / `reduce` **do not exist
yet** — `app.rs` is 75 LOC of mostly-unused enums (`KeyAction`,
`FilterState`, `SortOrder`). The plan below finishes the 20 spec tasks
across 7 PRs by layering state and effects under the existing widget
tree, then closing the loop with the manager rewrite and the docs.

**Why we split it this way.** The spec's 20 tasks are interlocked:
T004 (reducer) feeds T005/T006 (key controller + GSettings); T005/T006
feed T015 (manager); T015 feeds T020 (docs). A single big-bang PR would
be un-reviewable. The split below keeps each PR **independently
compilable** (CP rule 1) and **independently verifiable** (`just
verify` green at every step). PR 3 splits T004 into a foundation slice
(PR 3A: ~12 tests) and a full-coverage slice (PR 3B: ~30 tests)
because ~900 LOC of reducer code is too much for one agent pass (CP
locked decision).

### What this plan does NOT do (out of scope)

- **`gio::ListStore` + `gtk::SingleSelection` virtualization**
  (NFR-002 from `05-technical-design.md`). The list keeps rebuilding
  on refresh; a follow-up can swap to a recycled model.
- **`AdwComboRow` / `AdwSwitchRow` for the settings page**. The
  current `Switch` + `SpinButton` widgets stay; cosmetic upgrade
  tracked as PR 7.5 in CP §Out of scope.
- **WebView sandbox E2E tests** (PR 5.5). The plan unit-tests
  `WebContext` setup only.
- **CI install of `webkitgtk-6.0-dev`** (D13). The default build
  stays WebKit-free; the maintainer runs `--features webview` locally.
- **libcosmic reintroduction**. Hard ban (D1, D5, `08-review-checklist.md`).
- **mcp-server changes** (`crates/mcp-server/`). Not in AGENTS.md's
  crate list; the existing `IpcCommand::Copy { mime: None }` call site
  at `crates/mcp-server/src/handler.rs:276` already passes the audit.
- **Performance work** (NFR-001..005 in `07-test-plan.md`). Benchmarks
  are local-only, never CI.

---

## 2. Parallelization map

```
            ┌─ PR 2 (T018 shared picker) ───────────────────────────────────────┐
            │                                                                  │
PR 0 → PR 1 ┼─ PR 3A (T004 foundation) ──► PR 3B (T004 completion) ──► PR 4 ──┼─► PR 6 (T013/T015) ──► PR 7
            │                                              │                   │                  (T017/19/20)
            │                                              │                   │
            │                                              └─► PR 5 (T010) ──► PR 5.5 (WebKit) ───┘
            │
            └─ All in series on branch `feat/023-popup-bugs` (extend, no fork)
```

### Honest parallelism assessment

**True parallelizable pairs** (different files, no shared reducer / IPC
contract):

- **PR 2 ⇄ PR 3A.** PR 2 touches `crates/shared/src/picker.rs` +
  `crates/ctl/src/main.rs` + `crates/ui-gtk/src/pages/clipboard.rs`
  (only the `load_entries_for` helper). PR 3A creates a brand-new
  `app::reduce` and an `AppState` struct. They **do not overlap on
  the same file's API surface**. However, PR 3A imports
  `PickerFilter` and reads it, so a merge conflict on `pages/clipboard.rs`
  is possible if both are in flight at the same time on the same
  branch. **Recommendation: keep PR 2 and PR 3A on the same branch
  but treat them as one logical "shared-filter + reducer-foundation"
  step**. A subagent can do them in one session if needed.

- **PR 5 ⇄ PR 4.** PR 5 is widget-only (`widgets/preview.rs`); PR 4
  is controller + settings. They share no file. The blocker is that
  PR 6 depends on **both** finishing, so PR 5 cannot land before
  PR 4 only if reviewers insist on a serial merge order. In practice
  the maintainer can merge PR 5 in parallel with PR 4.

**Sequential** (must be ordered):

- **PR 1 → PR 3A.** The reducer's invariants assume
  `PageState::from_props` (D12, PR 1) and `IpcCommand::Copy { mime }`
  (D11, PR 1) are in place.
- **PR 3A → PR 3B.** The foundation `AppState` field set is the input
  to PR 3B's expanded surface.
- **PR 3B → PR 4.** PR 4 wires `Action` and `Effect` (both
  introduced in PR 3A/3B) into the global key controller and
  GSettings binding.
- **PR 5 → PR 5.5.** PR 5.5 extends PR 5's HTML branch.
- **PR 4 + PR 5 → PR 6.** The manager rewrite mounts `PreviewPane`
  (PR 5) and uses the global key controller (PR 4).
- **PR 6 → PR 7.** PR 7 adds `--filter` to hypr-picker (manager
  layout must be finalised so the help text matches the real
  shortcuts), updates `tests/smoke.sh` against the rewritten manager,
  and writes the docs that describe the final layout.

### Rationale table

| Ordering | Why it must be this way |
|---|---|
| PR 0, PR 1 → everything | Audit and P0 bug fixes established the API surface (`mime`, `PageState::from_props`) the rest of the plan assumes. |
| PR 2 in any order | Pure `shared::picker` change; `IpcCommand` is untouched; only one consumer (`ui-gtk/pages/clipboard.rs`) is in scope and that consumer does the trivial swap. |
| PR 3A → PR 3B → PR 4 | Reducer foundation before full coverage before controller wiring. The new `Action` variants in PR 3B are referenced by `controller/key.rs::map_key_extended` in PR 4. |
| PR 5 in parallel with PR 4 | PreviewPane widget does not depend on the controller; the manager rewrite (PR 6) needs both. |
| PR 5.5 last widget PR | Feature-gated; depends on PR 5's `ContentType::Html` arm existing. |
| PR 6 second-to-last | Biggest UI change; depends on PR 4 (key controller) + PR 5 (preview). |
| PR 7 last | Docs, snapshots, `--filter` flag, `ui-check`/`ui-smoke` recipes; describes the finalised manager. |

**Branch strategy**: all PRs land on `feat/023-popup-bugs` as linear
commits (no PR branches to merge back). Each PR is one or two
conventional commits; squash at review time if reviewers prefer.

---

## 3. Per-PR execution cards

> Each card is the subagent brief. The agent must read the
> `crates/ui-gtk/src/` and `crates/shared/src/picker.rs` files cited
> before patching. The agent must not invent GTK / libadwaita APIs
> (CP rule 3, also D15): if a desired widget is missing in the
> current crate version, fall back to `gtk::Box` + `gtk::ListBox` and
> record the choice in `09-decisions.md`.

### Pre-flight for every PR

Every subagent must run these **before** touching any file:

```bash
cd /home/namik/Documents/code/author-clipboard
git status                          # clean working tree expected
git rev-parse --abbrev-ref HEAD     # must be feat/023-popup-bugs
git tag --list 'pre-023*'           # must include pre-023-ui-rewrite
cargo --version                     # record version in the commit body
just verify                         # baseline must be green
```

If `just verify` is not green, **stop and report** — the agent must
not paper over a broken baseline.

---

### PR 2 — T018: thread `PickerFilter` through shared picker

- **Title**: `feat(shared): thread PickerFilter through filter_and_query + build_external_rows`
- **Estimated LOC delta**: +120 / −60 (in `crates/shared/`, `crates/ctl/`, `crates/ui-gtk/src/pages/clipboard.rs`).
- **Spec references**: CP §PR-2; `10-completion-plan.md` lines 196–232.

**Pre-PR verification**

```bash
just verify
rg -n 'filter_entries\(' crates/    # confirm call sites
```

**Files to touch**

- `crates/shared/src/picker.rs` (line 798 `build_external_rows`, line 834 `filter_entries`, line 850 `apply_filter`).
  - Add `pub fn filter_and_query(entries: &[PickerEntry], query: &str, filter: PickerFilter) -> Vec<PickerEntry>`. When `query.is_empty() && filter == PickerFilter::All`, return `entries.to_vec()` (identity). Otherwise apply `apply_filter` then `filter_entries`-style substring match.
  - Change `build_external_rows(entries, include_key_prefix)` → `build_external_rows(entries, filter, include_key_prefix)`. Internally call `apply_filter` so callers stop double-filtering.
  - Keep the old 2-arg `filter_entries(entries, query)` as a thin wrapper for the existing `test_filter_entries` test (which the subagent can keep or delete — see acceptance).
  - Add tests: every `PickerFilter` × query combination (7 filters × 3 queries = 21 cases minimum), plus empty-query + All identity, plus query with non-matching filter (still empty).
- `crates/ctl/src/main.rs` (line 562 `run_external_picker`).
  - Replace `picker::build_external_rows(&entries, true)` → `picker::build_external_rows(&entries, filter_enum, true)`. Drop the standalone `picker::apply_filter(...)` call above it (now done internally).
  - The `filter.parse().unwrap_or_default()` at line 591 stays.
- `crates/ui-gtk/src/pages/clipboard.rs` (line 237, `load_entries_for`).
  - Replace `picker::filter_entries(&entries, query)` with `picker::filter_and_query(&entries, query, filter)`. Drop the now-redundant `picker::apply_filter(...)` call at line 239 (move it inside `filter_and_query`).

**Acceptance criteria**

- `cargo test -p author-clipboard-shared` — ≥16 new test cases pass (record the exact number in the commit body).
- `cargo test -p author-clipboard-ctl` — green.
- `cargo test -p author-clipboard-ui-gtk -- clipboard` — green (no GTK init).
- `just verify` — green.
- `rg -n 'apply_filter|filter_entries' crates/` shows only the wrapper + the internal call inside `filter_and_query` / `build_external_rows`; no other call sites.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` row T018 → ✅.
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — **no new decision required** (this PR is exactly what the spec describes; no deviation).

**Subagent prompt (copy-paste)**

```
You are completing PR 2 of feature 023 on branch
feat/023-popup-bugs. `just verify` is green; 138 tests pass.
Read specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-2
and the four files listed in the "Files to touch" section above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Make the edits described in "Files to touch".
  3. Run the four cargo invocations + `just verify` from
     "Acceptance criteria".
  4. Commit:
       feat(shared): thread PickerFilter through filter_and_query

       Move apply_filter inside build_external_rows and add
       filter_and_query(entries, query, filter) so all three UIs
       share one filter+query path. ~16 new tests.
  5. Tick T018 in 06-task-plan.md (same commit or follow-up).

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for this PR
is: do not touch IpcCommand, do not edit
pages/clipboard.rs beyond the two `load_entries_for` lines, do
not add a new dep, do not edit AGENTS.md / docs/UI.md.

Rollback: revert one commit; no schema or IPC change.
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit + one docs tick.

**Rollback risk**: Low (additive; one public function renamed;
one extra parameter added; all 4 callers updated atomically).

**Decision updates**: none. If the test count differs by more than
±2 from the CP's "~16 new test cases", add a one-liner to
`09-decisions.md` under a new "D-note" header (not a full D-number).

---

### PR 3A — T004 (foundation): minimal `AppState` + `Action` + `reduce`

- **Title**: `feat(ui-gtk): introduce AppState + Action + reduce() foundation slice`
- **Estimated LOC delta**: +400 / −20 (in `crates/ui-gtk/src/app.rs`, `crates/ui-gtk/src/lib.rs`).
- **Spec references**: CP §PR-3A; `10-completion-plan.md` lines 234–283.

**Pre-PR verification**

```bash
just verify
cargo test -p author-clipboard-ui-gtk --lib    # currently 1 test (focus::resolve_escape) passes
```

**Files to touch**

- `crates/ui-gtk/src/app.rs` (rewrite from 75 LOC).
  - **Delete** the placeholder `KeyAction` (13 variants) and the unused `FilterState`, `SortOrder`. They were placeholders from PR 1's bug-fix slice and the new model supersedes them.
  - Add `pub enum PageId { Clipboard, Emoji, Symbols, Kaomoji, Snippets, Settings }` with `serde`-less derive; `Display + FromStr` for persistence.
  - Add `pub enum AppMode { Popup, Manager }`.
  - Add `pub enum FocusTarget { List, Search, Modal, None }` (re-exported in the controller modules too, but the canonical definition lives here so the reducer and the focus chain share one enum).
  - Add `pub struct AppState` (plain Rust, **not** `glib::Properties` — that comes in PR 4). Fields for this PR: `mode: AppMode`, `active_page: PageId`, `filter: PickerFilter`, `sort: SortOrder`, `search_query: String`, `selected_index: Option<usize>`, `focus: FocusTarget`, `config: PopupConfig` (default = `PopupConfig::default()`), `manager_config: ManagerConfig`. Use `#[derive(Debug, Clone, Default)]` plus a manual `Default` impl that fills `config` with `PopupConfig::default()`.
  - Add `pub enum Action` for the foundation slice only (CP §PR-3A lines 250–256). Do not include pin/star/delete/window/snippets/daemon actions — those ship in PR 3B.
  - Add `pub enum Effect` for the foundation slice only: `RefreshItems`, `PersistGSettings` (placeholder; PR 4 wires the binding).
  - Add `pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect>` — pure, no I/O, no GTK. The body must be deterministic and idempotent.
  - Add `#[cfg(test)] mod tests` with **one test per `Action` variant** (~12) plus the invariants CP §PR-3A calls out: `MoveBy` is a no-op on empty selection, `QueryChanged("")` is equivalent to `QueryCleared`, `CyclePage(1)` wraps around.
- `crates/ui-gtk/src/lib.rs` (line 27 area).
  - `pub use app::{Action, AppMode, AppState, Effect, PageId, reduce, FocusTarget};` (foundation surface only).
- `specs/features/023-unified-gtk4-ui/07-test-plan.md` — tick the "app::reduce — All Action variants" row partially with a note "PR 3A foundation; PR 3B finishes it".
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T004 marked "foundation only" until PR 3B.

**Acceptance criteria**

- `cargo test -p author-clipboard-ui-gtk -- reduce` — **≥12 tests pass**, no GTK init required (no `gtk::init()` in any test body).
- `cargo test -p author-clipboard-ui-gtk` — green (regression: existing focus, search, clipboard tests still pass).
- `just verify` — green.
- `rg -n 'gtk::init' crates/ui-gtk/src/app.rs` — empty (the reducer module is GTK-free).
- `rg -n '#\[derive\(.*glib::Properties' crates/ui-gtk/src/app.rs` — empty (no GObject derive in this PR; PR 4).
- The 12+ tests assert both state delta (e.g. `state.search_query == "x"`) and effect list (e.g. `effects == vec![Effect::RefreshItems]`).

**Subagent prompt (copy-paste)**

```
You are completing PR 3A of feature 023 on branch
feat/023-popup-bugs. PR 0 + PR 1 + PR 2 are landed; `just
verify` is green. Read
specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-3A
and the four files listed in "Files to touch" above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Rewrite crates/ui-gtk/src/app.rs (75 → ~400 LOC). Delete
     the existing 75 LOC entirely; do not preserve KeyAction /
     FilterState / SortOrder. Default for AppState must use
     PopupConfig::default() and ManagerConfig::default().
     CyclePage wraps via mod. Add the 12 test cases.
  3. Update crates/ui-gtk/src/lib.rs with the re-exports.
  4. Re-export FocusTarget from app in controller/focus.rs
     (single source of truth).
  5. Tick 06-task-plan.md T004 → "foundation only" and
     07-test-plan.md "app::reduce — All Action variants"
     partially.
  6. Run the four cargo invocations + `just verify` from
     "Acceptance criteria".
  7. Commit:
       feat(ui-gtk): introduce AppState + Action + reduce() foundation

       Foundation slice of the state machine: PageId, AppMode,
       FocusTarget, AppState, Action (12 variants), Effect
       (2 variants), reduce() with ≥12 unit tests. GTK-free.

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for
this PR is: do not add pin/star/delete/reveal/window/snippets/
daemon actions (PR 3B), do not use glib::Properties (PR 4),
do not wire the reducer to any GTK widget, do not change
PopupConfig or ManagerConfig in lib.rs, do not touch
pages/clipboard.rs / window/popup.rs / window/manager.rs, do
not invent AppState fields the plan does not list.

Rollback: revert one commit. Additive; placeholder enums
were unused.
```
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit; spec-file ticks in same commit.

**Rollback risk**: Low. The new `app` module is additive; nothing
in the GTK widget tree imports it yet.

**Decision updates**: none required. If a new field or invariant
is added beyond the plan, append a D-note.

---

### PR 3B — T004 (completion): full reducer coverage

- **Title**: `feat(ui-gtk): complete reducer with pin/star/delete/reveal/window/settings/snippets/daemon actions`
- **Estimated LOC delta**: +500 / −20 (in `crates/ui-gtk/src/app.rs`).
- **Spec references**: CP §PR-3B; `10-completion-plan.md` lines 285–328.

**Pre-PR verification**

```bash
cargo test -p author-clipboard-ui-gtk -- reduce
just verify
```

**Files to touch**

- `crates/ui-gtk/src/app.rs` (extend PR 3A).
  - **Extend** `AppState` (additive fields, no renames):
    - `sort: SortOrder` (already in PR 3A)
    - `show_redacted: bool` (default `false`)
    - `reveal_countdown: u8` (default `0`)
    - `daemon_running: bool` (default `true` until ping says otherwise)
    - `incognito: bool` (default `false`; reads the sentinel file at startup in PR 4, not here)
    - `items: Vec<ClipboardItem>` (default `vec![]`)
    - `snippets: Vec<Snippet>` (default `vec![]`)
  - **Add** Action variants (CP §PR-3B lines 297–304): `CopyRequested`, `QuickPasteRequested`, `TogglePin(i64)`, `ToggleStar(i64)`, `Delete(i64)`, `RevealRedacted`, `HideRedacted`, `RevealTick`, `SetDaemonRunning(bool)`, `ItemsLoaded(Vec<ClipboardItem>)`, `SnippetsLoaded(Vec<Snippet>)`, `Toast(String)`, `Quit`, `IncognitoToggled(bool)`, `WindowResized(i32, i32)`, `WindowPageChanged(PageId)`.
  - **Add** Effect variants: `CopyItem { id, mode, mime }`, `QuickPasteItem { id, mime }`, `PinItem`, `UnpinItem`, `StarItem`, `UnstarItem`, `DeleteItem`, `ClearUnpinned`, `RefreshSnippets`, `AddToast`, `PersistConfig`, `Quit`.
  - Extend `reduce` with the new handlers. **Keep the reducer pure.** The runtime (PR 4+) is responsible for I/O.
  - **`WindowResized` does NOT coalesce** in the reducer — debouncing belongs in the runtime (CP invariant).
  - **`RevealTick` decrements `reveal_countdown` and emits `HideRedacted` when the count hits 0** — covered by the new tests.
  - Extend `#[cfg(test)] mod tests` with **~30 more tests** (one per new Action variant plus invariants). Specifically test: `RevealTick` countdown 5→4→…→0→`HideRedacted` emitted, `WindowResized` does not collapse multiple calls, `IncognitoToggled(true)` reflects in state.
- `crates/ui-gtk/src/lib.rs` — no new re-exports (the new variants are still under `Action` / `Effect`).
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T004 row → ✅.
- `specs/features/023-unified-gtk4-ui/07-test-plan.md` — finish ticking the "app::reduce — All Action variants" row.

**Acceptance criteria**

- `cargo test -p author-clipboard-ui-gtk -- reduce` — **≥40 tests pass** (12 from PR 3A + ≥28 new).
- All new `Action` variants have at least one direct test (`Action::RevealTick` has 6 — one per countdown tick).
- `cargo test -p author-clipboard-ui-gtk` — green.
- `just verify` — green.
- `rg -n 'glib::timeout_add' crates/ui-gtk/src/app.rs` — empty (no debounce / no GLib in the reducer).
- T004 row in `06-task-plan.md` is fully ✅.

**Subagent prompt (copy-paste)**

```
You are completing PR 3B of feature 023 on branch
feat/023-popup-bugs. PR 3A landed; reducer foundation is in
place with 12 tests. Read
specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-3B
and the four files listed in "Files to touch" above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Extend AppState (additive only) and add the 17 new
     Action variants + 12 new Effect variants. Match the
     exact names in CP §PR-3B lines 297–304 / 304–307 — the
     runtime in PR 4+ will pattern-match on these.
  3. Extend reduce. Shape (the agent writes the full table):
       Action::RevealTick => { /* decrement, emit
         HideRedacted at 0 */ }
       Action::WindowResized(w, h) => { /* no coalesce;
         PersistConfig */ }
       Action::TogglePin(id) => { /* flip pinned, emit
         PinItem or UnpinItem */ }
  4. Add ≥28 new tests. The 4 reviewer invariants:
     RevealTick → HideRedacted at 0; WindowResized does not
     coalesce; IncognitoToggled flips state; SetDaemonRunning
     is observable.
  5. Tick 06-task-plan.md T004 → ✅ and 07-test-plan.md fully.
  6. Run the four cargo invocations + `just verify` from
     "Acceptance criteria".
  7. Commit:
       feat(ui-gtk): complete reducer (pin/star/delete/reveal/
       window/settings/snippets/daemon)

       Closes T004. ~30 new tests. Reducer remains pure;
       runtime is responsible for debouncing and I/O.

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for
this PR is: do not introduce GLib timers / channels / async
into the reducer, do not call IPC / DB / filesystem, do not
collapse WindowResized events, do not add fields to AppState
the plan does not list, do not "improve" the foundation
surface (PR 3A is locked).

Rollback: revert one commit. AppState is additive; PR 4+ has
not migrated any call sites yet.
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit; spec-file ticks in same commit.

**Rollback risk**: Low. Pure additive; no runtime yet.

**Decision updates**: none required. If a new Effect is required
mid-implementation (e.g. `PersistManagerConfig` separate from
`PersistConfig`), append a D-note to `09-decisions.md`.

---

### PR 4 — T005/T006: real Esc + global key controller + GSettings binding

- **Title**: `feat(ui-gtk): real Esc controller, GSettings binding, key resolver`
- **Estimated LOC delta**: +450 / −90 (in `crates/ui-gtk/src/{controller,settings,actions}.rs`).
- **Spec references**: CP §PR-4; `10-completion-plan.md` lines 330–379.

**Pre-PR verification**

```bash
cargo test -p author-clipboard-ui-gtk -- reduce
just verify
rg -n 'IpcCommand::Copy' crates/   # confirm last call-site inventory (no change this PR)
```

**Files to touch**

- `crates/ui-gtk/src/settings.rs` (rewrite from 94 LOC).
  - Typed accessors: `pub fn filter() -> PickerFilter` (parses the stored `String` via `PickerFilter::from_str`, falling back to `All` on parse error), `set_filter(PickerFilter)`, `pub fn sort() -> SortOrder`, `set_sort(SortOrder)`, `pub fn last_page() -> PageId`, `set_last_page(PageId)`, `pub fn popup_size() -> (i32, i32)`, `set_popup_size(w, h)`, `pub fn window_size() -> (i32, i32)`, `set_window_size(w, h)`.
  - **Delete** the string-typed accessors. They are unused (no consumer in the tree).
  - **Add** `pub struct SettingsBinding { settings: Settings, state: Rc<RefCell<AppState>> }` with `pub fn new(state: Rc<RefCell<AppState>>) -> Option<Self>`. Wires:
    - On construction: read every key from `gio::Settings`, apply to `state` via `reduce(...)` (or direct field set, since this is boot, not user input).
    - On each `gio::Settings` `changed` signal: emit an `Action::*` that the runtime forwards back to `reduce`.
    - On `Effect::PersistGSettings`: write the relevant state field back to the schema.
  - `pub fn SettingsBinding::persist(&self, effect: &Effect)` — single dispatch, no I/O outside the schema write.
- `crates/ui-gtk/src/controller/key.rs` (extend 43 LOC).
  - **Add** `pub fn map_key_extended(key: gdk::Key, mods: gdk::ModifierType) -> Option<crate::app::Action>` covering the full US-005 shortcut table:
    - `gdk::Key::slash` (no modifier) → `Action::Focus(FocusTarget::Search)`
    - `gdk::Key::question` → `Action::ShowShortcuts` (new action — add to `app.rs` as a foundation variant; if it conflicts with PR 3A's locked list, record D-note)
    - `gdk::Key::Escape` → `Action::Focus(FocusTarget::List)` (the runtime will then consult `focus.rs::resolve_escape` for the actual outcome; this keeps the reducer source-of-truth)
    - `gdk::Key::Up`/`Down`/`Left`/`Right`/`Home`/`End`/`Page_Up`/`Page_Down` → `MoveBy` / `MoveTo` / `MovePage`
    - `gdk::Key::Return` / `gdk::Key::KP_Enter` → `Enter` (runtime decides copy vs quick-paste based on config)
    - `gdk::Key::F1` (no modifier) → `ShowShortcuts`
    - `Ctrl+1..9` → `MoveTo(usize)` where the digit - 1 is the index
    - `Ctrl+Tab` / `Ctrl+Shift+Tab` → `CyclePage(1)` / `CyclePage(-1)`
  - **Add** tests in `#[cfg(test)] mod tests` — pure data: assert every shortcut's mapping. **No GTK init required** for the resolver (the function takes `gdk::Key` + `gdk::ModifierType` as values, not as widget events). Use `cfg!(test)` and the bare enum values.
- `crates/ui-gtk/src/controller/focus.rs` (extend 78 LOC).
  - **Real** `pub fn install(window: &impl IsA<Widget>, state: Rc<RefCell<AppState>>, effects_tx: glib::Sender<Effect>) -> EventControllerKey`. Capture-phase controller. Inside `connect_key_pressed`, call `crate::controller::key::map_key_extended(key, modifier)`. If `Some(action)`, dispatch through `reduce(&mut state.borrow_mut(), action)`, drain the returned `Vec<Effect>`, and forward each to `effects_tx`. **Do not** call `widget.grab_focus()` etc. directly — the reducer is the source of truth and the runtime will apply focus effects.
  - Keep the existing `FocusTarget` / `EscOutcome` / `resolve_escape` resolver but re-export `FocusTarget` from `crate::app`.
- `crates/ui-gtk/src/controller/search.rs` (rewrite from 6 LOC).
  - `pub struct SearchDebounce { pending: String, last_change: Instant, source: Option<glib::SourceId> }` with `Rc<RefCell<SearchDebounce>>` in the controller.
  - The timer-firing callback is factored out: `fn debounce_apply(pending: &str, now: Instant, last_change: Instant) -> Option<String>`. **Unit-test this pure function with a fake clock** (no `glib::timeout_add_local` in the test).
  - The runtime call is `glib::timeout_add_local(Duration::from_millis(DEBOUNCE_MS), move || { debounce_apply(...); glib::ControlFlow::Break })`.
  - **Replace** the existing `Rc<Cell<String>>` debounce with `Rc<RefCell<SearchDebounce>>`.
  - **Delete** the `controller/search.rs::DEBOUNCE_MS` const if duplicated in `widgets/search.rs` — keep one canonical value.
- `crates/ui-gtk/src/actions.rs` (currently 1 LOC).
  - `pub fn register(app: &adw::Application, state: Rc<RefCell<AppState>>, effects_tx: glib::Sender<Effect>)` registers GAction entries: `set-filter`, `set-search`, `set-page`, `toggle-pin`, `delete`, `toggle-star`, `reveal`, `quit`, `prev-page`, `next-page`. Each handler dispatches the matching `Action` and forwards the resulting `Effect`s to `effects_tx`.
- `crates/ui-gtk/src/window/popup.rs` (no change to the inline Esc controller in this PR; that gets deleted in PR 6 when the new controller is wired in).
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T005 + T006 → ✅.

**Acceptance criteria**

- `cargo test -p author-clipboard-ui-gtk -- key focus search` — all pass; the resolver tests run **without** `gtk::init()`.
- `cargo test -p author-clipboard-ui-gtk -- reduce` — regression green.
- `cargo test -p author-clipboard-ui-gtk` — green.
- `just verify` — green.
- `rg -n 'gtk::init' crates/ui-gtk/src/controller/` — only in widget-construction tests, **not** in the resolver tests.
- T005 and T006 rows in `06-task-plan.md` are fully ✅.

**Subagent prompt (copy-paste)**

```
You are completing PR 4 of feature 023 on branch
feat/023-popup-bugs. PR 3A + PR 3B landed; reducer has ≥40
tests and a stable Action/Effect surface. Read
specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-4
and the eight files listed in "Files to touch" above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Rewrite settings.rs (typed accessors + SettingsBinding).
     Delete the unused string-typed accessors; fall back
     gracefully on a missing schema.
  3. Extend key.rs with map_key_extended (pure function) +
     ≥10 resolver tests. Do not touch EventControllerKey.
  4. Extend focus.rs with install() that wires
     map_key_extended + reduce + Effect forwarding. Re-export
     FocusTarget from crate::app.
  5. Rewrite search.rs: factor debounce_apply as a pure fn;
     ≥3 tests with a fake clock. The GLib callback is a
     thin shell around the pure fn.
  6. Populate actions.rs: register() for 10 GAction entries
     using glib::Sender<Effect>.
  7. Tick 06-task-plan.md T005 + T006 → ✅.
  8. Run the five cargo invocations + `just verify` from
     "Acceptance criteria".
  9. Commit:
       feat(ui-gtk): real Esc controller, GSettings binding,
       key resolver

       Closes T005 + T006. Resolver is pure; runtime is
       responsible for I/O. Reducer remains the single source
       of truth.

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for
this PR is: do not change the Action/Effect surface (if a
new foundation variant is needed, add it to app.rs and flag
in the commit body), do not call IPC / DB / filesystem from
the resolver, do not delete the inline Esc controller in
popup.rs (PR 6 does that), do not introduce new
dependencies.

Rollback: revert one commit. Additive; old string-typed
settings accessors were unused.
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit.

**Rollback risk**: Medium. The new key controller is additive; the
old inline Esc handler in `popup.rs` stays until PR 6. New
GSettings binding reads new keys; if the schema is missing the
runtime falls back to `AppState` defaults (`Settings::new()` returns
`Option`).

**Decision updates**: Add **D16** to `09-decisions.md` if the
agent had to add a new foundation Action (e.g. `ShowShortcuts`)
beyond the CP's locked list. Otherwise no new decision.

---

### PR 5 — T010 (no WebKit): `PreviewPane` for text / image / files / sensitive

- **Title**: `feat(ui-gtk): PreviewPane for text / image / files / sensitive`
- **Estimated LOC delta**: +450 / −0 (new module; `widgets/preview.rs` is a 3-LOC stub).
- **Spec references**: CP §PR-5; `10-completion-plan.md` lines 381–429.

**Pre-PR verification**

```bash
cargo test -p author-clipboard-ui-gtk -- reduce
just verify
rg -n 'webkit6' crates/   # must be empty or commented
```

**Files to touch**

- `crates/ui-gtk/src/widgets/preview.rs` (rewrite from 3 LOC).
  - `pub struct PreviewPane` holding: `state: Rc<RefCell<AppState>>`, `widget: gtk::Box` (or `adw::Bin` root), `text_view: sourceview5::View`, `image: gtk::Picture`, `files_box: gtk::Box`, `redacted_overlay: adw::StatusPage`, `reveal_button: gtk::Button`, `empty: adw::StatusPage`.
  - `pub fn new(state: Rc<RefCell<AppState>>) -> Self`.
  - **Subscribes** to `state.selected_index` and `state.items`. In this PR, the subscription is a manual `connect_items_loaded` callback exposed via a method `pub fn on_items_loaded(&self, items: Vec<ClipboardItem>)` (the runtime calls it after IPC returns; wiring into a real signal is PR 6). A unit test calls it directly.
  - **ContentType::Text** → `sourceview5::View` (read-only, monospace, soft-wrap; `set_editable(false)`).
  - **ContentType::Image** → `gtk::Picture` backed by `gdk_pixbuf::Pixbuf::from_file_at_scale(path, 800, 600, true)`; thumbnail if available.
  - **ContentType::Html** → escape and render as `sourceview5::View` with `language-html` highlighting. **PR 5.5 swaps this for WebView behind the `webview` feature flag** — keep the escape-then-render path for now.
  - **ContentType::Files** → list of `AdwActionRow`s with file name + size; click opens via `gio::AppInfo::launch_default_for_uri`.
  - **Redaction**: when `sensitive && !show_redacted`, render an `AdwStatusPage` with `lock.svg` (from the existing 23 SVGs in `assets/icons/`), the redacted preview, and a "Reveal (5s)" button. `Action::RevealRedacted` starts the countdown; `Action::RevealTick` decrements every second; `Action::HideRedacted` reverts. A `chip-warning` shows the remaining seconds.
  - **Empty state**: `AdwStatusPage` with `empty-clipboard.svg` and "Select an item to preview".
  - **No `webkit6` import in this PR** (D13). The default build stays WebKit-free.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T010 row marked "no HTML preview" until PR 5.5.
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — confirm D13 entry exists; **no new decision** for this PR (PR 5.5 records the WebKit opt-in).

**Acceptance criteria**

- `cargo test -p author-clipboard-ui-gtk -- preview` — all pass. Widget construction tests **may** use `gtk::init()`; they must be `#[ignore]`-d or feature-gated (`#[cfg(feature = "widget-tests")]`) when no display is available. The default `cargo test` invocation **must not** require a display.
- `cargo test -p author-clipboard-ui-gtk -- reduce` — regression green.
- `cargo test -p author-clipboard-ui-gtk` — green.
- `just verify` — green.
- `rg -n 'webkit6' crates/ui-gtk/src/widgets/preview.rs` — **empty**.
- `rg -n 'webkit6' crates/ui-gtk/Cargo.toml` — `webkit6` is **commented out** (the line `# webkit6.workspace = true` stays commented; PR 5.5 uncomments + feature-gates).
- T010 row in `06-task-plan.md` is "no HTML preview" until PR 5.5.

**Subagent prompt (copy-paste)**

```
You are completing PR 5 of feature 023 on branch
feat/023-popup-bugs. PR 4 landed; controller and GSettings
are in place. Read
specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-5
and the six files listed in "Files to touch" above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Rewrite widgets/preview.rs (3 → ~450 LOC). Keep the pub
     surface minimal (new + on_items_loaded + widget()).
  3. The redacted state reads from state.borrow().show_redacted
     and state.borrow().reveal_countdown. Click "Reveal (5s)"
     dispatches Action::RevealRedacted; RevealTick decrements;
     HideRedacted flips back.
  4. Widget construction tests: any test that calls
     PreviewPane::new must use #[ignore] or
     #[cfg(feature = "widget-tests")]; default test run must
     not require a display.
  5. Tick 06-task-plan.md T010 → "no HTML preview". Confirm
     D13 in 09-decisions.md; add if missing.
  6. Run the four cargo invocations + `just verify` from
     "Acceptance criteria".
  7. Commit:
       feat(ui-gtk): PreviewPane for text / image / files /
       sensitive

       Closes T010 (no HTML preview — PR 5.5 adds WebView).
       ~450 LOC widget, no webkit6 import. Default build
       stays WebKit-free.

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for
this PR is: do not uncomment the webkit6 line, do not add a
feature flag, do not change AppState / Action / Effect
(PR 3B is locked), do not wire PreviewPane into the manager,
do not introduce new deps.

Rollback: revert one commit. New module; no call sites yet.
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit.

**Rollback risk**: Low (new widget, no existing call site).

**Decision updates**: Confirm **D13** exists in `09-decisions.md`.
If not, add it (one paragraph: "webkit6 is feature-gated behind
`--features webview`; default PR 5 ships no WebKit; PR 5.5
adds it").

---

### PR 5.5 — T010 (WebKit): optional HTML preview behind `webview` feature

- **Title**: `feat(ui-gtk): optional HTML preview via webkit6 behind webview feature`
- **Estimated LOC delta**: +80 / −10 (Cargo.toml + widgets/preview.rs).
- **Spec references**: CP §PR-5.5; `10-completion-plan.md` lines 431–467.

**Pre-PR verification**

```bash
cargo test -p author-clipboard-ui-gtk -- preview
just verify    # default build (no webkit)
which dnf apt pacman 2>/dev/null   # dev host package manager (informational only)
```

**Files to touch**

- `crates/ui-gtk/Cargo.toml`.
  - Add `[features] webview = ["dep:webkit6"]` to the `[features]` table.
  - Change `# webkit6.workspace = true` (the commented line) to `webkit6 = { workspace = true, optional = true }`.
- `crates/ui-gtk/src/widgets/preview.rs` (extend PR 5).
  - Add a `#[cfg(feature = "webview")] fn render_html_with_webview(text: &str) -> impl IsA<gtk::Widget>` that constructs a `webkit6::WebView`, sets up a `WebContext` with `set_sandbox_enabled(true)`, and loads via `data:text/html;base64,…` URL. **No unconditional `use webkit6::*;` at the top of the file.**
  - In the `ContentType::Html` arm, dispatch:
    ```rust
    #[cfg(feature = "webview")]
    { render_html_with_webview(&html) }
    #[cfg(not(feature = "webview"))]
    { /* keep the PR 5 sourceview fallback */ }
    ```
  - WebView construction test: `#[cfg(all(test, feature = "webview"))] mod webview_tests` — single test that the WebContext is sandboxed. `#[ignore]` if no display.
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — add a one-line note: "PR 5.5 confirms D13; webkit6 is now feature-gated behind `--features webview`; default build is unchanged."
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T010 row → ✅.

**Acceptance criteria**

- `cargo build -p author-clipboard-ui-gtk` — default build, **no `webkitgtk-6.0-dev` required**, green.
- `cargo build -p author-clipboard-ui-gtk --features webview` — opt-in build, requires the system library, green on the maintainer's local box.
- `cargo test -p author-clipboard-ui-gtk --features webview -- preview` — WebContext test passes (or is `#[ignore]`-d if no display).
- `cargo test -p author-clipboard-ui-gtk` — default green.
- `just verify` — green (it runs the default test, not the opt-in).
- `rg -n 'use webkit6' crates/ui-gtk/src/widgets/preview.rs` — **only** inside the `#[cfg(feature = "webview")]` block.
- T010 row in `06-task-plan.md` is fully ✅.

**Subagent prompt (copy-paste)**

```
You are completing PR 5.5 of feature 023 on branch
feat/023-popup-bugs. PR 5 landed. Read
specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-5.5
and the four files listed in "Files to touch" above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Edit Cargo.toml: add [features] webview = ["dep:webkit6"]
     and flip the dep to optional.
  3. Edit widgets/preview.rs:
     - All webkit6 imports live inside
       #[cfg(feature = "webview")] blocks. NO unconditional
       use at the top of the file.
     - Branch the ContentType::Html arm on the feature.
     - Any webview test is wrapped in
       #[cfg(all(test, feature = "webview"))] and marked
       #[ignore] if it constructs a widget.
  4. Tick 06-task-plan.md T010 → ✅ and append a one-line
     note to D13 in 09-decisions.md.
  5. Run:
       cargo build -p author-clipboard-ui-gtk   # default
       cargo build -p author-clipboard-ui-gtk --features webview
         (skip if webkitgtk-6.0-dev missing; record in
         commit body)
       cargo test -p author-clipboard-ui-gtk
       just verify
  6. Commit:
       feat(ui-gtk): optional HTML preview via webkit6
       behind webview feature

       Closes T010. Default build unchanged; maintainer
       opts in with --features webview. WebContext sandboxed.

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for
this PR is: do not unconditionally import webkit6 (CI
breaks), do not add a WebView field to the PreviewPane
struct directly, do not change Action / Effect / AppState
(PR 3B is locked), do not touch the manager (PR 6 does).

Rollback: revert one commit. Feature flag is additive;
removing the line restores PR 5 state.
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit.

**Rollback risk**: Low. Feature is additive; default build is
untouched.

**Decision updates**: Append a one-line note to **D13** in
`09-decisions.md` (do not allocate D17). The note records that
PR 5.5 added the WebKit opt-in.

---

### PR 6 — T013/T015: manager rewrite + persisted size + preview wiring

- **Title**: `feat(ui-gtk): manager rewrite with AdwNavigationView + sidebar + persisted size`
- **Estimated LOC delta**: +400 / −110 (in `crates/ui-gtk/src/window/manager.rs`, `crates/ui-gtk/src/window/popup.rs`).
- **Spec references**: CP §PR-6; `10-completion-plan.md` lines 469–534.

**Pre-PR verification**

```bash
cargo test -p author-clipboard-ui-gtk
just verify
rg -n 'AdwNavigationView' crates/ui-gtk/   # currently empty (manager uses ViewStack)
```

**Files to touch**

- `crates/ui-gtk/src/window/manager.rs` (rewrite from 132 LOC).
  - `AdwApplicationWindow` with `AdwToolbarView`.
  - `AdwNavigationView` + 6 `AdwNavigationPage`s: Clipboard, Emoji, Symbols, Kaomoji, Snippets, Settings.
  - **Sidebar visible at widths > 900 px.** If the current libadwaita version exposes a sidebar primitive (e.g. `adw::Sidebar`, `adw::OverlaySplitView`), use it. If not, build the sidebar from `gtk::Box` + `gtk::ListBox` with row icons — record the choice in **D15** (D15 is already allocated in the plan, see CP §PR-6 line 516).
  - **Clipboard page** is the only one that mounts `PreviewPane` next to the list, in a `Paned` (60% / 40%). Below 900 px the preview collapses to a modal sheet.
  - **Persistence**: read `(window_width, window_height)` from GSettings on startup; on `close-request` and on `size-allocate` (debounced 500 ms via `glib::timeout_add_local`) write back. Read `last-page`; jump to it on startup.
  - **Esc**: same chain as the popup — search has focus → `Action::ClearSearch` (or the foundation `Action::Focus(FocusTarget::List)` and let the focus controller resolve); modal open → close modal; list has focus → close window. Driven by `Action::Focus` / `Action::Quit` so the reducer is the source of truth.
  - **Status bar**: item count, pinned count, daemon indicator, reveal countdown chip when active.
  - **Toast overlay**: wrap the toolbar view; `Effect::AddToast` becomes `overlay.add_toast(adw::Toast::new(&msg))`.
- `crates/ui-gtk/src/window/popup.rs` (update).
  - **Delete** the inline `EventControllerKey` Esc blob (lines 95–129 of the current file). Replace with the global key controller from PR 4 (`crate::controller::focus::install`).
  - **Delete** the inline `EventControllerKey` `/` blob (lines 132–146). The new key controller handles `/` via `map_key_extended`.
  - **Read popup size from GSettings** on startup; **write back on `size-allocate`** (debounced 500 ms).
  - Initialize search text from `PopupConfig.query` (already done in PR 1).
  - Initialize `FilterBar` from `PopupConfig.filter` (already done in PR 1).
  - Initialize `PageState` from `PopupConfig` (already done in PR 1).
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — T013 + T015 → ✅.
- `specs/features/023-unified-gtk4-ui/08-review-checklist.md` — tick the applicable rows (US-003, "Manager window is a real `AdwApplicationWindow`", "Filter survives popup→manager", "Settings persist").
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — confirm **D15** is recorded with the sidebar primitive actually used. If the maintainer went with the `gtk::Box` + `gtk::ListBox` fallback, the rationale must include the missing libadwaita version and the crates.io search that confirmed the absence.

**Acceptance criteria**

- `cargo test -p author-clipboard-ui-gtk` — green.
- `cargo test -p author-clipboard-ui-gtk -- reduce preview key focus search` — all green.
- `just verify` — green.
- `rg -n 'EventControllerKey' crates/ui-gtk/src/window/popup.rs` — only the global one from PR 4, no inline `connect_key_pressed` for Esc or `/`.
- `rg -n 'ViewStack|ViewSwitcher' crates/ui-gtk/src/window/manager.rs` — empty (full `AdwNavigationView` rewrite, no fallback).
- `08-review-checklist.md` has the applicable rows ticked (US-003, persistence, filter survival).
- D15 in `09-decisions.md` records the actual sidebar primitive (or fallback).
- Manual: open manager, resize, close, reopen at the same size; toggle Pinned in popup, open manager, filter still Pinned; click into Settings, change a row, close, reopen, value persists.

**Subagent prompt (copy-paste)**

```
You are completing PR 6 of feature 023 on branch
feat/023-popup-bugs. PR 5.5 landed; widget tree, controller,
and reducer are in place. Read
specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-6
and the eight files listed in "Files to touch" above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Inspect the libadwaita crate version in Cargo.lock.
     Try adw::Sidebar / adw::OverlaySplitView first; if
     neither compiles, fall back to gtk::Box + gtk::ListBox
     with a CSS class. Record the choice in D15.
  3. Rewrite window/manager.rs (132 → ~400 LOC). Keep
     run(config: ManagerConfig) -> Result<()> intact. Mount
     PreviewPane in a Paned 60/40. Persist window size + last
     page via GSettings (debounce via
     glib::timeout_add_local — NOT in the reducer).
  4. Update window/popup.rs: delete the inline Esc and /
     controllers; wire crate::controller::focus::install.
     The / delete fixes the latent slash-swallow bug. Persist
     popup size via GSettings.
  5. Tick 06-task-plan.md T013 + T015 → ✅ and the
     applicable 08-review-checklist.md rows. Update D15.
  6. Run:
       cargo test -p author-clipboard-ui-gtk
       cargo test -p author-clipboard-ui-gtk -- reduce
         preview key focus search
       just verify
  7. Commit:
       feat(ui-gtk): manager rewrite with AdwNavigationView
       + sidebar + persisted size

       Closes T013 + T015. Sidebar primitive recorded in D15.
       Popup's inline Esc and / controllers deleted; the
       latent / -swallow bug is fixed.

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for
this PR is: do not add a ViewSwitcher / ViewStack fallback,
do not change AppState / Action / Effect (if a new field is
needed append a D-note and flag in the commit body), do not
debounce WindowResized in the reducer, do not introduce new
deps, do not re-introduce libcosmic.

Rollback: revert one commit. The pre-023-ui-rewrite tag
preserves the old manager.
```
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit; spec-file ticks in the same commit.

**Rollback risk**: Medium. The manager is rewritten; old applet is
preserved at the `pre-023-ui-rewrite` tag. The popup also loses
its inline Esc and `/` controllers, so reverting the PR leaves
the popup without them — the maintainer must also revert the
`popup.rs` changes if a true rollback is needed (the next agent
should `git revert` and verify the popup still opens).

**Decision updates**: **D15** in `09-decisions.md` records the
sidebar primitive actually used (and why, if it was a fallback).

---

### PR 7 — T017 + T019 + T020: CLI parity + smoke + docs

- **Title**: `feat(ui): hypr-picker --filter, smoke scenarios, docs and snapshots`
- **Estimated LOC delta**: +250 / −20 (in `crates/hypr-picker/`, `crates/ui-gtk/tests/`, `justfile`, `docs/`, `README.md`).
- **Spec references**: CP §PR-7; `10-completion-plan.md` lines 536–585.

**Pre-PR verification**

```bash
cargo test -p author-clipboard-hypr-picker
just verify
ls docs/UI/snapshots/ 2>/dev/null   # currently empty
```

**Files to touch**

- `crates/hypr-picker/src/main.rs` (extend 92 LOC).
  - Add `--filter` flag mapped to `PickerFilter` (parse via `PickerFilter::from_str`, fall back to `All` on parse error). Default `all`. Preserves legacy flags.
  - At line 84 (`filter: PickerFilter::All`), use the parsed value.
- `crates/ui-gtk/tests/smoke.sh` (extend the existing 53-LOC script).
  - Add scenarios:
    - `/` + type: `xdotool key slash; xdotool type "git"` — assert list filters.
    - Esc-then-Esc close: `xdotool type "x"; xdotool key Escape; xdotool key Escape` — assert search empty, then window closed.
    - Pinned filter persistence: open manager, set filter to Pinned, kill, reopen, assert still Pinned.
    - Manager opens to last page: open manager, navigate to Settings, kill, reopen, assert on Settings.
    - Sensitive reveal countdown: `xdotool key ctrl+shift+r` — assert countdown chip visible for 5s.
  - Each scenario saves a screenshot to `docs/UI/snapshots/`.
- `justfile` (add; **not** wired into `verify`):
  - `ui-check`: `glib-compile-schemas crates/ui-gtk/data/ && cargo check -p author-clipboard-ui-gtk`. Fails if the schema is stale.
  - `ui-smoke`: `xvfb-run -a crates/ui-gtk/tests/smoke.sh`. Saves screenshots to `docs/UI/snapshots/`.
  - `ui-test`: `cargo test -p author-clipboard-ui-gtk`.
- `docs/UI.md` (extend the existing 60+ LOC).
  - New "PreviewPane" section.
  - New "State machine" section with the reducer's Action table (copy from `crates/ui-gtk/src/app.rs`).
  - New "GSettings" section listing the schema IDs and the keys bound to `AppState`.
- `docs/UI/snapshots/` — commit 5 PNGs (popup, manager, clipboard-page, settings, sensitive-reveal). The maintainer runs `just ui-smoke` locally to produce them; the subagent cannot generate screenshots in this environment.
- `README.md` — embed 2 inline screenshots (popup + manager).
- `specs/features/023-unified-gtk4-ui/08-review-checklist.md` — tick every applicable row.
- `specs/features/023-unified-gtk4-ui/06-task-plan.md` — all 20 rows → ✅.
- `specs/features/023-unified-gtk4-ui/09-decisions.md` — **D14**: hypr-picker extended with `--filter`; `ui-check` / `ui-smoke` are manual-only.

**Acceptance criteria**

- `just verify` — green.
- `cargo test -p author-clipboard-hypr-picker` — green.
- `rg -n 'PickerFilter::All' crates/hypr-picker/src/main.rs` — only one occurrence, replaced by the parsed value.
- `ls docs/UI/snapshots/` — 5 PNGs (popup, manager, clipboard-page, settings, sensitive-reveal).
- `grep -c "docs/UI" README.md` — ≥ 2.
- `just ui-check` — green on the maintainer's local box (manual; not in CI).
- `just ui-smoke` — green on the maintainer's local box (manual; not in CI).
- `author-clipboard-hypr-picker --filter pinned` — shows only pinned entries (manual).
- All 20 rows of `06-task-plan.md` are ✅.
- All applicable rows of `08-review-checklist.md` are ticked.

**Subagent prompt (copy-paste)**

```
You are completing PR 7 of feature 023 on branch
feat/023-popup-bugs. PR 6 landed. Read
specs/features/023-unified-gtk4-ui/10-completion-plan.md §PR-7
and the six files listed in "Files to touch" above.

Steps:
  1. Run the pre-flight block at the top of this plan.
  2. Add --filter to hypr-picker/src/main.rs (parsed via
     PickerFilter::from_str, default "all").
  3. Extend tests/smoke.sh with 5 scenarios: / + type,
     Esc-then-Esc, Pinned persistence, manager opens to
     last page, sensitive reveal countdown. Save outputs to
     docs/UI/snapshots/.
  4. Add ui-check, ui-smoke, ui-test recipes to justfile.
     DO NOT add them to `verify`.
  5. Add PreviewPane, State machine, GSettings sections to
     docs/UI.md.
  6. Create docs/UI/snapshots/ with 5 .gitkeep placeholders.
     You cannot run Xvfb in this environment; document the
     maintainer's workflow (just ui-smoke before merging)
     in D14.
  7. Embed 2 inline screenshots in README.md.
  8. Tick 06-task-plan.md all 20 rows, the applicable
     08-review-checklist.md rows, and add D14 to
     09-decisions.md.
  9. Run:
       cargo test -p author-clipboard-hypr-picker
       cargo test -p author-clipboard-ui-gtk
       just verify
       bash -n crates/ui-gtk/tests/smoke.sh
  10. Commit:
       feat(ui): hypr-picker --filter, smoke scenarios, docs
       and snapshots

       Closes T017, T019, T020. ui-check and ui-smoke are
       manual-only. 5 snapshot placeholders; maintainer runs
       just ui-smoke before merging.

Cross-cutting rules: see §4. The "WHAT NOT TO DO" list for
this PR is: do not add ui-check / ui-smoke to `verify` (D14),
do not generate screenshots in this environment, do not
change the IPC layer / reducer / manager (all locked), do
not introduce new deps.

Rollback: revert one commit. Pure additive.
```

**Post-PR branch / commits**

- Branch: continue on `feat/023-popup-bugs`.
- Commit style: one feature commit; spec-file ticks in the same commit.

**Rollback risk**: Low (additive CLI flag; smoke test gated;
docs only).

**Decision updates**: **D14** in `09-decisions.md` records the
`--filter` flag and the manual-only nature of `ui-check` /
`ui-smoke`.

---

## 4. Shared subagent rules (boilerplate)

> Every per-PR "WHAT NOT TO DO" list points back at the rules below.
> Verbatim from `10-completion-plan.md` §"Execution rules", with
> call-site inventory from the PR 0 audit.

1. **Each PR must compile independently.** No "I'll fix the breakage
   in the next PR" — finish the slice or revert. `just verify` must
   be green at the end.
2. **Pre-flight + post-flight `just verify`.** Pre-flight establishes
   the baseline; post-flight must be green before the commit.
3. **Do not invent GTK / libadwaita APIs.** If a desired widget
   doesn't exist in the current crate version (check `Cargo.lock`),
   fall back to `gtk::Box` / `gtk::ListBox` and record the choice in
   `09-decisions.md`.
4. **Tick spec tasks only at the end of the PR that ships them.**
   PR 3A does **not** tick T004 — PR 3B does.
5. **Reducer tests must not require GTK init.** Resolver tests
   (PR 4) take `gdk::Key` + `gdk::ModifierType` as values;
   search-debounce tests pass a fake clock.
6. **Widget construction tests may use `gtk::init()`** and must be
   `#[ignore]`-d or feature-gated (`#[cfg(feature = "widget-tests")]`)
   when no display is available.
7. **IPC changes are atomic.** This PR set introduces no IPC
   changes (PR 1 already landed `mime`). If a subagent adds a new
   variant, update all 8 call sites in one commit:
   `clipboard-daemon/src/main.rs:785`, `shared/src/ipc.rs:693/704/712`,
   `shared/src/picker.rs:662`, `ctl/src/main.rs:528`,
   `ui-gtk/src/pages/clipboard.rs:308`, `mcp-server/src/handler.rs:276`
   (mcp-server is out of scope for these PRs but must keep compiling).
8. **No surprise CI changes.** `ui-check` and `ui-smoke` are
   manual-only. CI stays at `just verify`. PR 7 adds them to the
   justfile but **not** to the `verify` recipe.
9. **`pre-023-ui-rewrite` tag must remain.** Pre-flight includes
   `git tag --list 'pre-023*'` to confirm.
10. **No libcosmic.** Hard ban (D1, D5, `08-review-checklist.md`).
    Do not add `cosmic*` or `iced*` to `Cargo.toml`.
11. **No `gio::ListStore` virtualization** (NFR-002). Out of scope;
    ship the working `ListBox`.

---

## 5. Subagent type assignments

| PR | subagent_type | Justification |
|---|---|---|
| PR 2 | `minmax-subagent` | Multi-file, additive, ~120 LOC, ~16 new tests across `shared/`, `ctl/`, and `ui-gtk/`. Mechanical but the agent must run all 4 cargo invocations and update the spec. |
| PR 3A | `minmax-subagent` | Single-file rewrite (`app.rs`, 75 → ~400 LOC), 12+ tests, lays the foundation. The agent must keep the reducer pure (no GTK init). |
| PR 3B | `minmax-subagent` | Single-file extension, 30+ tests, 17 new Action variants, 12 new Effect variants. The heaviest test surface in the plan; reviewer focus map calls out 4 invariants to check. |
| PR 4 | `minmax-subagent` | Multi-file (4 files), 450 LOC, includes real GSettings binding + GAction registration. The agent must wire `glib::Sender<Effect>` correctly. |
| PR 5 | `minmax-subagent` | Single-file rewrite (`widgets/preview.rs`, 3 → 450 LOC), all four content types + redacted state. Widget construction tests need the `#[ignore]` / `#[cfg(feature = "widget-tests")]` discipline. |
| PR 5.5 | `minmax-subagent` | Two-file change (Cargo.toml + widgets/preview.rs), small LOC, but the feature-gate discipline is critical — an unconditional `use webkit6::*` breaks CI. Worth a full subagent pass. |
| PR 6 | `minmax-subagent` | The biggest UI change (400 LOC manager rewrite + popup cleanup). The agent must inspect the libadwaita crate version and choose the sidebar primitive. The latent `/`-swallow bug fix in popup.rs is also in this PR. |
| PR 7 | `minmax-subagent` | Multi-file (5 files), CLI + tests + docs + justfile. The subagent cannot run Xvfb; it must commit `.gitkeep` placeholders and document the maintainer's workflow. |

**`explore` / `general` assignments**: none for this plan. Every PR
above has a clear spec section in `10-completion-plan.md`; no
exploratory research is needed before the build phase.

---

## 6. Risk register

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | Reducer test suite accidentally depends on `gtk::init()`. | Medium (PR 3A + 3B add ~40 tests) | Medium (breaks `cargo test --workspace` on CI) | PR 3A/3B require `rg -n 'gtk::init' crates/ui-gtk/src/app.rs` to be empty. Reviewer focus map on PR 3B calls this out. |
| 2 | Manager rewrite breaks `run_manager`. | Medium (PR 6 is the biggest UI change) | High (applet cannot launch) | `pre-023-ui-rewrite` tag preserves the old manager. PR 6 requires `git revert` dry-run + green `just verify` before declaring done. |
| 3 | WebKit feature flag silently breaks default build. | Low (PR 5.5 is +80 LOC) | High (CI fails for everyone) | PR 5.5 requires `rg -n 'use webkit6'` to show only `#[cfg(feature = "webview")]` blocks. Default `cargo build` must succeed. |
| 4 | Latent `/` Capture bug survives PR 6. | Medium (PR 6 deletes the inline controller) | Medium (user-visible regression) | PR 6 prompt requires deleting the inline `EventControllerKey` for `/`. The new global controller routes through `map_key_extended` and only `Stop`s on `Some(action)`. The slash character is delivered to the search entry first; the reducer then sets focus. |
| 5 | Sidebar primitive unavailable in current libadwaita. | Medium (depends on `Cargo.lock` version) | Medium (manager loses native sidebar) | PR 6 checks the libadwaita version; falls back to `gtk::Box` + `gtk::ListBox` and records the choice in D15. |
| 6 | `mcp-server` diverges from PR 1's `mime: Option<String>`. | Low (PR 1 already updated all 5 in-scope callers; mcp-server uses `mime: None`) | Low | Pre-flight includes `rg -n 'IpcCommand::Copy' crates/`. mcp-server is out of scope for PRs 2–7. |
| 7 | PR 7 commits snapshot `.gitkeep` placeholders; maintainer forgets to regenerate. | Low | Low (docs show placeholders) | PR 7 records the workflow in D14; the maintainer's `08-review-checklist.md` tick includes "5 PNGs in `docs/UI/`". |
| 8 | libcosmic reintroduction breaks CI. | Low (workspace `Cargo.toml` doesn't list it) | High | Every prompt has "no libcosmic" in WHAT NOT TO DO. Pre-commit hook runs clippy `-D warnings` and would catch the unused dep. |
| 9 | Spec drift in line numbers (e.g. `clipboard.rs:117`). | Low (PR 0 audit done; `docs/023-current-state.md` is fresh) | Low | Pre-flight runs `just verify`; breakage surfaces before the agent commits. |
| 10 | hypr-picker `--filter` overlaps with applet `--filter`. | Low (different binaries) | Low (CLI ambiguity) | Both use the same `PickerFilter::from_str` parse path (CP §Locked decisions). PR 7 updates both `--help` strings. |

---

## 7. Definition of done

Feature 023 is complete when **all** of the following are true:

### Code

- [ ] All 20 spec tasks in `specs/features/023-unified-gtk4-ui/06-task-plan.md` are marked ✅.
- [ ] All applicable rows in `specs/features/023-unified-gtk4-ui/08-review-checklist.md` are ticked.
- [ ] `cargo test --workspace` is green; no GTK init required for the default test run.
- [ ] `just verify` is green (fmt-check, clippy, test, build).
- [ ] `pre-023-ui-rewrite` git tag still exists.

### Reducer / state

- [ ] `crates/ui-gtk/src/app.rs` exposes `AppState`, `Action` (≥28 variants), `Effect` (≥14 variants), `reduce()` with ≥40 unit tests.
- [ ] `crates/ui-gtk/src/controller/key.rs::map_key_extended` covers the full US-005 shortcut table with ≥10 resolver tests.
- [ ] `crates/ui-gtk/src/controller/search.rs` has a factorable `debounce_apply` pure fn with ≥3 tests using a fake clock.
- [ ] `crates/ui-gtk/src/settings.rs` has typed accessors + `SettingsBinding` for `filter`, `sort`, `last_page`, `popup_size`, `window_size`.

### Widgets

- [ ] `crates/ui-gtk/src/widgets/preview.rs` is a 450+ LOC `PreviewPane` covering Text / Image / Html / Files + redacted state + 5s countdown + empty state.
- [ ] `crates/ui-gtk/src/widgets/preview.rs` has no unconditional `use webkit6::*`; the `Html` arm is feature-gated.
- [ ] `crates/ui-gtk/src/widgets/item_row.rs` has in-row pin/star/delete action buttons and callbacks (this is implied by T007 in the spec; if not addressed in PR 6, mark as PR 7.5 in `09-decisions.md`).

### Windowing

- [ ] `crates/ui-gtk/src/window/manager.rs` uses `AdwNavigationView` + sidebar (or `gtk::Box` + `gtk::ListBox` fallback per D15) with 6 pages.
- [ ] Window size + last page persist via GSettings (debounced 500 ms).
- [ ] Esc chain is the same as the popup (search has focus → clear; modal open → close; list has focus → close).
- [ ] `crates/ui-gtk/src/window/popup.rs` uses the global key controller; the inline Esc and `/` controllers are deleted.
- [ ] Status bar + Toast overlay wired in the manager.

### CLI / IPC

- [ ] `author-clipboard-hypr-picker --filter <name>` works for all 7 `PickerFilter` variants.
- [ ] `author-clipboard-ctl picker --filter <name>` works for all 7 `PickerFilter` variants (already shipped in PR 2).
- [ ] `IpcCommand::Copy { id, mode, mime }` round-trips through all 5 in-scope call sites.
- [ ] Daemon's restore path is `mode`-driven (D11), the UI always sends `CopyMode::Copy`.

### Docs

- [ ] `docs/UI.md` has PreviewPane, State machine, and GSettings sections.
- [ ] `docs/UI/snapshots/` has 5 PNGs (popup, manager, clipboard-page, settings, sensitive-reveal).
- [ ] `README.md` has ≥2 inline screenshots.
- [ ] `just ui-check` and `just ui-smoke` recipes exist in the justfile; **not** wired into `verify`.
- [ ] `crates/ui-gtk/tests/smoke.sh` has 5 scenarios (`/` + type, Esc-then-Esc, Pinned persistence, manager last page, sensitive reveal).

### Decisions

- [ ] `09-decisions.md` has D11, D12, D13, D14, D15 (and D-notes for any deviation).
- [ ] D15 records the sidebar primitive actually used in the manager.

### Manual

- [ ] Maintainer runs `just ui-smoke` and visually signs off the 5 screenshots.
- [ ] Maintainer runs `just applet -- --popup --filter pinned --count 10 --query "git"` and confirms 10 items, Pinned filter, "git" pre-filled.
- [ ] Maintainer opens the manager, resizes, closes, reopens at the same size; toggles Pinned in popup, opens manager, filter still Pinned.

---

## 8. Estimated cost

| PR | Subagent turns (best guess) | Wall-clock impact | Lines of code (delta) |
|---|---|---|---|
| PR 2 | 2–3 turns (1 explore + 1 implement + 1 verify) | ~15 min | +120 / −60 |
| PR 3A | 3–4 turns (1 explore app.rs, 1 implement, 1 write tests, 1 verify) | ~30 min | +400 / −20 |
| PR 3B | 4–5 turns (1 explore, 1 extend AppState+Action+Effect, 1 reduce impl, 1 tests, 1 verify) | ~45 min | +500 / −20 |
| PR 4 | 4–5 turns (1 explore controller/, 1 settings.rs rewrite, 1 key.rs + focus.rs, 1 search.rs + actions.rs, 1 verify) | ~50 min | +450 / −90 |
| PR 5 | 3–4 turns (1 explore app.rs + types, 1 implement preview.rs, 1 tests + verify) | ~40 min | +450 / −0 |
| PR 5.5 | 1–2 turns (1 Cargo.toml + preview.rs + verify) | ~10 min | +80 / −10 |
| PR 6 | 5–6 turns (1 inspect libadwaita version, 1 manager rewrite, 1 popup cleanup, 1 wire PreviewPane + persist, 1 verify, 1 spec ticks) | ~60 min | +400 / −110 |
| PR 7 | 3–4 turns (1 hypr-picker, 1 smoke + justfile, 1 docs + snapshots, 1 verify) | ~25 min | +250 / −20 |
| **Total** | **~25–33 turns** | **~4.5 hours wall-clock** | **~2,650 / −330 net** |

**Cost notes:**

- PR 3B is the largest by test count (~30 new tests). Budget extra
  time for the reviewer focus map items (RevealTick, WindowResized,
  IncognitoToggled, SetDaemonRunning).
- PR 6 is the largest by single-file rewrite (manager.rs 132 → 400
  LOC). The popup.rs cleanup is a follow-on edit in the same PR.
- PR 5.5 is the cheapest because it's a feature flag + a single
  function.
- The `just verify` wall-clock on this box is ~3 min (138 tests + 7
  crates to lint). Plan for ~3 min × 8 PRs = ~25 min of pure CI time
  across the build phase.

**Worst-case escalation**: if a subagent gets stuck on the manager
rewrite (PR 6) for >10 turns, the maintainer should:

1. Re-read the latent `/`-swallow bug fix in `window/popup.rs` —
   the inline `EventControllerKey` for `/` must be deleted; the
   new global controller handles `/` via `map_key_extended` →
   `Action::Focus(FocusTarget::Search)`.
2. Re-read D15 in `09-decisions.md` — the sidebar primitive
   choice is recorded; the agent should not re-investigate.
3. Split PR 6 into PR 6a (manager rewrite) and PR 6b (popup
   cleanup). CP rule 1 ("each PR must compile independently")
   still holds; PR 6a is a superset of the current PR 6 minus the
   popup.rs edit.

---

**Last updated**: 2026-06-15
