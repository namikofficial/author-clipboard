# Requirements: Global Shortcut & COSMIC Integration

---

## User Stories

### US-001: Instant Picker Access
**As a** user
**I want to** press Super+V and see my clipboard history immediately
**So that** I can quickly find and paste old content

**Acceptance Criteria**:
- Given Super+V is pressed, when the picker is closed, then picker opens in < 100ms
- Given Super+V is pressed, when the picker is open, then picker closes

### US-002: Multi-Monitor Support
**As a** user with multiple monitors
**I want to** open the picker on the active monitor
**So that** I don't have to move my cursor

**Acceptance Criteria**:
- Given I press Super+V on monitor 2, then picker appears on monitor 2
- Given cursor is on monitor 1, when I press Super+V, then picker appears on monitor 1

### US-003: Focus Restoration
**As a** user
**I want to** press Escape to close the picker and return focus
**So that** I can continue my previous task

**Acceptance Criteria**:
- Given picker is open, when I press Escape, then picker closes and previous window has focus

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | IPC-based toggle | Must | `author-clipboard-ctl toggle` |
| FR-002 | Layer-shell positioning | Must | ShowAt with cursor coords |
| FR-003 | Focus handling | Must | Visibility toggle via IPC |
| FR-004 | Autostart systemd service | Must | User session service |
| FR-005 | .desktop file and icon | Must | App launcher |
| FR-006 | Shortcut conflict detection | Should | Log warning |

---

## Out of Scope

- Shortcut configuration UI
- Waybar integration

---

**Last Updated**: Phase 2 Complete