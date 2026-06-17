# Requirements: Phase 15 Denylist Completions

> Requirements for the two denylist additions that close out the remaining
> Phase 15 checkbox items in `PROJECT_PLAN.md`.

---

## User Stories

### US-001: Regex content denylist
**As a** power user
**I want** to set `content_pattern_mode: "regex"` and write a regular expression
**So that** I can match complex patterns (e.g. GitHub PATs, AWS keys, JWTs) with one rule instead of many

**Acceptance Criteria**:
- Given `content_pattern_mode: "regex"` and `content_denylist: ["^ghp_[A-Za-z0-9]{36}$"]` in `config.json`, when the user copies `ghp_abc123...`, then the item is **not** stored.
- Given an invalid regex (e.g. `[unclosed`), when the daemon loads the config, then it logs a warning naming the failing pattern and continues to start with that single pattern treated as no-match (fail-closed for security).
- Given a regex that matches the content, when `is_content_denied` is called repeatedly on the same content, then it returns `true` consistently (compiled once, cached).

### US-002: Source-app ignore rules
**As a** user
**I want** to set `app_denylist: ["keepassxc"]` in my config
**So that** clipboard items from KeePassXC (when source-app is available) are never recorded

**Acceptance Criteria**:
- Given `app_denylist: ["keepassxc"]` and `source_app = Some("keepassxc")`, when the daemon receives a new clipboard item, then `is_app_denied` returns `true`.
- Given `app_denylist: ["keepassxc"]` and `source_app = None`, when the daemon receives a new clipboard item, then `is_app_denied` returns `false` (nothing to match).
- Given `app_denylist: ["keepassxc"]` and `source_app = Some("KeePassXC")`, then matching is case-insensitive for the basename.
- Given an empty `app_denylist`, when the daemon starts, then no app-filter logging occurs and behavior is identical to today.

### US-003: Defaults are non-breaking
**As an** existing user
**I want** my current `config.json` to keep working after upgrade
**So that** I don't have to edit my config

**Acceptance Criteria**:
- A config file with no `app_denylist` key loads successfully (defaults to `[]`).
- A config file with no `content_pattern_mode` key keeps using `substring` (existing default).
- `just verify` (fmt + clippy + test + build) is green.

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | Add `ContentPatternMode::Regex` variant | Must | Uses `regex` crate. |
| FR-002 | Compile-and-cache compiled regexes on `Config::load` (or first call) | Must | Avoid recompiling per clipboard event. |
| FR-003 | Invalid regex logs once and is treated as non-matching | Must | Fail-closed for security. |
| FR-004 | Add `Config::app_denylist: Vec<String>` field with default `[]` | Must | Non-breaking. |
| FR-005 | Add `Config::is_app_denied(app: Option<&str>) -> bool` method | Must | Case-insensitive basename match. |
| FR-006 | Daemon calls `is_app_denied` before `insert_item` (text, html, files) | Must | Wire-only — current `source_app = None` keeps it a no-op. |
| FR-007 | Add `app_denylist` to `GetConfig` IPC response | Should | For UI/ctl parity. |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | `is_content_denied` latency per call | < 1 µs after first compile | Hot path runs on every clipboard event. |
| NFR-002 | Memory overhead per compiled regex | Bounded by `regex` crate defaults | No unbounded caching. |
| NFR-003 | No new clippy warnings under `-D warnings` | 0 warnings | Project rule. |
| NFR-004 | Test coverage for new matchers | ≥ 4 tests per matcher | Happy + adversarial. |

---

## Edge Cases

| Case | Handling |
|------|----------|
| Invalid regex in `content_denylist` | Log once at load, treat as no-match, daemon continues. |
| Empty content | Already handled by capture-path empty check; no denylist call. |
| `source_app` is `None` | `is_app_denied(None)` returns `false`. |
| `source_app` has full path (e.g. `/usr/bin/firefox`) | Match against basename only, case-insensitive. |
| `app_denylist` contains the empty string | Treated as no-match (empty string can't match a non-empty app). |
| Unicode in content or app name | `regex` crate handles by default. |
| Very long content (multi-MB) | Regex compiled once; match cost bounded by regex engine. |

---

## Out of Scope

- Changing `mime_denylist` to support regex (separate concern; MIME types are
  literal strings and rarely need it).
- An `--app-ignore` CLI flag for `author-clipboard-ctl` (not requested).
- Compositor-specific source-app discovery (requires protocol work outside this spec).

---

**Last Updated**: Phase 15 completion (June 2026)
