# UI Flow: Native Power-User Revamp

> Interaction model for the native picker, manager, inspector, actions, and
> power-user workflows.

---

## Primary Surfaces

### Native Picker

Purpose: fast keyboard-first access from a global shortcut.

Default Hyprland behavior:

```text
Centered floating XDG utility window
Resizable
Esc and close button terminate the picker process
Optional --layer-shell overlay mode
```

Target layout:

```text
+--------------------------------------------------------------------+
| Author Clipboard                         Copy mode   Health   X    |
| Search clipboard, snippets, commands, links, files...              |
| [All] [Text] [Code] [Links] [Files] [Images] [Pinned] [Sensitive]  |
+-----------------------------------+--------------------------------+
| List                              | Inspector                      |
| > git status --short              | Text / command                 |
|   command · 2m · text/plain       | git status --short             |
|   [text] [star]                   |                                |
|                                   | Actions                        |
|   ~/.config/hypr/hyprland.lua     | Copy  Quick paste  Pin  Star   |
|   path · 10m · text/plain         | Delete  Collection             |
+-----------------------------------+--------------------------------+
| 348 items · daemon running · incognito off · ? shortcuts            |
+--------------------------------------------------------------------+
```

### Manager

Purpose: deeper organization, settings, collections, snippets, import/export,
and history maintenance.

Expected layout:

```text
Sidebar: History, Collections, Snippets, Saved Filters, Settings, Health
Main: list/table
Right: inspector/details
Bottom: status and sync/import/export feedback
```

## Core Flows

### Open And Close

1. User presses `Super+Shift+V`.
2. Picker opens as centered floating window.
3. List is focused by default.
4. User presses `Esc` or clicks close.
5. Window closes and picker process exits.

### Search And Copy

1. User opens picker.
2. User presses `/` or starts typing if search has focus.
3. Results update with debounced query.
4. User navigates with arrows, PageUp/PageDown, Home/End.
5. User presses `Enter`.
6. Item is copied or quick-pasted depending on mode.
7. Picker closes by default for copy/quick-paste actions.

### Inspect Before Copy

1. User selects an item.
2. Inspector updates with content-specific preview.
3. User checks metadata: MIME, age, size, sensitive state, collections.
4. User chooses Copy, Quick paste, Pin, Star, Delete, or Add to collection.

### Sensitive Reveal

1. User selects a sensitive item.
2. Row and inspector show redacted preview and warning.
3. User presses reveal action.
4. UI shows content for a short countdown.
5. Countdown expires and content is redacted again.

### Collection Assignment

1. User selects an item.
2. User presses `Ctrl+Shift+C` or action menu.
3. Collection chooser appears.
4. User picks existing collection or creates a new one.
5. Item receives collection badge.

### Saved Filter

1. User enters query such as `type:command project:noxcrm`.
2. User opens command menu or action button.
3. User chooses "Save filter".
4. User names it `NoxCRM commands`.
5. Saved filter appears in sidebar and can be launched from CLI.

### First-Run/Health

1. User opens picker with daemon down or missing dependencies.
2. Header/status shows degraded state.
3. Health panel lists actionable checks:
   - daemon service status
   - compositor support
   - GSettings schema
   - quick-paste backend
   - wl-copy/wtype/ydotool
4. User can run copyable commands or click retry/start where safe.

## Keyboard Map

| Shortcut | Action |
|----------|--------|
| `Esc` | Close picker; in nested dialogs, close dialog first. |
| `/` | Focus search. |
| `Enter` | Copy selected item. |
| `Shift+Enter` | Quick paste selected item. |
| `Ctrl+P` | Toggle pin. |
| `Ctrl+Shift+S` | Toggle star. |
| `Delete` | Delete selected item with confirmation/undo. |
| `Ctrl+Shift+C` | Add to collection. |
| `Ctrl+1..9` | Jump to result. |
| `?` / `F1` | Shortcut overlay. |
| `Ctrl+Tab` | Next source/tab. |
| `Ctrl+Shift+Tab` | Previous source/tab. |

## Visual Direction

- Native utility, not web dashboard.
- Strong cards for rows, but not heavy.
- Clear contrast between content, metadata, actions, and warnings.
- More text visible by default; full text available in inspector.
- Compact enough for keyboard speed; rich enough for confidence.
- Cute and warm through tone, icons, spacing, and gentle surfaces; not novelty.

---

**Last Updated**: 2026-06-19
