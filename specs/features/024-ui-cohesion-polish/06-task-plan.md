# Task Plan: UI Cohesion & Dynamic Polish

> Small, reviewable UI-only slices. Each task should end with a
> verification command and a screenshot or manual check where relevant.

## Task Graph

```
T001 visual audit
   ├─ T002 token + theme refinement
   ├─ T003 popup hierarchy polish
   ├─ T004 manager layout polish
   ├─ T005 widget state + motion polish
   └─ T006 docs + screenshots + sign-off
```

## T001 · Visual audit

**Goal**: Capture the current UI and identify the biggest cohesion gaps.

**Files**: `docs/UI/snapshots/`, `docs/UI.md`, `README.md`

**Verification**:
```bash
just ui-smoke
```

## T002 · Token and theme refinement

**Goal**: Tighten the shared visual language across popup and manager.

**Files**: `crates/ui-gtk/data/style.css`, `crates/ui-gtk/src/theme.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk
```

## T003 · Popup hierarchy polish

**Goal**: Make search, filter, list, and empty states feel polished and balanced.

**Files**: `crates/ui-gtk/src/window/popup.rs`, `crates/ui-gtk/src/pages/clipboard.rs`, `crates/ui-gtk/src/widgets/{search,filter_bar,item_row,empty}.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- search filter_bar item_row
```

## T004 · Manager layout polish

**Goal**: Refine the manager shell, sidebar, preview proportions, and breakpoint behavior.

**Files**: `crates/ui-gtk/src/window/manager.rs`, `crates/ui-gtk/src/widgets/preview.rs`

**Verification**:
```bash
cargo test -p author-clipboard-ui-gtk -- preview
```

## T005 · Motion and state feedback

**Goal**: Add cohesive hover, focus, selection, redaction, and toast feedback.

**Files**: `crates/ui-gtk/src/widgets/{item_row,preview,toast,shortcuts_overlay}.rs`, `crates/ui-gtk/data/style.css`

**Verification**:
```bash
just verify
```

## T006 · Docs and screenshots

**Goal**: Update docs and captures to match the refined UI.

**Files**: `docs/UI.md`, `README.md`, `docs/UI/snapshots/`

**Verification**:
```bash
just ui-smoke
```
