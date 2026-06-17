# Decisions: Snippet Token Replacement & Preview

> Deviations from naive / obvious choices, with the rationale and trade-offs.

---

## DECISION-001 — Closed set of built-in variables in v1 (no user-defined)

**Context**: Most "snippet" tools (TextExpander, Espanso, etc.) let users
define their own variables (`${client_name}` filled from a config file or
prompted at expand time). The PROJECT_PLAN.md checkbox is also open-ended:
"token replacement and preview" could mean either.

**Decision**: v1 ships a closed set of built-in variables only.
User-defined variables (with storage in `config.json` and/or per-snippet
metadata) are deferred.

**Why**:
- A closed set is auditable: every variable name maps to exactly one
  resolution rule, no chance of a typo in a config file silently breaking
  a snippet.
- No storage migration. The `snippets` table keeps its existing schema.
- Users can still get the same result with `${clipboard}` (paste the
  value once, then reuse the snippet) for many real-world cases.

**Trade-off**: less powerful than Espanso-class tools. Acceptable for v1;
we can extend in v2 once we see what variable names users actually want.

---

## DECISION-002 — Unknown variables preserved verbatim, no prompts

**Context**: Some snippet tools prompt the user interactively when an
unresolved variable is encountered. This makes the picker a multi-step
dialog and ties it tightly to the compositor event loop.

**Decision**: Unknown variables (`${name}` where `name` is not a
built-in) are preserved verbatim. No prompts, no warnings in v1.

**Why**:
- Picker UX stays single-keystroke (select → paste).
- No new IPC contract for "prompt then resume".
- Preserves the user's intent: they wrote `Hi ${name}, welcome!` knowing
  the receiver would fill it in. Same model.

**Trade-off**: less ergonomic for interactive use. Mitigated by the
clipboard + select-from-history workflow for variable values.

---

## DECISION-003 — No shell command substitution

**Context**: Tools like Espanso support `${shell:...}` to run commands
and inject output. Powerful but a large attack surface: a snippet that
runs `rm -rf ${HOME}` would happily nuke the user's home directory.

**Decision**: The renderer is pure-text substitution. There is no
`eval`, no `Command::new`, no process spawn. This is enforced
structurally — the only functions in `template.rs` are string
manipulation, `chrono` formatting, `rand`, `uuid`, and env reads.

**Why**:
- Snippets are user-authored content, but they live in a database that
  also receives daemon events. A future "auto-import snippets from URL"
  feature must not be able to sneak in `${shell:rm -rf /}` and have
  the daemon execute it.
- Clipboard history is sensitive. Mixing in shell execution makes the
  blast radius of a malicious snippet equal to "everything the user
  can do from their terminal".

**Trade-off**: power users lose a useful feature. Acceptable; they can
always use a small wrapper script.

---

## DECISION-004 — Cursor offset returned but not yet consumed

**Context**: A "smart-paste" that lands the caret at `${cursor}` would
be a great UX win. It requires coordination between the daemon (which
renders the text + offset) and the applet (which does the actual paste
via wtype / ydotool).

**Decision**: The IPC response includes `cursor_offset: Option<usize>`.
The applet ignores it for now. A follow-up feature ("Smart paste")
will plumb the offset through the quick-paste subsystem.

**Why**:
- Returning the offset is free (we already compute it).
- Including it now lets us ship the data half of the contract without
  taking on quick-paste subsystem changes.
- No "compatibility cliff" later when smart-paste lands — the IPC shape
  is stable.

---

## DECISION-005 — Hand-written scanner instead of `regex`

**Context**: A `${name}` parser is a 30-line state machine. The `regex`
crate (already in the workspace after the Phase 15 regex denylist work)
could express it in one match arm.

**Decision**: Hand-written scanner.

**Why**:
- Faster: ~10 ns per char vs the regex engine's setup overhead.
- Trivial to reason about for security review (no regex backtracking
  surprises, no accidental catastrophic backtracking on adversarial
  snippet content).
- Avoids the engine's Unicode handling nuances for what is a simple
  bracket-matching problem.

**Trade-off**: a few more lines of code. Acceptable for a small, fixed
grammar.

---

## DECISION-006 — `${clipboard}` truncated to 1 KiB

**Context**: The daemon's `last_content` is whatever the user most
recently copied — possibly a multi-MB image or a huge SQL dump.
Embedding that into an IPC response would blow up the socket buffer.

**Decision**: `${clipboard}` resolves to at most 1024 bytes plus a `…`
ellipsis character (so 1025 bytes total). Truncation happens at a
UTF-8 char boundary.

**Why**:
- IPC payloads stay bounded.
- Most "current clipboard" use cases (e.g. wrapping a URL in a
  Markdown link) need the first few hundred bytes, not the whole thing.

---

**Last Updated**: Phase 15 completion (June 2026)
