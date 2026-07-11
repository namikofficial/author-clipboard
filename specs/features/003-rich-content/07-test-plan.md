# Test Plan: Rich Content Completion

- Unit-test HTML conversion preserves raw HTML, `text/html`, and plain fallback.
- Unit-test image path resolution for relative and absolute stored paths.
- Unit-test file preview metadata formatting for existing and missing files.
- Run the existing shared crate suite covering image storage, URI parsing,
  database persistence/search, and typed clipboard restoration.
- Compile and test `ui-gtk`; GTK display-dependent widget tests remain ignored in
  headless CI, while pure rendering helpers run normally.
