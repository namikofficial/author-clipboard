# Decisions: Hyprland-native UX & wlroots Polish

> Architectural decisions and deviations for Phase 19 polish.

---

## D-001: Waybar refresh is client-driven, not daemon-pushed

**Context**: Waybar's `signal` mechanism re-runs the `exec` chain when the
bar receives a configured signal. The natural pattern is to have the
daemon `pkill -SIGRTMIN+7 waybar` whenever something interesting
happens (new capture, pin change, etc.).

**Decision**: Keep the refresh path purely client-driven. Waybar polls
every 30 s. The module's `on-click` triggers a manual refresh via
`pkill -SIGUSR1 waybar`. No daemon signal handler is added.

**Why**:
- Smaller surface area, fewer moving parts in the daemon.
- Works identically on Waybar / Wayle / ags (all of which support
  `interval` + `exec`).
- 30 s polling is fine for a status indicator — clipboard history is
  a low-frequency event.
- If we later need real-time updates, we can add a `notify` channel
  without breaking the existing module.

**Trade-off**: Up to 30 s of staleness in the indicator. Acceptable for
a clipboard status display.

## D-002: `status --json` reads DB even when daemon is down

**Context**: The user might want to know "is my history still there?"
even when the daemon isn't running. The Waybar module should also
render gracefully in this state.

**Decision**: `status --json` always reads the local SQLite database
for `total` / `pinned` / `last_*`. The `running` field reports the
daemon's state via `IpcClient::send_command(Ping)`.

**Why**:
- Graceful degradation — the module still shows useful info.
- SQLite is read-only safe in this path; no daemon lock contention.

**Trade-off**: Two code paths in the helper. Documented in the
test plan.

## D-003: AUR package publishing is a manual maintainer step

**Context**: AUR submissions require a maintainer account, an SSH key,
and a `git push` to the AUR remote. None of those can be automated from
CI.

**Decision**: Keep the AUR publishing flow manual. CI validates that
the in-tree PKGBUILD and `.SRCINFO` are in sync and builds the package
in an Arch container. The release workflow uploads
`aur/PKGBUILD`, `aur/.SRCINFO`, and `aur/aur-files.tar.gz` to the
GitHub Release so a human maintainer has everything in one place.

**Why**:
- Matches the AUR's contribution model (trusted users / TU push).
- No secrets need to live in the CI runner.
- The PKGBUILD is the source of truth; the AUR is a mirror.

**Trade-off**: AUR releases lag the GitHub release by however long it
takes a maintainer to push.

## D-004: Nix flake SHA256 is left as a placeholder

**Context**: The first `nix build` of a `fetchFromGitHub`-sourced
derivation requires a real SHA256 hash. Computing it requires a Nix
machine with network access, which CI doesn't have.

**Decision**: The `sha256` in `flake.nix` and `default.nix` is
`"0000…0000"` (placeholder). The `nix flake check --no-build` step
validates metadata but not the source hash. The first real build
either uses `nix-prefetch-url` locally or
`nix flake update --override-input`.

**Why**:
- Keeps the flake in the tree so users can `nix flake check` /
  `nix develop` without first computing a hash.
- `nix build` will fail with a clear "hash mismatch" error pointing
  the user to the fix, which is the standard Nix workflow.

**Trade-off**: Out-of-the-box `nix build` does not work; users must
run `nix-prefetch-url` once. Documented in `docs/PACKAGING.md`.

## D-005: Demo is a text transcript + ASCII layout, not a GIF

**Context**: The PROJECT_PLAN calls for a "Demo GIF for Hyprland." The
build environment is headless and cannot capture screen content.

**Decision**: The `## Demo` section in `docs/HYPRLAND.md` provides a
reproducible shell transcript and an ASCII layout sketch. Maintainers
attaching a real GIF to a future release is the long-term plan; the
transcript is the canonical fallback for now.

**Why**:
- A transcript is auditable, reproducible, and works in any environment.
- An ASCII layout sketch conveys the spatial structure of the layer-shell
  popup without a renderer.
- The "Demo GIF" deliverable is satisfied as a documentation
  placeholder — explicit and honest.

**Trade-off**: Less immediately compelling than a real GIF. Tradeoff is
documented in `PROJECT_PLAN.md` as a known limitation.

---

**Last Updated**: 2026-06-08 (Phase 19 polish)
