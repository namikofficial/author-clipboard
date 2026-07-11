# Task Plan: Collections UI

## T021-01: Specify native collection management

**Goal**: Define the real-data GTK flow and independently verifiable behavior.

**Verification**: `test -f specs/features/015-collections/05-technical-design.md`

## T021-02: Add collection page view model and manager UI

**Goal**: List, create, rename, delete and browse collections with count badges.

**Verification**: `cargo test -p author-clipboard-ui-gtk -- collections`

## T021-03: Add membership removal and navigation integration

**Goal**: Add items through the Ctrl+Shift+C chooser, remove memberships without
deleting history, confirm collection deletion, and make the page reachable and
persistent through manager navigation.

**Verification**: `cargo test -p author-clipboard-ui-gtk`
