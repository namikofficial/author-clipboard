# Requirements: Quick Paste

---

## User Stories

### US-001: Quick Paste Text
**As a** user
**I want to** select an item and have it typed into my active application
**So that** I can paste in one action

**Acceptance Criteria**:
- Given `wtype` is installed and quick paste is enabled, when I select an item and press Enter, then text is typed
- Given only `wl-copy` is available, when I select an item, then clipboard is updated (copy-only fallback)

### US-002: Backend Detection
**As a** user
**I want to** see which paste backend is active
**So that** I know if quick paste will work

**Acceptance Criteria**:
- Given `wtype` is installed, when I open settings, then I see "wtype" as active
- Given `ydotool` is installed but not `wtype`, when I open settings, then I see "ydotool" as active
- Given no quick paste tool is installed, when I open settings, then I see "wl-copy (copy only)"

### US-003: Security Warning
**As a** user
**I want to** understand the permissions required for quick paste
**So that** I can make an informed decision

**Acceptance Criteria**:
- Given quick paste is off by default, when I enable it, then I see a warning about input permissions
- Warning explains that wtype/ydotool requires ability to send keyboard events

---

## Backend Priority

1. `wtype` — preferred, simpler permission model
2. `ydotool` — optional, may require daemon/permissions
3. `wl-copy` — fallback, copy-only

---

**Last Updated**: Phase 5 Complete