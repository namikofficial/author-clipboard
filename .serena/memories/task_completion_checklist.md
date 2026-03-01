# Task Completion Checklist

When a coding task is completed, run these checks:

1. **Format**: `just fmt` (auto-fix) or `just fmt-check` (verify)
2. **Lint**: `just lint` (check) or `just lint-fix` (auto-fix)
3. **Test**: `just test` (run all tests)
4. **Build**: `just build` (ensure compilation)

Or use the all-in-one command:
- `just verify` — runs: fmt-check → lint → test → build

## Quick Fix Workflow
If there are issues:
1. `just fix` — auto-format + auto-fix clippy
2. `just verify` — confirm everything passes

## Commit Convention
Commits must follow Conventional Commits (enforced by git hook):
- `feat(scope): description` for new features
- `fix(scope): description` for bug fixes
- See `docs/CONTRIBUTING.md` for full list of types
