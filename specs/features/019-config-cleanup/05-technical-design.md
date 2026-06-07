# Technical Design: Configuration Cleanup

> Implementation approach for renaming and clarifying the content denylist.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/shared/src/config.rs` | Rename field, add pattern mode |
| `crates/shared/src/db.rs` | Update is_content_denied logic |
| `crates/ctl/src/main.rs` | Update config display |
| `crates/applet/src/settings.rs` | Update settings UI |

---

## Implementation Details

### config.rs changes

```rust
// Rename field and add pattern mode
pub struct Config {
    // ... existing fields ...

    /// Content patterns to deny (simple patterns, not regex)
    #[serde(default = "default_content_denylist", alias = "content_regex_denylist")]
    pub content_denylist: Vec<String>,

    /// How to match content_denylist patterns
    #[serde(default = "default_content_pattern_mode")]
    pub content_pattern_mode: ContentPatternMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentPatternMode {
    Substring, // default
    Prefix,
    Suffix,
    Exact,
}

fn default_content_pattern_mode() -> ContentPatternMode {
    ContentPatternMode::Substring
}

// Update is_content_denied to use pattern mode
impl Config {
    pub fn is_content_denied(&self, content: &str) -> bool {
        self.content_denylist.iter().any(|pattern| {
            match self.content_pattern_mode {
                ContentPatternMode::Substring => content.contains(pattern),
                ContentPatternMode::Prefix => content.starts_with(pattern),
                ContentPatternMode::Suffix => content.ends_with(pattern),
                ContentPatternMode::Exact => content == pattern,
            }
        })
    }

    // Add migration logic
    pub fn migrate_if_needed(&mut self) {
        // If old field exists and new field doesn't, migrate
        if !self.content_denylist.is_empty() || self.content_pattern_mode != ContentPatternMode::Substring {
            // Already using new format
            return;
        }
        // Otherwise, check for old field in config file
        // (handled at load time via serde alias)
    }
}
```

---

## Testing

1. Test old config migration
2. Test new config loading
3. Test pattern mode behavior for each mode

---

**Last Updated**: Phase 15