//! Snippet template rendering.
//!
//! Supports a small, fixed set of built-in variables referenced by
//! `${name}` (or `${name:arg}` for `random:N`). Unknown variable names
//! are preserved verbatim so the receiver of the pasted text can fill
//! them in by hand. The escape sequences `$$` and `\$` produce a literal
//! `$`; an unclosed `${` is preserved literally so a malformed template
//! never crashes the daemon.
//!
//! **Security**: this module is a pure-text substitution engine. It
//! never spawns processes, never evaluates expressions, never touches
//! the network. The only side effects are reading env vars (`USER`,
//! `LOGNAME`) and generating randomness via `rand`. See
//! `specs/features/026-snippet-templates/09-decisions.md` for the
//! rationale.
//!
//! ## Example
//!
//! ```
//! use author_clipboard_shared::template::{render, render_now, RenderContext};
//!
//! let (out, cursor) = render_now("Hello ${user}, today is ${date}.");
//! assert!(out.starts_with("Hello "));
//! assert!(out.contains("today is "));
//! assert!(cursor.is_none());
//!
//! let ctx = RenderContext {
//!     now: None,
//!     user: Some("alice".into()),
//!     ..Default::default()
//! };
//! let (out, _) = render("Hi ${user}!", &ctx);
//! assert_eq!(out, "Hi alice!");
//! ```

use chrono::{DateTime, SecondsFormat, Utc};
use rand::Rng;

/// Maximum length of an embedded clipboard value (in bytes) before it
/// is truncated with an ellipsis. Prevents IPC payload blow-up when the
/// user has a multi-MB clipboard item.
const CLIPBOARD_EMBED_MAX_BYTES: usize = 1024;

/// Maximum length of `${random:N}` output. Anything bigger is clamped.
const RANDOM_MAX_LEN: usize = 128;

/// Context that the renderer reads built-in variables from.
///
/// All fields are optional; missing values resolve to empty strings
/// (the only exception is `now`, which falls back to `Utc::now()` so
/// single-argument callers don't have to build a clock).
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    /// Override for the wall clock used by `${date}`, `${time}`, etc.
    pub now: Option<DateTime<Utc>>,
    /// Value for `${clipboard}` — usually the daemon's last-captured
    /// clipboard content.
    pub clipboard: Option<String>,
    /// Value for `${user}` — usually `std::env::var("USER")`.
    pub user: Option<String>,
    /// Value for `${hostname}` — usually `hostname::get()`.
    pub hostname: Option<String>,
}

/// Render a snippet template to its expanded form.
///
/// Returns `(rendered_text, cursor_offset)`:
/// - `rendered_text`: the substituted string
/// - `cursor_offset`: byte position where the caret should land after
///   paste, derived from the `${cursor}` marker. `None` if the
///   template did not use `${cursor}` (or used it more than once —
///   the last occurrence wins).
#[must_use]
pub fn render(input: &str, ctx: &RenderContext) -> (String, Option<usize>) {
    let now = ctx.now.unwrap_or_else(Utc::now);
    let mut out = String::with_capacity(input.len());
    let mut cursor: Option<usize> = None;
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Fast path: if we are not at '$', copy the next char and advance.
        if bytes[i] != b'$' {
            let ch = input[i..].chars().next().expect("i < len");
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        // We are at '$'. Peek the next byte.
        let Some(next) = bytes.get(i + 1) else {
            // Trailing '$' — emit literally.
            out.push('$');
            i += 1;
            continue;
        };

        match next {
            b'$' => {
                // `$$` → literal `$`
                out.push('$');
                i += 2;
            }
            b'{' => {
                // Scan for matching '}'.
                let start = i + 2;
                let mut end = start;
                while end < bytes.len() && bytes[end] != b'}' {
                    end += 1;
                }
                if end >= bytes.len() {
                    // Unclosed — preserve the rest literally and stop.
                    out.push_str(&input[i..]);
                    return (out, cursor);
                }
                let name = &input[start..end];
                match resolve(name, ctx, now) {
                    Resolved::Text(s) => out.push_str(&s),
                    Resolved::Cursor => {
                        cursor = Some(out.len());
                    }
                    Resolved::Unknown => {
                        // Preserve "${<name>}" verbatim so the user sees what
                        // they need to fill in.
                        out.push_str("${");
                        out.push_str(name);
                        out.push('}');
                    }
                }
                i = end + 1;
            }
            _ => {
                // `$X` where X is not `$` or `{` — literal `$`.
                out.push('$');
                i += 1;
            }
        }
    }

    (out, cursor)
}

