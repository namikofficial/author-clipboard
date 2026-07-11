# Review Checklist: Rich Content Completion

- [x] Image filenames resolve under the configured data directory.
- [x] Failed image loads cannot retain a stale preview.
- [x] HTML compact previews use plain text, not MIME labels.
- [x] Default HTML preview does not execute active content.
- [x] URI-list comments are ignored and names are decoded.
- [x] File rows show existence, MIME, and size information.
- [x] Original MIME types remain intact for copy/restore.
- [x] Sensitive rich content remains redacted by default.
- [x] Targeted tests pass (6 headless tests; 14 display-dependent tests remain ignored).
- [x] Rich-content targeted tests pass.
- [ ] Display-dependent preview tests are run under a supported Wayland compositor.
- [x] Integrated `just verify` passes in the final review working tree (2026-07-12).
