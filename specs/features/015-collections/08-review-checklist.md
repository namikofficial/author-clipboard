# Review Checklist: Collections UI

## Automated Evidence — 2026-07-12

- [x] Collection create, list, rename, delete, membership, ordering, counts,
  cascade cleanup, and multi-membership storage behavior have database tests.
- [x] GTK collection models retain stable IDs and item counts.
- [x] Collection names are trimmed and blank names are rejected.
- [x] The manager provides create, rename, delete, browse, add, and remove UI.
- [x] The clipboard page opens an IPC-backed collection chooser for the selected item.
- [x] UI tests pass headlessly (95 passed; 14 display-dependent preview tests ignored).

## Unverified or Incomplete Acceptance Criteria

- [ ] Ctrl+P pin toggle is manually verified in the integrated picker.
- [ ] Ctrl+Shift+S star toggle and starred ranking are manually verified.
- [ ] Ctrl+Shift+P/A quick filters are implemented and manually verified.
- [ ] Collection UI behavior with 50+, 1,000 items, and 100 collections is measured.
- [ ] Virtual scrolling/pagination for large collection contents is demonstrated.
- [ ] Collection lifecycle and multi-membership are manually smoked on Wayland.
- [x] Integrated `just verify` passes in the final review working tree (2026-07-12).
