# Task Plan: Hyprland-native UX & wlroots Polish

> Atomic, independently verifiable tasks for Phase 19 polish work.

---

## T001: Add `--json` to `author-clipboard-ctl status`

**Goal**: Emit a structured JSON payload from `Status` for the Waybar module.

**Files to Edit**:
- `crates/ctl/src/main.rs` (add `--json` / `--pretty` to the `Status` arm)
- `crates/ctl/src/main.rs` (add `run_status_json` helper)

**Verification**:
```bash
cargo build -p author-clipboard-ctl
./target/debug/author-clipboard-ctl status --json | jq .
```

---

## T002: Add `daemon_pid` to the `Ping` response

**Goal**: The Waybar module can show whether the daemon is alive and which
PID owns the IPC socket.

**Files to Edit**:
- `crates/shared/src/ipc.rs` (extend the Ping response if missing)
- `crates/clipboard-daemon/src/main.rs` (include `std::process::id()` in the response)

**Verification**:
```bash
cargo build -p author-clipboard-shared -p author-clipboard-ctl
./target/debug/author-clipboard-ctl ping
./target/debug/author-clipboard-ctl status --json | jq .daemon_pid
```

---

## T003: Waybar module script

**Goal**: Drop-in `contrib/waybar/clipboard.sh` that maps the JSON payload
to Waybar's `text` / `tooltip` / `class` / `alt`.

**Files to Create**:
- `contrib/waybar/clipboard.sh`
- `contrib/waybar/config.example.json`
- `contrib/waybar/style.css`
- `contrib/waybar/README.md`

**Verification**:
```bash
chmod +x contrib/waybar/clipboard.sh
shellcheck contrib/waybar/clipboard.sh
bash -n contrib/waybar/clipboard.sh
./contrib/waybar/clipboard.sh update | jq .
```

---

## T004: `just waybar-check` recipe

**Goal**: Local / CI helper that runs `shellcheck` and `bash -n` on the
Waybar module script.

**Files to Edit**:
- `justfile` (add `waybar-check` recipe)

**Verification**:
```bash
just waybar-check
```

---

## T005: AUR PKGBUILD polish

**Goal**: Add `optdepends` for `wofi`, `fuzzel`, `rofi` and confirm the
PKGBUILD builds all four binaries.

**Files to Edit**:
- `packaging/arch/PKGBUILD`

**Verification**:
```bash
cd packaging/arch
makepkg --printsrcinfo > .SRCINFO.new
diff -u .SRCINFO .SRCINFO.new
```

---

## T006: Regenerate `.SRCINFO`

**Goal**: Make `.SRCINFO` match the updated PKGBUILD.

**Files to Edit**:
- `packaging/arch/.SRCINFO` (regenerated)

**Verification**:
```bash
just aur-check
```

---

## T007: Nix flake verification

**Goal**: Confirm the flake builds (or fails cleanly) and the four
binary packages are exposed.

**Files to Edit**:
- `flake.nix` (no code change expected; document any polish in
  `09-decisions.md`)

**Verification**:
```bash
just nix-check
```

---

## T008: HYPRLAND.md Demo section

**Goal**: Add a `## Demo` section to `docs/HYPRLAND.md` with a reproducible
shell transcript and an ASCII layout sketch.

**Files to Edit**:
- `docs/HYPRLAND.md`

**Verification**:
```bash
# Manual: open docs/HYPRLAND.md and confirm the new section is well-formed
```

---

## T009: README updates

**Goal**: Reference the new Waybar module, AUR PKGBUILD, and Nix flake
from the README.

**Files to Edit**:
- `README.md`

**Verification**:
```bash
grep -n "Waybar\|AUR\|Nix flake" README.md
```

---

## T010: PROJECT_PLAN.md updates

**Goal**: Mark Phase 19 deliverables complete and bump the "Last Review"
date.

**Files to Edit**:
- `PROJECT_PLAN.md`

**Verification**:
```bash
grep -n "Phase 19" PROJECT_PLAN.md
```

---

## T011: Update feature spec status

**Goal**: Mark `01-hyprland-integration/06-task-plan.md` and the new
`021-hyprland-wlroots-polish` spec's `08-review-checklist.md` complete.

**Files to Edit**:
- `specs/features/021-hyprland-wlroots-polish/08-review-checklist.md` (create)
- `specs/features/021-hyprland-wlroots-polish/09-decisions.md` (create)

**Verification**:
```bash
ls specs/features/021-hyprland-wlroots-polish/
```

---

## Status

| Task | Status | Notes |
|------|--------|-------|
| T001 | Pending | Status JSON payload |
| T002 | Pending | Ping response `daemon_pid` |
| T003 | Pending | Waybar script + docs |
| T004 | Pending | justfile `waybar-check` |
| T005 | Pending | PKGBUILD optdepends |
| T006 | Pending | .SRCINFO regen |
| T007 | Pending | Flake check |
| T008 | Pending | HYPRLAND.md Demo |
| T009 | Pending | README updates |
| T010 | Pending | PROJECT_PLAN.md |
| T011 | Pending | Spec self-update |

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
