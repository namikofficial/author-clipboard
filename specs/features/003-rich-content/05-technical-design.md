# Technical Design: Rich Content Completion

## Existing Pipeline

The daemon already prioritizes supported `image/*`, `text/html`, and
`text/uri-list` offers, stores typed `ClipboardItem` values, generates image
thumbnails, indexes HTML plain text, and restores the original MIME through IPC.
The shared `file_handler` already parses URI lists and resolves file metadata.

## Completion Design

Keep capture, database, and IPC contracts unchanged. Complete the GTK rendering
path by:

1. Resolving stored image filenames through `Config::data_dir/images` while
   retaining support for absolute paths used by imported/legacy records.
2. Clearing the `gtk::Picture` before each load so a failed load cannot display
   the previously selected image.
3. Mapping HTML picker entries with their display/search title as the plain-text
   fallback, never the MIME value.
4. Building file preview rows from `file_handler::parse_uri_list`, displaying
   decoded names and useful metadata while retaining the original URI for the
   open action.
5. Resetting all preview visibility consistently before showing a new state.

Sensitive-item behavior remains owned by the existing redaction overlay.
Default HTML preview remains inert source text. The optional sandboxed WebKit
feature remains an opt-in enhancement.

## Affected Files

- `specs/features/003-rich-content/01-requirements.md`
- `specs/features/003-rich-content/05-technical-design.md`
- `specs/features/003-rich-content/06-task-plan.md`
- `specs/features/003-rich-content/07-test-plan.md`
- `specs/features/003-rich-content/08-review-checklist.md`
- `crates/ui-gtk/src/pages/clipboard.rs`
- `crates/ui-gtk/src/widgets/preview.rs`

No schema, daemon protocol, or dependency changes are required.

## Implementation Notes

`PreviewPane` uses two small pure helpers so path resolution and file metadata
labels are testable without a running GTK display. File activation remains a UI
boundary: parsed metadata drives the label, while the unmodified source URI is
passed to `gio::AppInfo` so percent encoding and URI semantics are preserved.
