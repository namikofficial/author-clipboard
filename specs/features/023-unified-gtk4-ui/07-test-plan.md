# Test Plan: Unified GTK4 UI

---

## Unit (in-crate, no GTK main loop required)

| Module | Test | Method |
|---|---|---|
| `app::reduce` | All `Action` variants | ✅ 42 tests — all 29 Action variants covered |
| `controller::focus` | Esc with focus on search, list, button | mock `Focusable` trait |
| `controller::focus` | Esc twice closes | state machine trace |
| `controller::search` | Debounce timing, only latest query applied | mock clock |
| `widgets::item_row` | All content types render without panic | `gtk4_test::RenderTest` |
| `widgets::item_row` | Sensitive row shows `redacted_preview`, not `content` | inspect children |
| `widgets::filter_bar` | Each chip emits correct `PickerFilter` | record actions |
| `widgets::search` | `/` focuses; Esc clears | record actions |
| `pages::clipboard` | `Ctrl+1..9` sets `selected_index` | send key, assert |
| `pages::settings` | Each setting writes to `Config` | mutate, reload |
| `shared::picker::filter_entries` | All `PickerFilter` values | sample data |

## Unit (in-crate, requires GTK init)

| Module | Test | Method |
|---|---|---|
| `widgets::preview` | Sensitive reveal countdown | manual time travel |
| `window::popup` | Layer-shell init succeeds (under `xvfb`) | smoke |
| `window::manager` | Navigation push/pop | smoke |

## Integration

| Scenario | Command | Pass criteria |
|---|---|---|
| Popup opens, list focused | `xvfb-run author-clipboard --popup` | first row has CSS class `selected` |
| Search via `/` | `xdotool key slash; type "git"` | list filters live, debounced |
| Esc clears search then closes | `xdotool type "x"; key Escape; key Escape` | search empty, then window closed |
| Copy on Enter | `xdotool key Return` | toast "Copied", IPC `Copy` sent, exit 0 |
| Sensitive reveal | `xdotool key ctrl+shift+r` | toast with countdown, content visible 5s |
| Manager opens | `xvfb-run author-clipboard --manager` | window 1100×720, sidebar present |
| Settings persist | change Max items to 500, close, reopen | value is 500 |
| Filter survives popup→manager | set filter to Pinned, switch modes | filter still Pinned |
| External picker `--filter pinned` | `author-clipboard-ctl picker --filter pinned` | only pinned rows shown |
| Theme adapts to dark | `gsettings set org.gnome.desktop.interface color-scheme prefer-dark` | UI re-themes within 1 frame |

## Visual

Screenshots saved by T019 to `docs/UI/snapshots/`. Manual review on
PR. CI does not diff (CI can't render); local maintainer run with
`just ui-smoke` updates the snapshots and a maintainer eyeballs the
diff.

## Lint / Format / Build

```bash
just verify   # runs fmt, clippy -D warnings, test, build
```

## Coverage Targets

- All public functions in `ui-gtk` have a test (unit or integration).
- All `Action` variants have at least one `reduce` test.
- All `PickerFilter` values have a `filter_entries` test.
- All keyboard shortcuts from US-005 have at least one integration
  test (covered by the smoke shell test).

## Edge Case Tests

| Case | Test |
|---|---|
| Empty `items` list | `view()` returns empty state, not blank |
| Sensitive item in list | row never displays `content` even if bound |
| Redacted reveal timeout | content auto-hides at 5s |
| 1000+ items in list | list scrolls without rebuilding the world |
| Daemon down | UI shows "Offline" chip, IPC errors are toasts |
| `Ctrl+1` with no items | no-op, no crash |
| `Ctrl+1` past visible range | no-op, no crash |
| Filter `Sensitive` on empty | empty state shown |
| Long URL in text preview | ellipsized, not overflowing the row |
| Image item with missing thumbnail | `???` chip, not crash |
| HTML item with malformed HTML | raw text shown, WebView fails gracefully |
| Unicode in search query | exact-match, FTS5 still works |
| Multi-byte clipboard content (emoji in text) | renders correctly |
| GSettings unavailable | falls back to in-memory state, no crash |

## Performance Tests

| Metric | Target | Method |
|---|---|---|
| Popup cold start | < 150ms | `time` shell wrapper |
| Manager cold start | < 300ms | `time` shell wrapper |
| List scroll FPS | 60fps with 1000 items | `gtk4_test::ScrollBench` |
| Memory (manager, 1000 items) | < 80MB | `/usr/bin/time -v` |
| Search debounce | fires 150ms ± 30ms after last keystroke | unit test with mock clock |
| IPC round-trip | < 50ms local | tokio timing assert in unit test |

---

**Last Updated**: 2026-06-12
