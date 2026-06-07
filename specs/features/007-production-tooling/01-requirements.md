# Requirements: Production Tooling

---

## User Stories

### US-001: CLI Control
**As a** user
**I want to** control the daemon via CLI
**So that** I can script and automate clipboard operations

**Acceptance Criteria**:
- Given `author-clipboard-ctl ping` is run, when daemon is running, then "pong" is returned
- Given `author-clipboard-ctl toggle` is run, then picker toggles visibility
- Given `author-clipboard-ctl history --limit 10` is run, then JSON list of items is returned

### US-002: Config File
**As a** user
**I want to** configure the daemon via JSON file
**So that** settings persist across restarts

**Acceptance Criteria**:
- Given `~/.config/author-clipboard/config.json` exists, when daemon starts, then settings are loaded
- Given `author-clipboard-ctl config` is run, then current settings are printed

### US-003: Graceful Shutdown
**As a** user
**I want to** stop the daemon cleanly
**So that** socket is removed and no stale state remains

**Acceptance Criteria**:
- Given daemon is running, when it receives SIGTERM, then socket is cleaned up
- Given daemon is running, when `systemctl stop` is run, then no stale socket remains

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `toggle` | Show/hide picker |
| `show` | Show picker |
| `hide` | Hide picker |
| `ping` | Health check |
| `history` | List items (with `--limit`, `--offset`) |
| `status` | Database statistics |
| `clear` | Clear unpinned items |
| `export` | Export to JSON |
| `config` | Show current config |
| `doctor` | Probe display/protocol support |
| `copy <id>` | Copy item by ID |
| `picker` | Open external picker |
| `hyprland-config` | Print Hyprland keybinds |

---

**Last Updated**: Phase 8 Complete