/// Convenience: render with `now = Utc::now()` and everything else empty.
///
/// Useful for the picker preview where the daemon's full context isn't
/// available — the preview just needs "what would this look like today".
#[must_use]
pub fn render_now(input: &str) -> (String, Option<usize>) {
    render(input, &RenderContext::default())
}

// ── Resolver ────────────────────────────────────────────────────────────

enum Resolved {
    Text(String),
    Cursor,
    Unknown,
}

fn resolve(name: &str, ctx: &RenderContext, now: DateTime<Utc>) -> Resolved {
    let text = match name {
        // `${date}` and `${iso_date}` resolve to the same string in v1:
        // both format `now` (which is UTC) as `YYYY-MM-DD`. A future
        // revision could distinguish local vs UTC by calling
        // `now.with_timezone(&Local)` for `${date}` — see
        // `specs/features/026-snippet-templates/09-decisions.md`.
        "date" | "iso_date" => Some(now.format("%Y-%m-%d").to_string()),
        "time" => Some(now.format("%H:%M:%S").to_string()),
        "datetime" => Some(now.format("%Y-%m-%d %H:%M:%S").to_string()),
        "iso_time" => Some(now.format("%H:%M:%SZ").to_string()),
        "iso_datetime" => Some(now.to_rfc3339_opts(SecondsFormat::Secs, true)),
        "year" => Some(now.format("%Y").to_string()),
        "month" => Some(now.format("%m").to_string()),
        "day" => Some(now.format("%d").to_string()),
        "hour" => Some(now.format("%H").to_string()),
        "minute" => Some(now.format("%M").to_string()),
        "second" => Some(now.format("%S").to_string()),
        "unix" => Some(now.timestamp().to_string()),
        "uuid" => Some(uuid::Uuid::new_v4().to_string()),
        "cursor" => return Resolved::Cursor,
        "clipboard" => ctx.clipboard.as_deref().map(truncate_clipboard),
        "user" => ctx.user.clone().or_else(env_user),
        "hostname" => ctx.hostname.clone(),
        n if n.starts_with("random:") => {
            let arg = &n["random:".len()..];
            let len: usize = arg.parse().unwrap_or(1);
            Some(random_alnum(len))
        }
        _ => return Resolved::Unknown,
    };
    Resolved::Text(text.unwrap_or_default())
}

/// Read `$USER` (or `$LOGNAME` as fallback) for `${user}` when no
/// explicit value is in the context.
fn env_user() -> Option<String> {
    std::env::var("USER")
        .ok()
        .or_else(|| std::env::var("LOGNAME").ok())
}

/// Generate an alphanumeric random string of length `n`, clamped to
/// `1..=RANDOM_MAX_LEN`.
fn random_alnum(n: usize) -> String {
    use rand::distributions::Alphanumeric;
    let clamped = n.clamp(1, RANDOM_MAX_LEN);
    let mut rng = rand::thread_rng();
    (0..clamped)
        .map(|_| {
            let b: u8 = rng.sample(Alphanumeric);
            // Alphanumeric is already in the right charset; cast through
            // char to avoid accidentally picking up non-ASCII bytes.
            b as char
        })
        .collect()
}

