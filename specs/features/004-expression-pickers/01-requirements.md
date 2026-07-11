# Requirements: Expression Pickers

---

## User Stories

### US-001: Emoji Picker
**As a** user
**I want to** browse and search emoji
**So that** I can insert them into my text

**Acceptance Criteria**:
- Given I open the Emoji tab, then I see categories (Smileys, Objects, etc.)
- Given I type "heart", then I see heart-related emoji
- Given I click an emoji, then it is copied to clipboard

### US-002: Symbol Picker
**As a** user
**I want to** find math symbols, arrows, currency, etc.
**So that** I can insert special characters

**Acceptance Criteria**:
- Given I open the Symbols tab, then I see categories
- Given I click a symbol, then it is copied to clipboard

### US-003: Kaomoji Picker
**As a** user
**I want to** browse and search kaomoji
**So that** I can add emotive text faces

**Acceptance Criteria**:
- Given I open the Kaomoji tab, then I see categories
- Given I type "happy", then I see happy kaomoji
- Given I click a kaomoji, then it is copied to clipboard

### US-004: Recently Used
**As a** user
**I want to** quickly access recently used items from any picker
**So that** I don't have to search again

**Acceptance Criteria**:
- Given I use emoji, when I switch tabs and back, then used emoji appear in Recently Used
- Given I restart the app, when I open a picker, then Recently Used still shows past items

### US-005: Fast keyboard-oriented discovery
**As a** user
**I want to** search or narrow expressions by category
**So that** large expression libraries remain quick to use

**Acceptance Criteria**:
- Given a picker is open, when I type a category or item description, then only matching expressions remain visible
- Given I select a category chip, then only expressions in that category remain visible
- Given no expression matches, then the picker shows a clear empty result message
- Given I activate an expression, then it is copied as plain text and its usage is persisted

---

## Out of Scope

- Tenor API GIF search (requires API key, deferred)
- GIF picker (deferred)

---

**Last Updated**: 2026-07-12
