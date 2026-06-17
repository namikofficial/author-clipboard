# Technical Design: Snippet Token Replacement & Preview

> Implementation approach, file-by-file.

---

## 1. New module: `crates/shared/src/template.rs`

### Parser Strategy

Single-pass scanner over `&str` using a `let mut i = 0;` index. No
regex — regex would be slower than a hand-written scanner for the
small, fixed set of variable names. State is a plain `enum ParserState
{ Text, VarName }`.

```rust
pub fn render(input: &str, ctx: &RenderContext) -> (String, Option<usize>) {
    let mut out = String::with_capacity(input.len());
    let mut cursor: Option<usize> = None;
    let bytes = input.as_bytes();
    let mut i = 0;
    let now = ctx.now.unwrap_or_else(Utc::now);

    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'$' => { out.push('$'); i += 2; }
                b'{' => {
                    // Scan for matching '}'
                    let start = i + 2;
                    let mut end = start;
                    while end < bytes.len() && bytes[end] != b'}' { end += 1; }
                    if end >= bytes.len() {
                        // Unclosed — preserve literally
                        out.push_str(&input[i..]);
                        break;
                    }
                    let name = &input[start..end];
                    match resolve(name, &ctx, now) {
                        Resolved::Text(s) => out.push_str(&s),
                        Resolved::Cursor => {
                            cursor = Some(out.len());
                        }
                        Resolved::Unknown(_) => {
                            out.push_str("${");
                            out.push_str(name);
                            out.push('}');
                        }
                    }
                    i = end + 1;
                }
                _ => { out.push('$'); i += 1; }
            }
        } else {
            // Push one UTF-8 char (not just one byte) — input may be
            // multi-byte. Use char_indices for correctness.
            let next = input[i..].chars().next().unwrap();
            out.push(next);
            i += next.len_utf8();
        }
    }

    (out, cursor)
}
```

### Random string helper

```rust
fn random_alnum(n: usize) -> String {
    use rand::Rng;
    let clamped = n.clamp(1, 128);
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(clamped)
        .map(|b| CHARSET[(b as usize) % CHARSET.len()] as char)
        .collect()
}
```

The project already pulls `rand` for encryption, so this is free.

### UTF-8 correctness

The outer loop walks `bytes` but must push whole chars — using
`input[i..].chars().next().unwrap()` and `next.len_utf8()` keeps us safe
for non-ASCII template content (e.g. Cyrillic, emoji).

## 2. IPC addition (`crates/shared/src/ipc.rs`)

Add `RenderSnippet { id: i64 }` to `IpcCommand`. Add `RenderedSnippet`
to `IpcResponse` variants (or include the fields inline in the JSON
response — see choice below).

**Choice**: extend the response with two fields rather than introducing
a new variant. The handler returns:

```rust
IpcResponse::Ok(serde_json::json!({
    "content": rendered,
    "cursor_offset": cursor,
}))
```

This keeps `IpcResponse` simple. Errors use the existing `Err` variant
with code `"SNIPPET_NOT_FOUND"`.

## 3. Daemon handler (`crates/clipboard-daemon/src/main.rs`)

```rust
IpcCommand::RenderSnippet { id } => {
    let snippet = match self.db.get_snippet(id) {
        Ok(Some(s)) => s,
        Ok(None) => return IpcResponse::err(
            "SNIPPET_NOT_FOUND",
            format!("No snippet with id={id}"),
        ),
        Err(e) => return IpcResponse::err("DB_ERROR", format!("{e}")),
    };
    let ctx = RenderContext {
        now: Some(chrono::Utc::now()),
        clipboard: self.last_content.clone(),
        user: std::env::var("USER").ok().or_else(|| std::env::var("LOGNAME").ok()),
        hostname: hostname::get().ok().and_then(|h| h.into_string().ok()),
    };
    let (rendered, cursor) = template::render(&snippet.content, &ctx);
    IpcResponse::ok(serde_json::json!({
        "content": rendered,
        "cursor_offset": cursor,
    }))
}
```

