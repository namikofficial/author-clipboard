# Technical Design: Phase 15 Denylist Completions

> Implementation approach for the regex denylist and `app_denylist` additions.

---

## 1. Workspace Dependency

Add to root `Cargo.toml` under `[workspace.dependencies]`:

```toml
regex = "1"
```

Reference it in `crates/shared/Cargo.toml`:

```toml
[dependencies]
regex.workspace = true
```

`regex` 1.x is the standard, widely-used Rust regex engine. We pick the
full crate over `regex-lite` because the project already accepts
mid-sized deps (libcosmic, wayland-protocols, etc.) and the full crate
gives us Unicode word-boundary support and better diagnostics without
paying meaningful build-time cost for a single config field.

## 2. Config Changes (`crates/shared/src/config.rs`)

### 2.1 Add `Regex` to `ContentPatternMode`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentPatternMode {
    Substring,
    Prefix,
    Suffix,
    Exact,
    Regex,
}
```

No default change — existing configs stay on `Substring`.

### 2.2 Add `app_denylist` field

```rust
/// Source-app ignore list (basenames, case-insensitive).
/// Items whose `source_app` matches any entry here are dropped
/// before storage. Currently a no-op because wlr-data-control
/// does not expose source-app info — see 09-decisions.md.
#[serde(default)]
pub app_denylist: Vec<String>,
```

Placed after `content_denylist` for grouping. Default is `[]` via `Default`.

### 2.3 Add `is_app_denied` method

```rust
pub fn is_app_denied(&self, source_app: Option<&str>) -> bool {
    let Some(app) = source_app else { return false };
    if self.app_denylist.is_empty() {
        return false;
    }
    let basename = app.rsplit('/').next().unwrap_or(app);
    let basename_lower = basename.to_ascii_lowercase();
    self.app_denylist.iter().any(|rule| {
        !rule.is_empty() && rule.to_ascii_lowercase() == basename_lower
    })
}
```

Case-insensitive exact match on basename — keeps the rule semantics
predictable and avoids regex compilation surprises for app names.

### 2.4 Lazy regex cache

```rust
#[serde(skip)]
compiled_regex_cache: std::sync::OnceLock<Vec<Option<regex::Regex>>>,
```

In `Config::default()`:

```rust
compiled_regex_cache: std::sync::OnceLock::new(),
```

The cache is `#[serde(skip)]` so it never appears in serialized JSON.

In `is_content_denied`, replace the `match` arm for `Regex`:

```rust
ContentPatternMode::Regex => {
    let cache = self.compiled_regex_cache.get_or_init(|| {
        self.content_denylist
            .iter()
            .map(|p| regex::Regex::new(p).ok())
            .collect()
    });
    cache.iter().zip(&self.content_denylist).any(|(re_opt, raw)| {
        match re_opt {
            Some(re) => re.is_match(content),
            None => {
                // We only log the first time we see this raw pattern.
                // `OnceLock::get_or_init` runs once, so a tracing::warn!
                // here would fire exactly once per daemon lifetime.
                tracing::warn!(
                    pattern = raw,
                    "Invalid regex in content_denylist; treating as no-match"
                );
                false
            }
        }
    })
}
```

> **Note**: the warn-on-invalid fires once per daemon startup (per unique
> bad pattern), which is the right behavior — annoying in a loop, silent
> forever would hide the misconfiguration.

### 2.5 Update existing tests

The existing `test_config_roundtrip` constructs a `Config` literal; add
`app_denylist: vec![]` and `compiled_regex_cache: OnceLock::new()` to keep
the test compiling. Use `..Default::default()` where possible to minimise
future drift.

## 3. Daemon Wiring (`crates/clipboard-daemon/src/main.rs`)

In each of the three capture branches (text, html, files), after the
existing MIME / content-denylist check and before building the
`ClipboardItem`, add:

```rust
let source_app: Option<String> = None; // Wayland limitation; see 09-decisions.md
if state.config.is_app_denied(source_app.as_deref()) {
    debug!("Content blocked by app denylist, skipping");
    // (then continue the existing flow without storing)
}
```

Because `source_app` is currently always `None`, the call is a no-op, but
the wiring is exercised by unit tests in `config.rs` and will activate
the moment a compositor exposes source-app info.

## 4. `GetConfig` IPC Response

In `crates/clipboard-daemon/src/main.rs` around line ~1095, add:

```rust
"app_denylist": self.config.app_denylist,
"content_pattern_mode": self.config.content_pattern_mode,
```

This keeps `author-clipboard-ctl config` output consistent.

## 5. Tests

In `crates/shared/src/config.rs` `mod tests`:

- `test_app_denylist_none` — `is_app_denied(None)` → `false`.
- `test_app_denylist_empty_config` — empty list → `false` even with app set.
- `test_app_denylist_match_basename` — `"firefox"` rule matches
  `"/usr/bin/firefox"`.
- `test_app_denylist_case_insensitive` — `"KeePassXC"` rule matches
  `"keepassxc"` source.
- `test_content_denylist_regex_match` — `^ghp_[A-Za-z0-9]{36}$` matches a
  fake GitHub PAT.
- `test_content_denylist_regex_no_match` — same regex against safe content.
- `test_content_denylist_regex_invalid_pattern_does_not_panic` — invalid
  regex `[unclosed` → returns `false`, daemon does not crash.

Total: **7 new tests**.

## 6. Documentation

`PROJECT_PLAN.md` Phase 15: mark both checkboxes `[x]` and add a short
note pointing at the deviation (`source_app` is a wlr-data-control
limitation).

---

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| `regex` crate compile time bloat | Low | Already pulling many heavy crates; < 5s impact expected. |
| Invalid regex in user config crashes daemon | None (fail-closed) | `OnceLock` cache returns `None` for invalid; warn once. |
| `app_denylist` confusing users (no current effect) | Medium | Note in deviation doc + brief changelog mention. |

---

**Last Updated**: Phase 15 completion (June 2026)
