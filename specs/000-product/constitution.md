# Project Constitution

> Non-negotiable rules that govern how author-clipboard is built and maintained.

**Prompt creates spec. Spec creates code. Code must pass spec.**

---

## Spec-Driven Development Rules

These rules apply to all non-trivial changes:

1. **Do not write code first.** Create or update a feature spec under `/specs/features`.
2. **Requirements must include acceptance criteria.** Each criterion must be independently verifiable.
3. **Design must list affected files and modules.** No unspecified side-effects.
4. **Tasks must be atomic and independently verifiable.** One task, one goal.
5. **Implement one task at a time.** Do not combine unrelated changes.
6. **After implementation, verify against acceptance criteria.** Tests must pass.
7. **If implementation differs from spec, update `decisions.md`.** Document why.
8. **Never perform unrelated refactors during a task.** Stay focused.

---

## Agent Roles

### Spec Agent
- Turn rough product intent into requirements
- Identify ambiguity and ask clarifying questions
- Define acceptance criteria and out-of-scope items
- **Never write implementation code**

### Architect Agent
- Read `requirements.md`
- Inspect existing repo structure
- Propose minimal technical design
- Preserve existing conventions
- Avoid overengineering
- Document tradeoffs in `decisions.md`
- **Never write code**

### Task Planner Agent
- Convert approved specs into atomic tasks
- Each task must include: goal, files to edit, verification command
- Order tasks by dependency
- **Never write code**

### Implementation Agent
- Implement only the current task
- Do not modify files outside the task unless necessary
- Run relevant tests after implementation
- Update task status
- Update `decisions.md` if implementation differs from design

### Review Agent
- Check: requirements coverage, security issues, tenant isolation, RBAC correctness, test coverage
- Do not add new features

---

## Non-Negotiable Code Rules

- **Rust strict mode enabled.** No `#![allow(clippy::all)]` without documented justification.
- **No `anyhow::Error` in library crates.** Use `thiserror::Error` for public error types.
- **No `unsafe` outside dedicated `unsafe` modules.** Contained and documented.
- **All public APIs must have doc comments.** `///` not `//`.
- **Tests required for new functionality.** Unit tests in `#[cfg(test)]` modules.
- **Format on save enforced.** `rustfmt` must pass.
- **Clippy pedantic warnings as errors.** `-D warnings` in CI.
- **Conventional commits enforced.** `<type>(<scope>): <description>`.

---

## Architecture Constraints

### Layered Architecture

```
Controllers (CLI/IPC) → Services → Repositories → Database
```

- **Controllers** handle IPC, CLI parsing, and request/response transformation only
- **Services** contain business logic and orchestrate repositories
- **Repositories** handle data access and raw SQL
- **No direct database calls from controllers**
- **No business logic in repositories**

### Crate Boundaries

| Crate | Responsibility |
|-------|----------------|
| `clipboard-daemon` | Wayland clipboard monitoring, IPC daemon |
| `applet` | libcosmic UI, user interaction |
| `shared` | DB schema, config, types, image_store, picker logic |
| `ctl` | CLI tool, IPC client |
| `hypr-picker` | GTK4 layer-shell native picker |

### IPC Protocol

- Unix socket in `$XDG_RUNTIME_DIR` (fallback: private cache dir, never `/tmp`)
- JSON request/response over socket
- All commands: `toggle`, `show`, `hide`, `ping`, `history`, `status`, `clear`, `export`, `config`, `picker`

---

## Security Requirements

- **Sensitive content detection required** for: passwords, OTPs, JWTs, API keys, SSH keys, AWS credentials, URI credentials, high-entropy secrets
- **Encryption at rest** with AES-256-GCM when `encrypt_sensitive` enabled
- **IPC socket permissions** 0700 directory, 0600 key file
- **No raw sensitive data in logs** — only structured audit events
- **Incognito mode** pauses all capture when `.incognito` flag exists
- **Screen lock detection** clears sensitive items via `loginctl` or D-Bus `org.freedesktop.ScreenSaver`

---

## Database Rules

- **SQLite with WAL mode** (`PRAGMA journal_mode=WAL`)
- **FTS5 virtual table** for full-text search with LIKE fallback
- **Parameterized queries only** — no string interpolation
- **Migrations** via versioned schema in `shared/src/db/migrations.rs`
- **No schema changes outside migrations**

---

## Wayland Integration Rules

- **COSMIC**: Requires `COSMIC_DATA_CONTROL_ENABLED=1` env var
- **Hyprland/Sway**: Uses `wlr-data-control` protocol via Wayland registry
- **No `/tmp` socket fallback** — use private cache directory
- **Graceful degradation** — if protocol unavailable, log error and continue

---

## Testing Requirements

| Type | Location | Command |
|------|----------|---------|
| Unit tests | Same file as `#[cfg(test)]` | `cargo test -p <crate>` |
| Integration tests | `tests/` directory in crate | `cargo test --all` |
| Full verification | CI pipeline | `just verify` |

Minimum coverage: All public APIs, database operations, sensitive detection patterns.

---

## Documentation Requirements

| Document | Purpose | Location |
|----------|---------|----------|
| Constitution | Non-negotiable rules | `/specs/000-product/constitution.md` |
| Architecture | System design | `/specs/000-product/architecture.md` |
| Glossary | Terminology | `/specs/000-product/glossary.md` |
| Feature spec | Feature requirements, design, tasks | `/specs/features/FFF-name/` |
| README | Quick start, install, config | `/README.md` |
| Development | Tooling, workflow | `/docs/DEVELOPMENT.md` |

---

## Commit Message Format

```
<type>(<scope>): <description>

[optional body]
```

**Valid types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

**Example**:
```
feat(daemon): add screen lock detection via loginctl

Implement org.freedesktop.ScreenSaver monitoring to trigger
sensitive item cleanup when session locks.
```

---

## Change Procedure

For any change to this constitution:

1. Propose change in a GitHub issue with rationale
2. Major changes require RFC PR with 7-day comment period
3. Minor clarifications can be PR-only
4. Changes apply to future work, not retroactively

---

**Last Updated**: Phase 14 Complete (v0.5.0)