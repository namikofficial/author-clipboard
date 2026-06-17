# Domain Model: Phase 15 Denylist Completions

> Data structures and state for the regex denylist mode and `app_denylist` config.

---

## New / Changed Types (in `crates/shared/src/config.rs`)

```rust
/// Pattern matching mode for the content denylist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentPatternMode {
    Substring,
    Prefix,
    Suffix,
    Exact,
    /// Match each entry in `content_denylist` as a regular expression
    /// against the candidate content. Invalid patterns are logged once
    /// at load time and treated as non-matching.
    Regex,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    // ... existing fields ...

    /// Source-app ignore list (basenames, case-insensitive).
    /// Items whose `source_app` matches any entry here are dropped
    /// before storage. Currently a no-op because wlr-data-control
    /// does not expose source-app info — see 09-decisions.md.
    #[serde(default)]
    pub app_denylist: Vec<String>,
}
```

## New Methods

```rust
impl Config {
    /// Returns `true` when the candidate app should be denied.
    ///
    /// Matching rules:
    /// - `None` always returns `false` (no source-app info available).
    /// - Comparison is against `app.rsplit('/').next().unwrap_or(app)`
    ///   so paths like `/usr/bin/firefox` match a rule of `firefox`.
    /// - Comparison is ASCII-case-insensitive.
    /// - Empty rule strings never match.
    pub fn is_app_denied(&self, source_app: Option<&str>) -> bool { ... }

    /// Same as `is_content_denied`, but for `Regex` mode compiles
    /// patterns lazily on first call and caches the result.
    pub fn is_content_denied(&self, content: &str) -> bool { ... }
}
```

### Caching Strategy

A new private field on `Config`:

```rust
/// Compiled regexes (one per `content_denylist` entry when mode = Regex).
/// Populated lazily on first `is_content_denied` call after a successful
/// `Config::load` / `Config::clone`. Invalid patterns are stored as `None`.
#[serde(skip)]
pub(super) compiled_regex_cache: std::sync::OnceLock<Vec<Option<regex::Regex>>>,
```

`OnceLock` (stable since 1.70) is sufficient because `is_content_denied` is
called from a single task at a time on the daemon's event loop.

## Wire / IPC Changes

The daemon's `IpcCommand::GetConfig` handler currently emits `mime_denylist`
and `content_denylist`. Add `app_denylist` and `content_pattern_mode` for
visibility:

```json
{
  "app_denylist": ["keepassxc"],
  "content_pattern_mode": "substring",
  "content_denylist": []
}
```

No new IPC command is required.

---

**Last Updated**: Phase 15 completion (June 2026)
