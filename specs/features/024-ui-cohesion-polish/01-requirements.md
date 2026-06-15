# Requirements: UI Cohesion & Dynamic Polish

## User Stories

### US-001: Cohesive Shell
**As a** user
**I want** popup and manager surfaces to share the same visual language
**So that** the app feels like one product, not separate screens

**Acceptance Criteria**
- Popup and manager use the same spacing scale, radii, shadows, and icon sizing
- Primary actions, chips, rows, and status elements feel visually related
- Theme changes update both surfaces consistently

### US-002: Clear Visual Hierarchy
**As a** user
**I want** the important content to stand out immediately
**So that** I can scan the UI quickly

**Acceptance Criteria**
- Search, filter, list, preview, and status sections have distinct hierarchy
- Selected rows are obvious without looking flashy
- Empty states read as intentional, not as missing content

### US-003: Responsive Manager
**As a** user
**I want** the manager to adapt cleanly to narrow and wide windows
**So that** it works well on laptop and desktop layouts

**Acceptance Criteria**
- The manager keeps a polished two-column layout on wide windows
- Narrow widths collapse gracefully without visual breakage
- Sidebar, preview, and list spacing remain balanced at each breakpoint

### US-004: Dynamic Feedback
**As a** user
**I want** hover, focus, selection, and toast feedback to feel responsive
**So that** the interface feels alive and predictable

**Acceptance Criteria**
- Hover and selection transitions are consistent across widgets
- Focus rings are visible and theme-aware
- Toasts, reveal states, and empty states have clear visual states

### US-005: Accessibility Preserved
**As a** user
**I want** polished visuals without losing keyboard usability
**So that** the app remains usable without a mouse

**Acceptance Criteria**
- Contrast remains readable in light and dark themes
- Focus order and keyboard shortcuts remain unchanged
- No visual treatment hides labels or state from keyboard users

### US-006: Documented and Verifiable
**As a** maintainer
**I want** the UI pass to be documented with screenshots and checks
**So that** future changes can be reviewed against a known-good baseline

**Acceptance Criteria**
- Updated screenshots exist for popup and manager
- The UI change list is recorded in docs
- Verification steps are explicit and repeatable

