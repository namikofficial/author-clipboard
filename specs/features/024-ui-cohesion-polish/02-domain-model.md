# Domain Model: UI Cohesion & Dynamic Polish

## Concepts

### Surface
One of:
- `Popup`
- `Manager`
- `NativePicker`

Each surface shares the same theme tokens but has its own layout density
and hierarchy.

### Visual Token Set
Shared design values that define the shell:
- spacing scale
- corner radius scale
- shadow depth
- border contrast
- motion duration
- focus ring treatment
- icon sizing

### State Layers
The UI presents a small set of interaction layers:
- idle
- hover
- focus
- selected
- empty
- loading
- redacted
- toast

These are presentation-only states. They do not change persisted data.

### Breakpoints
Layout switches by width:
- compact popup
- medium manager
- wide manager with sidebar and preview separation

## Constraints

- No persisted schema changes
- No IPC changes
- No change to selection semantics
- No change to clipboard storage or retrieval semantics

