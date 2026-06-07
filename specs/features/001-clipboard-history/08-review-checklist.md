# Review Checklist: Clipboard History

> Pre-merge review criteria for the clipboard history feature.

---

## Requirements Coverage

- [x] All user stories have acceptance criteria
- [x] All acceptance criteria are verified by tests
- [x] Edge cases are identified and handled
- [x] Out-of-scope items are explicitly documented

---

## Code Quality

- [x] `just verify` passes
- [x] No clippy warnings (pedantic, `-D warnings`)
- [x] Rustfmt applied
- [x] No `#![allow(clippy::all)]` without justification
- [x] Public APIs have doc comments

---

## Architecture

- [x] Follows layered architecture (controller → service → repository)
- [x] No direct database calls from controllers (CLI routes through IPC per Feature 012)
- [x] No business logic in repositories
- [x] Error handling via `thiserror` in libraries, `anyhow` in binaries

---

## Security

- [x] Sensitive data not logged
- [x] Input validation on all boundaries
- [x] IPC permissions checked
- [x] Encryption applied where required (Feature 006)
- [x] No new `unsafe` blocks

---

## Testing

- [x] Unit tests for new public APIs
- [x] Integration tests for critical paths
- [x] Tests pass: `cargo test --all`
- [x] Manual testing completed for UI changes

---

## Performance

- [x] Database queries use indexes
- [x] No blocking operations on main async executor
- [x] Memory usage acceptable

---

## Breaking Changes

- [x] No breaking changes to IPC protocol (or version incremented)
- [x] Database migrations are additive only
- [x] CLI help text updated if new commands added

---

## Notes

This feature is implemented in v0.5.0. The checklist is complete for the initial implementation.

**Phase 15 update**: Feature 012 (Service API Normalization) will update CLI to route through IPC, ensuring the architecture rule "No direct database calls from controllers" is fully enforced.

---

**Last Updated**: Phase 15