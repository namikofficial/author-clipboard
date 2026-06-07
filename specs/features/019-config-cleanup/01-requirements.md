# Requirements: Configuration Cleanup

> Requirements for renaming and clarifying the content denylist configuration.

---

## User Stories

### US-001: Rename Field
**As a** user
**I want to** see `content_denylist` instead of `content_regex_denylist`
**So that** the name accurately reflects what it does

**Acceptance Criteria**:
- Given I read the config schema, when I look for content filtering, then I see `content_denylist` not `content_regex_denylist`
- Given I have old config with `content_regex_denylist`, when the daemon loads, then it migrates to `content_denylist`

### US-002: Pattern Mode
**As a** user
**I want to** specify how the pattern is matched (prefix, suffix, substring, exact)
**So that** I can configure filtering precisely

**Acceptance Criteria**:
- Given `content_pattern_mode: "prefix"`, when content starts with the pattern, then it is denied
- Given `content_pattern_mode: "suffix"`, when content ends with the pattern, then it is denied
- Given `content_pattern_mode: "substring"`, when content contains the pattern, then it is denied
- Given `content_pattern_mode: "exact"`, when content exactly matches the pattern, then it is denied

### US-003: Backward Compatibility
**As a** user
**I want to** have my existing config continue to work
**So that** I don't have to update everything after an update

**Acceptance Criteria**:
- Given old config with `content_regex_denylist`, when daemon loads, then it works (migrated)
- Given new config with `content_denylist`, when daemon loads, then it works normally

---

## Config Schema

### Before (Current)

```json
{
  "content_regex_denylist": ["^OTP:", "SECRET", ".token$"]
}
```

### After (New)

```json
{
  "content_denylist": ["OTP:", "SECRET", ".token"],
  "content_pattern_mode": "substring"
}
```

Supported modes:
- `substring` (default): Pattern appears anywhere in content
- `prefix`: Content starts with pattern
- `suffix`: Content ends with pattern
- `exact`: Content exactly matches pattern

---

## Migration Path

1. Daemon loads config
2. If `content_regex_denylist` exists and `content_denylist` doesn't, migrate:
   - Copy `content_regex_denylist` to `content_denylist`
   - Set `content_pattern_mode` to "substring"
3. Save migrated config (add `_migrated` flag to avoid re-migration)
4. Use `content_denylist` and `content_pattern_mode` going forward

---

## Dependencies

- None (standalone config change)

---

**Last Updated**: Phase 15