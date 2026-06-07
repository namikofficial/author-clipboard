# Requirements: World-Class UX

> Requirements for a premium clipboard experience.

---

## User Stories

### US-001: Split Preview Pane
**As a** user
**I want to** see a split view with list on left and preview on right
**So that** I can see item details without clicking

**Acceptance Criteria**:
- Given an item is selected, when I look at the right pane, then I see syntax-highlighted content
- Given an image is selected, when I look at the right pane, then I see the image rendered with dimensions
- Given an HTML item is selected, when I look at the right pane, then I see a sandboxed HTML preview
- Given no item is selected, when I look at the right pane, then I see usage hints

### US-002: Virtualized List
**As a** user
**I want to** scroll through 1000+ items without lag
**So that** my history remains usable under load

**Acceptance Criteria**:
- Given I have 1000 items, when I scroll the list, then it remains at 60fps
- Given I have 1000 items, when I press PageDown, then the list scrolls smoothly
- Given I have 1000 items, when I press End, then the list jumps to the last item

### US-003: Visual Sensitivity Treatment
**As a** user
**I want to** immediately see which items are sensitive
**So that** I can make informed decisions about sharing

**Acceptance Criteria**:
- Given a sensitive item, when it appears in the list, then it has a red left border (ribbon)
- Given a sensitive item, when it appears in the list, then it has a lock icon
- Given a sensitive item, when I hover over it, then I see a tooltip explaining what was detected
- Given a sensitive item, when I preview it, then the content is masked with "••••••••"

### US-004: Context Chips
**As a** user
**I want to** see context chips on items showing source app, age, and type
**So that** I can quickly identify items without reading content

**Acceptance Criteria**:
- Given an item from kitty, when it appears in the list, then I see a "kitty" chip
- Given an item copied today, when it appears in the list, then I see a "today" chip
- Given an image, when it appears in the list, then I see an "image" chip with dimensions
- Given a pinned item, when it appears in the list, then I see a "📌" indicator

### US-005: Excellent Empty States
**As a** user
**I want to** see helpful messages when lists are empty
**So that** I know what to do next

**Acceptance Criteria**:
- Given no clipboard history, when I open the picker, then I see "Copy something to get started"
- Given no pinned items, when I go to the Pinned section, then I see "Pin items with Ctrl+P"
- Given no search results, when my search has no matches, then I see "Try different keywords or remove filters"
- Given no snippets, when I go to the Snippets tab, then I see "Create a snippet with Ctrl+N"

### US-006: Keyboard Discoverability
**As a** user
**I want to** discover keyboard shortcuts through the UI
**So that** I don't have to memorize everything

**Acceptance Criteria**:
- Given the picker is open, when I press "?", then I see a keyboard shortcuts overlay
- Given an item is selected, when I hover over the action buttons, then I see the keyboard shortcut
- Given the settings tab is open, then I see a "Keyboard Shortcuts" section listing all shortcuts

### US-007: Smooth Animations
**As a** user
**I want to** see smooth animations for all UI transitions
**So that** the app feels polished and responsive

**Acceptance Criteria**:
- Given I select an item, when the preview pane updates, then the transition is smooth (200ms)
- Given I pin an item, when the icon changes, then there is a subtle animation
- Given I switch tabs, when the content changes, then the transition is smooth (150ms)
- Given the picker opens, when it appears, then it fades in smoothly (100ms)

---

## UI Layout

