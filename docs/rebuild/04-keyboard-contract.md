# Author Clipboard — Unified Keyboard Controller

**Date:** 2026-07-27
**Status:** ✅ Complete

---

## Problem

The popup used capture‑phase `EventControllerKey` to intercept Esc before GTK
widgets could handle it. This worked for Esc but caused capture‑phase conflicts
with GTK `ListBox` navigation (Up/Down/Home/End/PageUp/PageDown) and
`SearchEntry` text input (character keys). Key handling was split across
window, page, and widget layers with inconsistent routing.

## Approach

Move **all** keyboard handling to **bubble‑phase** `EventControllerKey`
installed from the window‑level controller. GTK widgets own their natural
handlers (ListBox navigation, SearchEntry text); the controller handles
everything else by matching unconsumed key events.

## Ownership Table

| Keys | Owner | Phase | Mechanism |
|------|-------|-------|-----------|
| Up / Down / Home / End / PageUp / PageDown | `ListBox` (widget) | Bubble (widget default) | Built‑in GTK navigation; `row‑selected` updates `AppState.selected_id` |
| Enter (activate) | `ListBox` (widget) | Bubble | `row‑activated` → `on_copy` callback |
| Text input (a–z, 0–9, space, backspace, etc.) | `SearchEntry` (widget) | Bubble | Normal GTK text editing |
| Esc | Window‑level controller | Bubble | Resolved through `resolve_escape()` based on `FocusTarget` |
| `/` (focus search) | Window‑level controller | Bubble | `map_window_key` → `FocusTarget::Search` |
| `?` (shortcuts) | Window‑level controller | Bubble | `map_window_key` → `Action::ShowShortcuts` |
| `F1` (modal) | Window‑level controller | Bubble | `map_window_key` → `Action::ShowShortcuts` |
| `Ctrl+Enter` / `Ctrl+KP_Enter` | Window‑level controller | Bubble | `map_window_key` → `Action::AltActivate` |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Window‑level controller | Bubble | `map_window_key` → `Action::CyclePage(±1)` |
| `Ctrl+P` | Window‑level controller | Bubble | `map_window_key` → `Action::ToggleSelectedPin` |
| `Ctrl+Shift+S` | Window‑level controller | Bubble | `map_window_key` → `Action::ToggleSelectedStar` |
| `Ctrl+1..9` | Window‑level controller | Bubble | `map_window_key` → `Action::QuickPickItem(n)` |
| `Delete` / `Ctrl+D` | Window‑level controller | Bubble | `map_window_key` → `Action::DeleteSelected` (guarded: skipped if focus is Search or TextInput) |
| `Ctrl+Shift+P` / `Ctrl+Shift+C` / `Ctrl+Shift+A` | Window‑level controller | Bubble | `map_window_key` → `Action::PageChanged(...)` |
| `Ctrl+Shift+/` | Window‑level controller | Bubble | `map_window_key` → `Action::ShowShortcuts` |

## Esc Resolution (`controller/focus.rs`)

`resolve_escape(focus_target, search_query) → Propagate`:

| `FocusTarget` | `query.is_empty()` | Behavior |
|---|---|---|
| `Search` | false | Clear query, emit `Action::QueryCleared`, return `Proceed` |
| `Search` | true | Blur search entry, set focus to `List`, return `Proceed` |
| `TextInput` | (any) | Returns `Proceed` (let the widget handle it) |
| `List` | (any) | Call close callback, return `Stop` |
| `Modal` | (any) | Returns `Proceed` (let the dialog handle it) |
| (none/unset) | (any) | Call close callback, return `Stop` |

The `FocusTarget::TextInput` variant was added so that in‑field Esc always
passes through to the widget (e.g., GTK `SearchEntry`, `Entry`, or custom
text inputs that manage their own Esc handling).

## Action Rail Reactivity (`widgets/action_bar.rs`)

The action rail previously used a 100ms `glib::timeout_add_local` poll loop
to refresh button sensitivity. This has been replaced with event‑driven
reactivity:

| Trigger | Mechanism |
|---------|-----------|
| `ListBox::row-selected` signal | `rail_refresh()` called in the popup's row‑selected handler |
| State changes through reducer | Effects dispatched through the `tx`/`rx` channel; the idle-loop handler re‑evaluates the rail when relevant effects arrive |

The `ActionRail` struct exposes a `refresh: Rc<dyn Fn()>` field so
window‑level code can wire it to any signal.

## `install()` Signature

```rust
pub fn install(
    window: &impl IsA<gtk4::Window>,
    state: &Rc<RefCell<AppState>>,
    tx: &Sender<Effect>,
    on_close: Option<Box<dyn Fn()>>,
    search: Option<&gtk4::SearchEntry>,
    list: Option<&gtk4::ListBox>,
    on_page_key: Option<Box<dyn Fn(KeyEvent) -> Option<Action>>>,
);
```

- `on_close` — called when Esc resolves to "close" (popup) or `None` (manager).
- `on_page_key` — injected by `install_page_keys()` for page‑level keys
  that need page context (spelled out in the shared controller module).

## Delete Guard Logic

`Ctrl+D` / `Delete` keys are suppressed when `FocusTarget` is `Search` or
`TextInput` to prevent accidental deletion while typing. The guard is in
`controller/key.rs` at the point where `map_window_key` results are consumed.

## Files Changed

- `crates/ui-gtk/src/lib.rs` — stale capture‑phase comment updated
- `crates/ui-gtk/src/app.rs` — `FocusTarget::TextInput`, new `Action`/`Effect` variants
- `crates/ui-gtk/src/controller/mod.rs` — shared `install_page_keys()`
- `crates/ui-gtk/src/controller/key.rs` — consolidated `map_window_key`, `install()`, delete guard
- `crates/ui-gtk/src/controller/focus.rs` — `TextInput` support in `resolve_escape()`, tests
- `crates/ui-gtk/src/widgets/action_bar.rs` — replaced 100ms polling with `ActionRail` + `Rc<dyn Fn()>` refresh
- `crates/ui-gtk/src/window/popup.rs` — removed duplicate `tx`/`rx`, updated effect handler, wired rail refresh
- `crates/ui-gtk/src/window/manager.rs` — updated `install()` call, effect handler with service dispatch
- `crates/ui-gtk/src/pages/clipboard.rs` — removed page‑level key controller (moved to `install_page_keys()`)
