# Requirements: Advanced Filtering & Saved Searches

> Requirements for composable filter chips and saved searches.

---

## User Stories

### US-001: Text Search with Filters
**As a** user
**I want to** type text and add filter chips to narrow results
**So that** I can find specific items quickly

**Acceptance Criteria**:
- Given I type "password" and add `type:text` chip, when I press Enter, then I see only text items containing "password"
- Given I type "AWS" and add `age:today` chip, when I press Enter, then I see only today's items containing "AWS"
- Given I clear the search, then all filters are cleared and I see recent items

### US-002: Composable Filter Chips
**As a** user
**I want to** combine multiple filter chips
**So that** I can narrow down to exactly what I need

**Acceptance Criteria**:
- Given I add `type:text`, `app:kitty`, and `pinned:false` chips, when I press Enter, then I see text items from kitty that are not pinned
- Given I add `type:image` and `age:week` chips, when I press Enter, then I see images from the last week
- Given I have 3 chips active, when I click one to remove it, then the other chips remain

### US-003: Saved Searches
**As a** user
**I want to** save my common searches with names
**So that** I can run them with one click

**Acceptance Criteria**:
- Given I have a search "type:text sensitive:true", when I click Save, then I am prompted for a name like "Sensitive text"
- Given I have saved searches, when I open the search box, then I see a dropdown of saved searches
- Given I select "API keys copied today" from saved searches, then the search is executed with those parameters

### US-004: Filter Autocomplete
**As a** user
**I want to** see autocomplete suggestions for filter values
**So that** I don't have to remember exact app names or dates

**Acceptance Criteria**:
- Given I type `app:`, when I pause, then I see a dropdown of recently seen source apps
- Given I type `age:`, when I pause, then I see options like "today", "week", "month"
- Given I type `type:`, when I pause, then I see options like "text", "image", "html", "files"

### US-005: Search Suggestions
**As a** user
**I want to** see search suggestions based on my history
**So that** I can quickly repeat previous searches

**Acceptance Criteria**:
- Given I have searched for "rust" before, when I type "r", then "rust" appears as a suggestion
- Given I have used `app:kitty` before, when I type "app:", then "app:kitty" appears as a suggestion
- Given I press Enter without selecting a suggestion, then the literal text is searched

---

## Search Grammar

### Filter Chips

| Filter | Syntax | Values | Example |
|--------|--------|--------|---------|
| Content type | `type:` | text, image, html, files | `type:text` |
| Age | `age:` | today, week, month, <number>s/m/h/d | `age:today`, `age:2h` |
| Source app | `app:` | any string | `app:kitty`, `app:firefox` |
| Pinned | `pinned:` | true, false | `pinned:true` |
| Sensitive | `sensitive:` | true, false | `sensitive:true` |
| Starred | `starred:` | true, false | `starred:true` |
| Size | `size:` | small (<1KB), medium (1KB-1MB), large (>1MB) | `size:large` |
| Collection | `in:` | collection name | `in:prompts`, `in:deploy` |

### Examples

- `type:text age:today` - Text items from today
- `app:kitty sensitive:false` - Non-sensitive items from kitty
- `AWS size:medium` - Medium-sized items containing "AWS"
- `type:html in:prompts` - HTML items in the "prompts" collection

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | Unified search box with text + chips | Must | |
| FR-002 | Chip-based filter UI | Must | Click to add, click to remove |
| FR-003 | Real-time filter preview | Must | Show active filters above results |
| FR-004 | Saved searches with names | Must | Persist in config |
| FR-005 | Filter autocomplete | Must | For type, age, app, etc. |
| FR-006 | Search history suggestions | Should | Based on recent searches |
| FR-007 | Keyboard shortcut to open saved searches | Should | |
| FR-008 | Import/export saved searches | Should | |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | Filter parsing latency | < 10ms | |
| NFR-002 | Autocomplete popup display | < 50ms | |
| NFR-003 | Search results update | < 100ms | As user types |

---

## Edge Cases

| Case | Handling |
|------|----------|
| Invalid filter syntax | Highlight error, show suggestion |
| Unknown app name | Show empty results, not error |
| Saved search with deleted collection | Show warning, allow deletion or re-selection |
| Conflicting filters (pinned:true pinned:false) | Use most recent |

---

## Out of Scope

- Natural language search ("show me today's passwords")
- Full regex in search text
- Collaborative/shared searches

---

## Dependencies

- Feature `012-service-api` (required - IPC query commands)
- Feature `015-collections` (in collection filter)
- Feature `016-world-class-ux` (UI implementation)

---

**Last Updated**: Phase 15