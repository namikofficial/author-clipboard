# Review Checklist: {feature-name}

> Pre-merge review criteria.

---

## Requirements Coverage

- [ ] All user stories have acceptance criteria
- [ ] All acceptance criteria are verified by tests
- [ ] Edge cases are identified and handled
- [ ] Out-of-scope items are explicitly documented

---

## Code Quality

- [ ] `just verify` passes
- [ ] No clippy warnings (pedantic, `-D warnings`)
- [ ] Rustfmt applied
- [ ] No `#![allow(clippy::all)]` without justification
- [ ] Public APIs have doc comments

---

## Architecture

- [ ] Follows layered architecture (controller → service → repository)
- [ ] No direct database calls from controllers
- [ ] No business logic in repositories
- [ ] Error handling via `thiserror` in libraries, `anyhow` in binaries

---

## Security

- [ ] Sensitive data not logged
- [ ] Input validation on all boundaries
- [ ] IPC permissions checked
- [ ] Encryption applied where required
- [ ] No new `unsafe` blocks (or contained in dedicated modules)

---

## Testing

- [ ] Unit tests for new public APIs
- [ ] Integration tests for critical paths
- [ ] Tests pass: `cargo test --all`
- [ ] Manual testing completed for UI changes

---

## Documentation

- [ ] Feature spec updated if implementation differs from design
- [ ] `decisions.md` updated with rationale for any deviations
- [ ] `AGENTS.md` updated if new conventions introduced

---

## Performance

- [ ] Database queries use indexes
- [ ] No blocking operations on main async executor
- [ ] Memory usage acceptable

---

## Breaking Changes

- [ ] No breaking changes to IPC protocol (or version incremented)
- [ ] Database migrations are additive only
- [ ] CLI help text updated if new commands added

---

**Last Updated**: {date}