# Technical Design: UI Cohesion & Dynamic Polish

## Goal

Improve the perceived quality of the current GTK4 UI by tightening the
visual system and interaction feedback, while preserving all existing
behavior.

## Affected Files

| File | Expected Change |
|------|-----------------|
| `crates/ui-gtk/data/style.css` | Refine token scale, spacing, radii, shadows, transitions |
| `crates/ui-gtk/src/theme.rs` | Align runtime theme behavior with the token set |
| `crates/ui-gtk/src/widgets/item_row.rs` | Refine row density, selection, hover, metadata layout |
| `crates/ui-gtk/src/widgets/filter_bar.rs` | Make chip styling and active states more cohesive |
| `crates/ui-gtk/src/widgets/search.rs` | Tighten search field styling and focus states |
| `crates/ui-gtk/src/widgets/preview.rs` | Improve preview chrome, redaction, and empty-state presentation |
| `crates/ui-gtk/src/widgets/empty.rs` | Standardize empty-state composition |
| `crates/ui-gtk/src/window/popup.rs` | Tune header spacing and shell balance |
| `crates/ui-gtk/src/window/manager.rs` | Improve responsive layout, sidebar, and preview proportions |
| `crates/ui-gtk/tests/smoke.sh` | Capture before/after screenshots for review |
| `docs/UI.md` | Document the refined visual language |
| `README.md` | Refresh screenshots if the visual change is significant |

## Design Approach

- Use one spacing and radius scale across all widgets
- Keep motion short and purposeful, not decorative
- Make the preview pane feel like a secondary reading surface, not a
  separate app
- Use stronger contrast for primary actions and selection
- Keep the interface calm; avoid novelty for its own sake

## Implementation Notes

- Prefer CSS and layout tuning before introducing new widgets
- Reuse existing shell structure where possible
- Do not add backend state for purely visual concerns
- Preserve keyboard shortcuts and focus behavior

