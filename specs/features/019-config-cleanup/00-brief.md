# Feature Brief: Configuration Cleanup

> Rename `content_regex_denylist` to `content_denylist` and clarify its behavior as simple patterns, not regex.

---

## Problem Statement

The config field `content_regex_denylist` is misleading because:
1. It doesn't support full regex (only prefix/suffix/substring)
2. The README and code comments acknowledge this: "despite the legacy field name"
3. New users may expect regex and be confused when it doesn't work

## Proposed Solution

1. Rename `content_regex_denylist` to `content_denylist`
2. Add a `content_pattern_mode` field with options: `prefix`, `suffix`, `substring` (default), `exact`
3. Update all config file examples and documentation

## Goals

- Clear naming that reflects actual behavior
- Config migration path (old name still works, new name preferred)
- Documentation clarifies supported patterns

## Non-Goals

- Adding full regex support (out of scope)
- Breaking existing configurations

## Stakeholders

All users who configure content filtering.

---

**Created**: Phase 15 (Post-Research)
**Status**: Draft