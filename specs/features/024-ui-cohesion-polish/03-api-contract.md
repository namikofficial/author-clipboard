# API Contract: UI Cohesion & Dynamic Polish

## Public Surface

No new public daemon, CLI, or IPC contract is expected for this feature.
The following existing entry points remain stable:

- `ui_gtk::run_popup(PopupConfig)`
- `ui_gtk::run_manager(ManagerConfig)`
- `author-clipboard`
- `author-clipboard-hypr-picker`

## Internal UI Contract

The following internal surfaces may change as part of the polish pass:

- CSS classes and style tokens in `crates/ui-gtk/data/style.css`
- widget composition in `crates/ui-gtk/src/widgets/*`
- window layout and breakpoint behavior in `crates/ui-gtk/src/window/*`
- screenshot and smoke-test expectations in `crates/ui-gtk/tests/smoke.sh`

## Non-Goals

- No new IPC messages
- No new CLI flags
- No change to database schema
- No change to daemon behavior