### Main Picker Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ [Search Box]                                    [Filters] [?]   │
├─────────────────────────────────────────────────────────────────┤
│ Tabs: [Clipboard] [Emoji] [Symbols] [Kaomoji] [Snippets] [⚙️] │
├────────────────────────────┬────────────────────────────────────┤
│                            │                                    │
│  List (virtualized)        │  Preview Pane                      │
│  ┌──────────────────────┐ │  ┌────────────────────────────┐   │
│  │ 📌 [content preview] │ │  │                            │   │
│  │    [chips] [actions]  │ │  │ Syntax-highlighted content │   │
│  ├──────────────────────┤ │  │                            │   │
│  │ ⭐ [content preview] │ │  │ Image: rendered preview    │   │
│  │    [chips] [actions]  │ │  │ HTML: sandboxed preview    │   │
│  ├──────────────────────┤ │  │ Files: file cards           │   │
│  │ [content preview]    │ │  │                            │   │
│  │    [chips] [actions]  │ │  │                            │   │
│  └──────────────────────┘ │  └────────────────────────────┘   │
│                            │                                    │
├────────────────────────────┴────────────────────────────────────┤
│ Status: 150 items | 12 pinned | ● Daemon running | 🔒 Incog   │
└─────────────────────────────────────────────────────────────────┘
```

### Preview Pane States

| Content Type | Preview Content |
|--------------|-----------------|
| Text | Syntax-highlighted (auto-detect language) with line numbers |
| HTML | Sandboxed iframe with rendered HTML |
| Image | Rendered image with dimensions and file size |
| Files | File cards with icons, names, and paths |
| Sensitive | Masked content with lock icon and warning |

---

## Visual Design

### Color Palette (from system theme)

| Element | Light Mode | Dark Mode |
|---------|------------|-----------|
| Background | system background | system background |
| Surface | #FFFFFF | #1E1E1E |
| Primary | system accent | system accent |
| Text | system text | system text |
| Secondary Text | #666666 | #999999 |
| Border | #E0E0E0 | #333333 |
| Sensitive Ribbon | #DC3545 | #FF6B6B |
| Pin Indicator | #28A745 | #4ADE80 |
| Star Indicator | #FFC107 | #FFD54F |
| Error | #DC3545 | #FF6B6B |

### Typography

| Element | Font | Size | Weight |
|---------|------|------|--------|
| Item Content | Monospace | 13px | 400 |
| Chips | System | 11px | 500 |
| Section Headers | System | 12px | 600 |
| Preview Code | Monospace | 12px | 400 |
| Status Bar | System | 11px | 400 |

### Spacing

- Item padding: 8px 12px
- Chip margin: 4px
- Section gap: 16px
- Preview padding: 16px

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | Split preview pane | Must | Left list, right preview |
| FR-002 | Virtualized list rendering | Must | Handle 1000+ items |
| FR-003 | Syntax highlighting | Must | Auto-detect language |
| FR-004 | HTML sandbox preview | Must | Sandboxed iframe |
| FR-005 | Image preview with dimensions | Must | |
| FR-006 | Sensitive item visual treatment | Must | Red ribbon, lock icon |
| FR-007 | Context chips (app, age, type) | Must | |
| FR-008 | Empty states for all sections | Must | |
| FR-009 | Keyboard shortcuts overlay | Must | Press "?" |
| FR-010 | Smooth animations | Must | 60fps target |
| FR-011 | System theme integration | Must | Light/dark support |
| FR-012 | Action bar with keyboard hints | Must | |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | List scroll performance | 60fps | With 1000 items |
| NFR-002 | Preview pane update | < 100ms | |
| NFR-003 | Animation frame time | < 16ms | 60fps |
| NFR-004 | Memory usage | < 100MB | With 1000 items |
| NFR-005 | Startup time | < 200ms | Cold start |

---

## Edge Cases

| Case | Handling |
|------|----------|
| Very long text content | Truncate with "..." in list, full in preview |
| Very large image | Downscale for thumbnail, full in preview |
| Corrupt HTML | Show raw HTML in preview |
| Unknown content type | Show generic icon and filename |
| 1000+ items | Virtualized list, load more on scroll |

---

## Out of Scope

- Custom themes
- Animation customization
- Multiple color schemes
- Custom fonts

---

## Dependencies

- Feature `015-collections` (for chip display)
- Feature `014-advanced-filtering` (for filter chips in search)

---

**Last Updated**: Phase 15