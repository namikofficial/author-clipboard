# Requirements: Dotfiles Production Integration

> Requirements for integrating author-clipboard into the production dotfiles environment.

---

## User Stories

### US-001: Replace Cliphist Internals
**As a** production user (namik)
**I want to** have author-clipboard handle clipboard while keeping existing keybinds
**So that** I can switch without relearning my workflow

**Acceptance Criteria**:
- Given Super+Ctrl+V is pressed, when author-clipboard daemon is running, then `author-clipboard-ctl picker --menu rofi --source history` executes
- Given Super+Shift+V is pressed, when author-clipboard daemon is running, then `author-clipboard-hypr-picker` executes
- Given the existing scripts call `cliphist`, when I update them to call author-clipboard, then behavior is identical

### US-002: Daemon Health via dev-health
**As a** production user
**I want to** see clipboard daemon status in dev-health checks
**So that** I can verify everything is working

**Acceptance Criteria**:
- Given I run `dev-health`, when the clipboard daemon check runs, then it probes the IPC socket and reports running/not running
- Given the daemon is down, when dev-health runs, then it shows a warning with instructions to restart

### US-003: Settings Hub Integration
**As a** production user
**I want to** access clipboard configuration from the Settings Hub
**So that** I don't have to edit config files manually

**Acceptance Criteria**:
- Given I open the Settings Hub, when I navigate to Clipboard, then I see retention settings, sensitive policy, and picker mode
- Given I change a setting, when I save, then the config file is updated and the daemon is notified

### US-004: AI Helper Integration
**As a** production user
**I want to** use clipboard context in AI helper workflows
**So that** I can incorporate clipboard history into code generation

**Acceptance Criteria**:
- Given I am in an ai-helper session, when I run `clip summarize`, then it summarizes recent clipboard items
- Given I am in an ai-helper session, when I run `clip to-codex`, then the selected item is sent to Codex
- Given MCP is configured, when Codex runs, then it has access to clipboard tools

### US-005: Incognito Mode Quick Toggle
**As a** production user
**I want to** quickly toggle incognito mode via keyboard shortcut
**So that** I can pause capture when needed

**Acceptance Criteria**:
- Given Super+Shift+I is pressed, when the daemon is running, then incognito mode toggles
- Given incognito is active, when I press Super+Shift+I, then capture resumes
- Given incognito is active, when I open the picker, then I see an indicator that capture is paused

---

## Existing Dotfiles Structure

### Keybinds (hypr/conf/60-binds-media.lua)

```lua
-- Current cliphist bindings (to be replaced)
exec(mainMod .. " + CTRL + V", "bash /home/namik/.dotfiles/scripts/cliphist-rofi.sh")
exec(mainMod .. " + SHIFT + V", "bash /home/namik/.dotfiles/scripts/cliphist-toggle.sh")
```

### Scripts to Update

```
scripts/
├── cliphist-rofi.sh      # -> author-clipboard-ctl picker --menu rofi
├── cliphist-toggle.sh    # -> author-clipboard-hypr-picker
├── cliphist-daemon.sh    # -> systemctl commands
└── cliphist-ipc.py       # -> (obsolete, remove)
```

### Package Dependencies (package_install)

```
# Already present
wl-clipboard
rofi-wayland

# New (if not already present)
author-clipboard (after build)
```

---

## Integration Points

### 1. Hyprland Keybinds

**Stage 1** (keep existing binds, update scripts):
```lua
exec(mainMod .. " + CTRL + V", "bash /home/namik/.dotfiles/scripts/cliphist-rofi.sh")
```

**cliphist-rofi.sh** (updated):
```bash
#!/bin/bash
# Replace cliphist call with author-clipboard
author-clipboard-ctl picker --menu rofi --source history
```

**Stage 2** (first-party binds):
```lua
# After confirming everything works, update to:
exec(mainMod .. " + CTRL + V", "author-clipboard-ctl picker --menu rofi --source history")
exec(mainMod .. " + SHIFT + V", "author-clipboard-hypr-picker")
```

### 2. dev-health Integration

```bash
#!/bin/bash
# In dev-health: clipboard section
echo "Clipboard Daemon:"
if author-clipboard-ctl ping 2>/dev/null; then
    echo "  ● Running"
    author-clipboard-ctl status | head -4
else
    echo "  ○ Not running"
    echo "  Run: systemctl --user start author-clipboard-daemon"
fi
```

### 3. Settings Hub Entry

```json
// In settings hub: clipboard section
{
  "id": "clipboard",
  "name": "Clipboard",
  "description": "Clipboard history and quick paste",
  "commands": [
    {
      "label": "Max Items",
      "type": "number",
      "get": "author-clipboard-ctl config | grep max_items",
      "set": "author-clipboard-ctl config set max_items $1"
    },
    {
      "label": "Clear on Lock",
      "type": "toggle",
      "get": "author-clipboard-ctl config | grep clear_on_lock",
      "set": "author-clipboard-ctl config set clear_on_lock $1"
    }
  ]
}
```

### 4. AI Helper Enhancement

```bash
#!/bin/bash
# In ai-helper.sh: new clip commands

clip_summarize() {
    # Summarize recent clipboard items
    author-clipboard-ctl history --limit 10 --json | jq -r '.[] | .preview' | head -5
}

clip_to_codex() {
    # Send selected item to Codex
    local item_id=$(author-clipboard-ctl history --limit 5 --json | jq -r '.[0].id')
    local content=$(author-clipboard-ctl copy $item_id --json | jq -r '.content')
    # Pass to Codex exec
}

clip_search() {
    # Search clipboard history
    author-clipboard-ctl search "$1" --json
}
```

### 5. Systemd Service

```ini
# ~/.config/systemd/user/author-clipboard-daemon.service
[Unit]
Description=Author Clipboard Daemon
After=wayland.socket
Requires=wayland.socket

[Service]
ExecStart=/home/namik/.local/bin/author-clipboard-daemon
Restart=on-failure
RestartSec=5
Environment=COSMIC_DATA_CONTROL_ENABLED=1

[Install]
WantedBy=default.target
```

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | Replace cliphist-rofi.sh script | Must | Keep keybind, change internals |
| FR-002 | Replace cliphist-toggle.sh script | Must | Keep keybind, change internals |
| FR-003 | Update cliphist-daemon.sh | Must | systemctl commands |
| FR-004 | Add dev-health clipboard probe | Should | |
| FR-005 | Settings Hub clipboard entry | Should | |
| FR-006 | AI helper clip commands | Should | |
| FR-007 | Incognito keyboard shortcut | Should | Super+Shift+I |
| FR-008 | Systemd service file | Must | |

---

## Out of Scope

- Waybar module (separate feature)
- Non-Hyprland compositor support
- Multi-machine sync

---

## Dependencies

- Feature `012-service-api` (CLI routing through daemon)
- Feature `016-world-class-ux` (UI quality)
- Systemd service file from Feature `010-packaging-systemd`

---

**Last Updated**: Phase 15