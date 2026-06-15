# Test Plan: UI Cohesion & Dynamic Polish

## Automated

- `just verify`
- `cargo test -p author-clipboard-ui-gtk -- search filter_bar item_row preview theme`
- `just ui-smoke`

## Manual

- Popup and manager share one visual language
- Dark and light themes look coherent
- Selection, hover, and focus feedback are legible
- Empty states feel intentional
- Responsive manager layout still feels balanced at narrow and wide widths
- No keyboard shortcut regressions

