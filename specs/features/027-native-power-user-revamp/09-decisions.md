# Decisions: Native Power-User Revamp

> Architectural and product decisions for this umbrella spec.

---

## D1: This Spec Supersedes `016-world-class-ux`

**Decision**: Treat `016-world-class-ux` as historical input, not the current
implementation plan.

**Reasoning**: It references older applet paths and does not cover storage, IPC,
CLI, install, compositor, or integration requirements.

## D2: Hypr Picker Defaults To XDG Window

**Decision**: `author-clipboard-hypr-picker` defaults to a normal resizable GTK
window. Layer-shell remains available through `--layer-shell`.

**Reasoning**: Power-user clipboard managers need close buttons, resize,
compositor rules, and predictable process lifecycle. Layer-shell is useful for
overlays, but it is a poor default for a richer command-center UI.

## D3: Do Not Fake Source-App Metadata

**Decision**: If Wayland/wlr-data-control does not expose source app/window
metadata, the UI must show unknown or rely on explicit integrations.

**Reasoning**: Incorrect provenance is worse than missing provenance for a
privacy/security-oriented clipboard manager.

## D4: UI Actions Require IPC/CLI Contracts

**Decision**: Pin/star/delete/collection/reveal/search actions must not be UI
only. They need daemon/DB and CLI/IPC surfaces.

**Reasoning**: This app targets power users and developers. Automation parity is
part of the product, not a later nice-to-have.

## D5: Sensitive Redaction Wins Over Polish

**Decision**: No visual polish may reveal raw sensitive content by default.

**Reasoning**: Clipboard managers routinely handle secrets. Trust and privacy
are non-negotiable.

## D6: Additive Migrations Only

**Decision**: Collections, saved filters, and metadata are additive DB changes.

**Reasoning**: Existing user history must survive the revamp.

## D7: Performance Claims Need Seeded Evidence

**Decision**: Do not claim virtualization or large-history performance until a
seeded benchmark exists and passes.

**Reasoning**: UI smoothness under real history size is a product requirement,
not a design statement.

## D8: Keep Visual System GTK/libadwaita Native

**Decision**: Use GTK4/libadwaita widgets and CSS tokens rather than a custom
theme engine.

**Reasoning**: The goal is first-party Linux native feel, not a web-app skin.

---

**Last Updated**: 2026-06-19
