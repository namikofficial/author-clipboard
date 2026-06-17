# Domain Model: Snippet Token Replacement & Preview

> Public API of the renderer, IPC contract, and built-in variable table.

---

## New Module: `crates/shared/src/template.rs`

```rust
//! Snippet template rendering.
//!
//! Syntax: `${name}` substitutes a built-in variable. `$$` and `\$` escape
//! to a literal `$`. Unknown variables are preserved verbatim so the user
//! can manually fill them in the pasted text.
//!
//! **Security**: this module never executes anything. There is no `eval`,
//! no shell, no script — only textual substitution against a fixed set
//! of named variables. See `09-decisions.md` in
//! `specs/features/026-snippet-templates/` for the rationale.

use chrono::{DateTime, Utc};

/// Context that the renderer reads built-in variables from.
///
/// All fields are optional; missing values resolve to empty strings (with
/// one exception: missing `now` falls back to `Utc::now()` so single-arg
/// callers don't have to construct a clock).
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    pub now: Option<DateTime<Utc>>,
    pub clipboard: Option<String>,
    pub user: Option<String>,
    pub hostname: Option<String>,
}

/// Render a snippet template to its expanded form.
///
/// Returns `(rendered_text, cursor_offset)`. `cursor_offset` is the byte
/// position in `rendered_text` where the caret should land after paste
/// (from the `${cursor}` marker); `None` if the template did not use
/// `${cursor}`.
#[must_use]
pub fn render(input: &str, ctx: &RenderContext) -> (String, Option<usize>);

/// Convenience: render with `now = Utc::now()` and everything else empty.
#[must_use]
pub fn render_now(input: &str) -> (String, Option<usize>);
```

### Internal Parser

The renderer is a single-pass scanner over `input`:

| State | Trigger | Action |
|---|---|---|
| Default | `$` | peek next char; if `{` → enter Var; if `$` → literal `$`; if `\` not relevant here; else literal `$` |
| Default | anything else | append to output |
| Var | `}` | resolve name via the table; emit value; return to Default |
| Var | other | accumulate into name |
| Var | end of input | emit `${` + accumulated name literally |

The cursor offset is the byte length of `output` at the moment `${cursor}`
is consumed.

### Built-in Variable Table

```rust
fn resolve(name: &str, ctx: &RenderContext) -> Resolved {
    use Resolved::*;
    match name {
        "date"         => Text(ctx.now_or_now().format("%Y-%m-%d").to_string()),
        "time"         => Text(ctx.now_or_now().format("%H:%M:%S").to_string()),
        "datetime"     => Text(ctx.now_or_now().format("%Y-%m-%d %H:%M:%S").to_string()),
        "iso_date"     => Text(ctx.now_or_now().format("%Y-%m-%d").to_string()),
        "iso_time"     => Text(ctx.now_or_now().format("%H:%M:%SZ").to_string()),
        "iso_datetime" => Text(ctx.now_or_now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        "year"  => Text(ctx.now_or_now().format("%Y").to_string()),
        "month" => Text(ctx.now_or_now().format("%m").to_string()),
        "day"   => Text(ctx.now_or_now().format("%d").to_string()),
        "hour"  => Text(ctx.now_or_now().format("%H").to_string()),
        "minute"=> Text(ctx.now_or_now().format("%M").to_string()),
        "second"=> Text(ctx.now_or_now().format("%S").to_string()),
        "unix"  => Text(ctx.now_or_now().timestamp().to_string()),
        "uuid"  => Text(uuid::Uuid::new_v4().to_string()),
        "cursor"=> Cursor,
        "clipboard" => Text(truncate(&ctx.clipboard, 1024)),
        "user"     => Text(ctx.user.clone().unwrap_or_default()),
        "hostname" => Text(ctx.hostname.clone().unwrap_or_default()),
        n if n.starts_with("random:") => Text(random_alnum(n.len_after_colon())),
        _ => Unknown(name.to_string()),
    }
}
```

## IPC Contract

```rust
// shared/src/ipc.rs — additions only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum IpcCommand {
    // ...existing commands...

    /// Render a snippet template against the daemon's current context.
    /// Returns the rendered text plus an optional caret offset (from
    /// `${cursor}`). See specs/features/026-snippet-templates/.
    RenderSnippet { id: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RenderedSnippet {
    Ok {
        content: String,
        cursor_offset: Option<usize>,
    },
    NotFound { id: i64 },
}
```

The daemon builds a `RenderContext` from app state (`now`, the
`last_content` it last captured, `whoami`-style user lookup, hostname
read once at startup).

## Picker Preview

`crates/shared/src/picker.rs` does NOT gain a new field on
`PickerEntry`. Instead, snippet previews are computed at the call
site using `crate::template::render_now(&entry.content)`. The UI
checks `entry.source == PickerSource::Snippets` before rendering the
preview, keeping non-snippet entries zero-cost.

The `snippet_preview()` function continues to return a normal
`PickerEntry` with the raw template in `content`; callers (UI-gtk,
applet, `ctl snippet list`) compute the preview text from
`entry.content` when they need it.

Rationale: adding `preview` to `PickerEntry` would force a `Default`
impl and touch ~17 call sites that fully construct the struct.
Computing the preview on the consumer side keeps `PickerEntry`
unchanged and lets each UI decide how to display it (label, secondary
line, tooltip, etc.).

---

**Last Updated**: Phase 15 completion (June 2026)