/// Truncate the clipboard embed to `CLIPBOARD_EMBED_MAX_BYTES` plus a
/// trailing `…`. Cuts on a UTF-8 char boundary.
fn truncate_clipboard(s: &str) -> String {
    if s.len() <= CLIPBOARD_EMBED_MAX_BYTES {
        return s.to_owned();
    }
    let mut idx = CLIPBOARD_EMBED_MAX_BYTES;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    let mut out = String::with_capacity(idx + 3);
    out.push_str(&s[..idx]);
    out.push('\u{2026}');
    out
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 17, 14, 23, 8).unwrap()
    }

    fn ctx_with(now: DateTime<Utc>) -> RenderContext {
        RenderContext {
            now: Some(now),
            ..Default::default()
        }
    }

    // ── Passthrough ────────────────────────────────────────────────

    #[test]
    fn test_render_passthrough_plain() {
        let (out, c) = render_now("Hello, world!");
        assert_eq!(out, "Hello, world!");
        assert!(c.is_none());
    }

    #[test]
    fn test_render_passthrough_empty() {
        let (out, c) = render_now("");
        assert_eq!(out, "");
        assert!(c.is_none());
    }

    #[test]
    fn test_render_passthrough_lone_dollar() {
        let (out, _) = render_now("price: $5");
        assert_eq!(out, "price: $5");
    }

    // ── Built-ins (deterministic with fixed `now`) ─────────────────

    #[test]
    fn test_render_builtin_date_and_time() {
        let (out, _) = render("Date: ${date}, Time: ${time}", &ctx_with(fixed_now()));
        assert_eq!(out, "Date: 2026-06-17, Time: 14:23:08");
    }

    #[test]
    fn test_render_builtin_datetime_and_iso() {
        let (out, _) = render(
            "${datetime} | ${iso_date} | ${iso_time} | ${iso_datetime}",
            &ctx_with(fixed_now()),
        );
        assert_eq!(
            out,
            "2026-06-17 14:23:08 | 2026-06-17 | 14:23:08Z | 2026-06-17T14:23:08Z"
        );
    }

    #[test]
    fn test_render_builtin_components_zero_padded() {
        let (out, _) = render(
            "${year}-${month}-${day} ${hour}:${minute}:${second}",
            &ctx_with(fixed_now()),
        );
        assert_eq!(out, "2026-06-17 14:23:08");
    }

    #[test]
    fn test_render_builtin_unix() {
        let (out, _) = render("${unix}", &ctx_with(fixed_now()));
        // 2026-06-17 14:23:08 UTC
        assert_eq!(out, "1781706188");
    }

    #[test]
    fn test_render_builtin_uuid_changes_each_call() {
        let ctx = ctx_with(fixed_now());
        let (a, _) = render("${uuid}", &ctx);
        let (b, _) = render("${uuid}", &ctx);
        assert_ne!(a, b);
        // Validate UUID v4 format: 8-4-4-4-12 hex.
        assert_eq!(a.len(), 36);
        assert_eq!(a.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn test_render_builtin_random_clamps() {
        let ctx = ctx_with(fixed_now());
        let (zero, _) = render("${random:0}", &ctx);
        assert_eq!(zero.len(), 1);
        let (huge, _) = render("${random:9999}", &ctx);
        assert_eq!(huge.len(), RANDOM_MAX_LEN);
        let (eight, _) = render("${random:8}", &ctx);
        assert_eq!(eight.len(), 8);
        assert!(eight.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_render_builtin_cursor_offset() {
        let (out, c) = render("Hello, ${cursor}world!", &ctx_with(fixed_now()));
        assert_eq!(out, "Hello, world!");
        assert_eq!(c, Some(7));
    }

    #[test]
    fn test_render_builtin_cursor_multiple_last_wins() {
        let (out, c) = render("abc${cursor}xyz${cursor}END", &ctx_with(fixed_now()));
        assert_eq!(out, "abcxyzEND");
        assert_eq!(c, Some(6));
    }

    #[test]
    fn test_render_builtin_user_and_hostname_from_context() {
        let ctx = RenderContext {
            user: Some("alice".into()),
            hostname: Some("laptop".into()),
            ..ctx_with(fixed_now())
        };
        let (out, _) = render("${user}@${hostname}", &ctx);
        assert_eq!(out, "alice@laptop");
    }

    #[test]
    fn test_render_builtin_user_missing_is_empty() {
        // No env var set in test env? Either way, must not panic and
        // must be either the env value or empty.
        let (out, _) = render("Hi ${user}", &ctx_with(fixed_now()));
        assert!(out.starts_with("Hi "));
        assert!(out.len() <= 64);
    }

    #[test]
    fn test_render_builtin_clipboard_truncated() {
        let big = "x".repeat(CLIPBOARD_EMBED_MAX_BYTES + 100);
        let ctx = RenderContext {
            clipboard: Some(big.clone()),
            ..ctx_with(fixed_now())
        };
        let (out, _) = render("[${clipboard}]", &ctx);
        // Length: 1 ('[') + CLIPBOARD_EMBED_MAX_BYTES + 1 (ellipsis as 3 UTF-8 bytes) + 1 (']')
        // Ellipsis is '…' = 3 bytes, so total bytes = 1 + 1024 + 3 + 1 = 1029.
        assert!(out.starts_with('['));
        assert!(out.ends_with("\u{2026}]"));
    }

    #[test]
    fn test_render_builtin_clipboard_short_passes_through() {
        let ctx = RenderContext {
            clipboard: Some("hello".into()),
            ..ctx_with(fixed_now())
        };
        let (out, _) = render("cb=${clipboard}", &ctx);
        assert_eq!(out, "cb=hello");
    }

    // ── Escape and unknown ─────────────────────────────────────────

    #[test]
    fn test_render_escape_double_dollar() {
        let (out, _) = render("price: $${not_a_var}", &ctx_with(fixed_now()));
        assert_eq!(out, "price: ${not_a_var}");
    }

    #[test]
    fn test_render_escape_double_dollar_followed_by_text() {
        let (out, _) = render("$$literal", &ctx_with(fixed_now()));
        assert_eq!(out, "$literal");
    }

    #[test]
    fn test_render_unknown_variable_preserved() {
        let (out, _) = render("Hi ${name}, welcome!", &ctx_with(fixed_now()));
        assert_eq!(out, "Hi ${name}, welcome!");
    }

    #[test]
    fn test_render_empty_braces_preserved() {
        let (out, _) = render("a${}b", &ctx_with(fixed_now()));
        assert_eq!(out, "a${}b");
    }

    #[test]
    fn test_render_unclosed_brace_preserved() {
        let (out, _) = render("a${name and then more", &ctx_with(fixed_now()));
        assert_eq!(out, "a${name and then more");
    }

    // ── UTF-8 correctness ──────────────────────────────────────────

    #[test]
    fn test_render_utf8_passthrough() {
        let (out, _) = render("Здравствуй, мир!", &ctx_with(fixed_now()));
        assert_eq!(out, "Здравствуй, мир!");
    }

    #[test]
    fn test_render_utf8_with_unknown_var() {
        let (out, _) = render("Привет, ${name}!", &ctx_with(fixed_now()));
        assert_eq!(out, "Привет, ${name}!");
    }

    #[test]
    fn test_render_emoji_passthrough() {
        let (out, _) = render("👋 ${user} 🎉", &ctx_with(fixed_now()));
        // ${user} resolves to env value or empty. Verify emoji survives.
        assert!(out.starts_with("👋 "));
        assert!(out.ends_with(" 🎉"));
    }

    // ── Mixed / integration-ish ─────────────────────────────────────

    #[test]
    fn test_render_realistic_template() {
        let ctx = RenderContext {
            user: Some("namik".into()),
            clipboard: Some("https://example.com".into()),
            ..ctx_with(fixed_now())
        };
        let tmpl = "\
# Daily log for ${date}

- Author: ${user}
- Clipboard: ${clipboard}
- ID: ${uuid}

Notes:${cursor}
";
        let (out, cursor) = render(tmpl, &ctx);
        assert!(out.starts_with("# Daily log for 2026-06-17\n"));
        assert!(out.contains("- Author: namik"));
        assert!(out.contains("- Clipboard: https://example.com"));
        assert!(out.contains("- ID: "));
        assert!(out.ends_with("Notes:\n"));
        // `Notes:\n` is 6 bytes after the body; cursor offset should be
        // the byte length of everything up to and including the newline.
        assert_eq!(cursor, Some(out.len() - 1));
    }
}