Add `db.get_snippet(id) -> SqlResult<Option<Snippet>>` to db.rs (one-row
helper).

## 4. Picker preview (`crates/shared/src/picker.rs`)

**Decision**: `PickerEntry` does NOT gain a new field. Callers that
want a rendered preview for snippets compute it from `entry.content`
via `crate::template::render_now`. This keeps `PickerEntry` unchanged
and lets each UI pick how to surface the preview (label, secondary
line, tooltip). UI-gtk and applet call sites are added in
`crates/ui-gtk/src/pages/snippets.rs` and the applet's snippets
page; `ctl snippet list` includes the rendered text in its JSON
output when `--preview` is passed.

```rust
// Example call site:
fn snippet_preview_label(entry: &PickerEntry) -> String {
    if entry.source == PickerSource::Snippets {
        let (rendered, _) = template::render_now(&entry.content);
        truncate(&rendered, 80)
    } else {
        entry.subtitle.clone().unwrap_or_default()
    }
}
```

The `snippet_preview()` function continues to return a normal
`PickerEntry` with the raw template in `content`.

## 5. CLI (`crates/ctl/src/main.rs`)

Add an `ExpandSnippet` subcommand with flags:

```
author-clipboard-ctl expand-snippet <NAME_OR_ID> [--stdout] [--cursor-offset]
```

- Default: copy to clipboard + print to stdout.
- `--stdout`: skip the clipboard copy.
- `--cursor-offset`: print `text\t<offset>\n` instead of `text\n`.

Lookup is name-first, falls back to id if the argument parses as i64.
Not-found returns exit code 3 with an error message.

## 6. UI-gtk preview row (`crates/ui-gtk/src/pages/snippets.rs`)

Add a non-editable preview label below the content `Entry`. Update via
a closure connected to `content_entry.connect_changed`. The preview
uses `template::render_now` against the content text.

For applet-side rendering, the existing `PageId::Snippets` page already
renders snippet lists — extend it to show a one-line preview under each
row using the same `template::render_now` helper.

## 7. Tests (`crates/shared/src/template.rs`)

Inline `#[cfg(test)] mod tests` with the following cases:

| # | Input | Expected output / behaviour |
|---|---|---|
| 1 | `Hello, world!` | `("Hello, world!", None)` |
| 2 | `${date}` with fixed `now` | deterministic date |
| 3 | `${time}` with fixed `now` | deterministic time |
| 4 | `${year}`, `${month}`, `${day}` | each formatted with leading zero |
| 5 | `${hour}`, `${minute}`, `${second}` | 24h clock, zero-padded |
| 6 | `${unix}` | epoch seconds |
| 7 | `${uuid}` | matches UUID v4 regex |
| 8 | `${random:8}` | 8 alphanumeric chars |
| 9 | `${random:0}` and `${random:9999}` | clamped to 1..=128 |
| 10 | `${cursor}` | empty insertion + non-None offset |
| 11 | `Hi ${name}!` | `("Hi ${name}!", None)` — unknown preserved |
| 12 | `$$literal` | `"$literal"` (the `$$` escapes) |
| 13 | `\$literal` | `"$literal"` |
| 14 | `${` (unclosed) | preserved literally |
| 15 | `}` alone | preserved literally |
| 16 | Empty input | `("", None)` |
| 17 | Unicode input (`Здравствуй, ${name}!`) | preserved with UTF-8 correctness |
| 18 | `${clipboard}` with long clipboard | truncated to 1024 bytes + `…` |

Plus an integration test in `crates/shared/src/db.rs` for
`db.get_snippet(id)`.

---

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Time-zone confusion (`${date}` local vs `${iso_date}` UTC) | Medium | Document explicitly in rustdoc + FEATURES.md. |
| `${clipboard}` payload blows up IPC | Medium | Truncate to 1 KiB. |
| Hand-written parser bug on edge UTF-8 | Low | Add Unicode test case + property-based check in a follow-up. |
| Cursor offset conflicts with quick-paste | Low | Out of scope; offset returned but not consumed. |

---

**Last Updated**: Phase 15 completion (June 2026)
