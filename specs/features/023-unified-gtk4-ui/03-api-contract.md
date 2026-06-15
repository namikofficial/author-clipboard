# API Contract: Unified GTK4 UI

---

## No IPC Changes

The IPC protocol is untouched. The new UI consumes the same
`IpcCommand::History`, `Pin`, `Unpin`, `Delete`, `Copy`, `ToggleStar`,
`ClearUnpinned`, `ListSnippets`, `UpsertSnippet`, `DeleteSnippet`,
`Status` commands the applet already uses.

## New CLI Flags on `author-clipboard`

```
author-clipboard [--popup|--manager] [--source history|emoji|...|all]
                 [--filter all|text|images|files|pinned|starred|sensitive]
                 [--query "string"]
                 [--action copy|quick-paste]
                 [--count N]
```

| Flag | Default | Notes |
|---|---|---|
| `--popup` | off | Layer-shell popup (720×520). |
| `--manager` | off | Normal XDG window (1100×720). |
| `--source` | `history` | `history`, `emoji`, `symbols`, `kaomoji`, `snippets`, `all` |
| `--filter` | `all` | Same enum as the chips |
| `--query` | none | Pre-fills search |
| `--action` | `copy` | `copy` or `quick-paste` |
| `--count` | `200` | Max items to load |

Default mode: if neither `--popup` nor `--manager` is set, the
binary inspects its environment — if it has a controlling TTY
(launched from terminal), it opens `--manager`; otherwise it opens
`--popup`. The `.desktop` file passes `--manager` explicitly.

### Error responses

| Code | Meaning |
|---|---|
| `INVALID_ARG` | Unknown `--source`, `--filter`, or `--action` value |
| `DAEMON_DOWN` | IPC `Status` returned `running: false`; UI shows banner, degrades gracefully |
| `PERMISSION_DENIED` | Layer-shell protocol unavailable; UI falls back to XDG window |

## `author-clipboard-hypr-picker` (preserved for backward compat)

Keeps all existing flags for Hyprland keybind compatibility:

```
author-clipboard-hypr-picker [--source history|...|all]
                             [--count 50]
                             [--include-sensitive]
                             [--action copy|quick-paste]
                             [--query "string"]
```

Internally rewritten as a one-liner:

```rust
fn main() -> anyhow::Result<()> {
    let cli = HyprPickerCli::parse();
    ui_gtk::run_popup(ui_gtk::PopupConfig::from_cli(cli))
}
```

## External Picker (`ctl picker`) — Small Change

Adds `--filter` to mirror the chips:

```
author-clipboard-ctl picker --menu auto --filter pinned --include-sensitive --query "git"
```

`picker::build_external_rows` gains a `filter: PickerFilter` parameter.
`filter_entries` is updated to apply it. The new flags appear in
`author-clipboard-ctl picker --help` output.

### External row format (unchanged shape, new prefix)

```
🔒  ghp_xxxxxxxxxxxxxxxxxxxx  ·  2m  ·  📌      [sensitive, pinned]
    git pull --tags origin dev                  [text]
📷  Screenshot 2026-06-12 at 14.32.png  ·  just now      [image, 2.4MB]
    /home/namik/Pictures/Screenshots/...        [image]
```

Sensitive rows gain a `🔒` prefix; pinned rows gain a `📌` suffix.

## GSettings Bindings (D-Bus schema)

`com.namikofficial.author-clipboard.state`:

| Key | Type | Default | Bound to |
|---|---|---|---|
| `filter` | enum | `all` | `AppState.filter` |
| `sort` | enum | `newest` | `AppState.sort` |
| `last-page` | enum | `clipboard` | `AppState.active_page` (manager only) |
| `window-width` | int | `1100` | manager window |
| `window-height` | int | `720` | manager window |
| `popup-width` | int | `720` | popup window |
| `popup-height` | int | `520` | popup window |

Reads via `settings.get::<PickerFilter>("filter")`; writes via
`settings.set("filter", &value)`. Bindings auto-update on schema
change so a CLI command (`gsettings set …`) reflects immediately.

---

**Last Updated**: 2026-06-12
