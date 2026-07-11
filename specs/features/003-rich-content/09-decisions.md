# Decisions: Rich Content Completion

## D-001: Keep HTML preview inert by default

The default build displays HTML source in SourceView. This preserves content
without executing scripts, loading remote resources, or expanding the default
dependency surface. Sandboxed WebKit rendering remains feature-gated.

## D-002: Preserve the original file URI for activation

The UI parses URI lists only to produce decoded names and filesystem metadata.
It passes the original non-comment URI to the default application launcher so
encoded paths and non-file URI schemes are not reconstructed incorrectly.

## Deviations

None. Implementation matches `05-technical-design.md`.
