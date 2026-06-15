# Decisions: UI Cohesion & Dynamic Polish

## D1: No backend or IPC changes

This pass is presentation-only. If the UI needs a visual improvement, the
default path is CSS/layout/widget work, not protocol changes.

## D2: Keep keyboard semantics stable

Navigation, copy, delete, reveal, and escape behavior stay unchanged.
This pass may refine feedback, not meaning.

## D3: Prefer cohesive restraint over visual novelty

The target is polished and dynamic, not flashy. Motion and color should
support hierarchy, not compete with it.

