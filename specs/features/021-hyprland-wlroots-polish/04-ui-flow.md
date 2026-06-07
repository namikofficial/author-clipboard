# UI Flow: Hyprland-native UX & wlroots Polish

> User interaction flows for the Waybar module and the Hyprland demo
> content.

---

## Waybar module flow

```
[Waybar starts]
        |
        v
[waybar reads `interval: 30` for custom/clipboard]
        |
        v
[exec: contrib/waybar/clipboard.sh update]
        |
        v
[clipboard.sh calls author-clipboard-ctl status --json]
        |
        v
[ctl reads db.stats() + db.get_recent(1), tries IPC ping]
        |
        v
[clipboard.sh maps JSON -> Waybar fields (text, tooltip, class, alt)]
        |
        v
[Waybar renders the module with class-based CSS]
        |
        v
[User clicks the module]
        |
        v
[on-click: author-clipboard-hypr-picker launches as a layer-shell overlay]
        |
        v
[User selects an item -> picker closes -> clipboard updated]
        |
        v
[User can also right-click: on-click-right runs author-clipboard-ctl toggle
 for the COSMIC applet fallback]
```

---

## Signal-based refresh (optional)

```
[Daemon writes a new clipboard item]
        |
        v
[User-configured hook / a ctl subcommand sends pkill -SIGUSR1 waybar]
        |
        v
[Waybar re-runs the exec chain]
        |
        v
[Module updates immediately]
```

This path is **not** wired by default (see `09-decisions.md → D-001`).
The 30 s polling interval is the default; the `pkill` refresh is
documented as an opt-in.

---

## Hyprland demo flow

```
[Fresh Hyprland session]
        |
        v
[Install: yay -S author-clipboard]
        |
        v
[Enable: systemctl --user enable --now author-clipboard-daemon]
        |
        v
[Copy some text in any app]
        |
        v
[Press Super+Shift+V -> native picker appears as a layer overlay]
        |
        v
[Type to filter, press Enter to copy, picker closes]
        |
        v
[Press Ctrl+V in any app -> pasted content matches what was selected]
```

The `docs/HYPRLAND.md` Demo section captures this flow as a
reproducible shell transcript so a user can validate it without
watching a screencast.

---

## Waybar on-click behavior

| Gesture | Action |
|---------|--------|
| Left click | Open native picker (`author-clipboard-hypr-picker`) |
| Right click | Toggle COSMIC applet (`author-clipboard-ctl toggle`) |
| Middle click | (no binding by default) |

The right-click action opens the COSMIC applet if it's the user's
preferred UI; on a stock Hyprland install the applet isn't present, so
the toggle is a soft no-op. We don't fail the click — Waybar expects
the click to "work" even if the underlying command is absent.

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
