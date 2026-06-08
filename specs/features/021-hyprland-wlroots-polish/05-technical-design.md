# Technical Design: Hyprland-native UX & wlroots Polish

> Implementation approach for the Waybar module, the `status --json`
> payload, and the AUR/Nix polish.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/ctl/src/main.rs` | Add `--json` / `--pretty` to `Status`; emit structured payload |
| `crates/shared/src/db.rs` | Add `Database::stats()` accessor (already exists, just confirm) |
| `crates/shared/src/ipc.rs` | Add `daemon_pid` to `Ping` response if not present |
| `crates/clipboard-daemon/src/main.rs` | Emit `SIGUSR1` handler as no-op (deferred: do not block on this) |
| `contrib/waybar/clipboard.sh` | New — Waybar module script |
| `contrib/waybar/config.example.json` | New — Waybar module config snippet |
| `contrib/waybar/style.css` | New — example styling for the module |
| `contrib/waybar/README.md` | New — install / use instructions |
| `docs/HYPRLAND.md` | Append `## Demo` section with reproducible transcript + ASCII layout |
| `packaging/arch/PKGBUILD` | Verify deps (gtk4, gtk4-layer-shell) and binaries installed |
| `packaging/arch/.SRCINFO` | Regenerate to match PKGBUILD |
| `flake.nix` | Verify outputs cover all four binaries; add `signal` for refresh |
| `default.nix` | Verify mirror of flake.nix |
| `README.md` | Link to the new Waybar / AUR / Nix sections |
| `PROJECT_PLAN.md` | Mark Phase 19 deliverables complete |
| `justfile` | Add `waybar-check` recipe (runs `shellcheck` on the script) |

---

## `Status` command output

The existing `Status` command already prints a human-readable block. We
add `--json` to emit a single JSON object on stdout. The change is
small: branch on `--json` early in the `Command::Status` arm and call a
new `run_status_json()` helper that returns the structured payload.

The helper:

1. Tries `IpcClient::send_command(IpcCommand::Ping)` to set `running`.
2. If the IPC call succeeds, uses the response's `daemon_pid` (added in
   this phase if not present).
3. Reads `db.stats()` for `total` / `pinned`.
4. Reads `db.get_recent(1)` for the most recent item.
5. Builds the JSON object with `serde_json::json!()`.

If the IPC call fails, `running` is `false` and `daemon_pid` is `null`
but the rest of the payload is still populated from the local DB.

---

## Waybar module script

`contrib/waybar/clipboard.sh` is a small POSIX sh wrapper around
`author-clipboard-ctl status --json`. It does the minimum to be
Waybar-friendly:

- Calls `ctl status --json` (suppresses stderr; exit `0` always).
- Parses the JSON with `jq` (Waybar assumes `jq` is available on every
  Hyprland box — it is a hard dep of Waybar itself).
- Maps fields to Waybar's `text` / `tooltip` / `class` / `alt`.
- Emits a single JSON line on stdout.

`shellcheck` validates the script in CI via the new `waybar-check`
justfile recipe (`shellcheck` is `dl`-able; we don't make CI require it
unconditionally, but the local recipe works when the tool is installed).

---

## `SIGUSR1` / `SIGRTMIN+7` refresh path

Waybar's `signal` field re-runs `exec-on-event` when the bar process
receives the configured signal. The natural pattern is for the daemon
(or `author-clipboard-ctl`) to send `pkill -SIGRTMIN+7 waybar` when
something interesting happens (a new clipboard capture, a pin change,
etc.).

For Phase 19, we keep the refresh path purely client-driven: Waybar
itself polls every 30 s. The signal path is documented in the module's
README as a "future enhancement" and an `on-click` handler can manually
trigger a refresh via `pkill -SIGUSR1 waybar`.

This keeps the scope tight: no daemon signal-handler refactor is
required, and the module still works on Wayle / ags (which use the
same `interval` + `exec` model).

---

## AUR polish

The existing `packaging/arch/PKGBUILD` already includes `gtk4` and
`gtk4-layer-shell` in `makedepends` (added in commit `3e046b4`). For
Phase 19, we:

1. Verify `cargo build --release --workspace` is what's invoked (it is).
2. Add `optdepends` for `wofi` / `fuzzel` / `rofi` so users see the
   "external menu picker" hint at install time.
3. Regenerate `.SRCINFO` against the updated PKGBUILD.
4. Confirm `ci.yml → arch-pkg` runs `makepkg --printsrcinfo` and diffs
   against the committed `.SRCINFO` (it does).

We do **not** modify the AUR publishing flow: it stays a manual
maintainer step (see `docs/AUR.md`).

---

## Nix polish

The existing `flake.nix` already exposes all four binary packages
plus a dev shell. Phase 19 adds:

1. A `signal-refresh` example in the flake's `description` (textual
   only — no code change).
2. A `just nix-check` target that runs `nix flake check --no-build`
   (already exists; verify it's still wired).
3. A `nixBuild` target for the `hypr-picker` package specifically
   (useful for NixOS users who only want the picker without the applet).

`default.nix` already mirrors the flake; we verify the `version`
field matches `flake.nix`.

---

## `docs/HYPRLAND.md` Demo section

The Demo section is plain markdown. It contains:

1. **Reproducible shell transcript** — commands the user can paste into
   a fresh Hyprland session to see the picker in action.
2. **ASCII layout sketch** — a 12-line monospace drawing of the
   layer-shell popup.
3. **Maintainer note** — explains that the upstream maintainers will
   record a real GIF in a future release, and that the transcript is
   the canonical "what to expect" content.

---

## Testing strategy

| Layer | Command | Coverage |
|-------|---------|----------|
| Unit | `cargo test -p author-clipboard-shared` | DB stats, sensitive masking |
| Unit | `cargo test -p author-clipboard-ctl` | `Status` JSON shape |
| Lint | `cargo clippy -p author-clipboard-shared -p author-clipboard-ctl -p author-clipboard-hypr-picker -- -D warnings` | New code |
| Shell | `shellcheck contrib/waybar/clipboard.sh` (manual / `just waybar-check`) | POSIX sh compliance |
| JSON | `jq . <(author-clipboard-ctl status --json)` (manual) | Payload shape |
| PKGBUILD | `just aur-check` (CI `arch-pkg` job) | `.SRCINFO` parity |
| Nix | `just nix-check` (manual, requires Nix) | Flake metadata |

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
