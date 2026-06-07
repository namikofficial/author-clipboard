# Requirements: Rich Content Support

---

## User Stories

### US-001: Image Clipboard
**As a** user
**I want to** copy an image and see it in my history with a thumbnail
**So that** I can select and paste it later

**Acceptance Criteria**:
- Given I copy an image, when I open the picker, then I see a thumbnail of the image
- Given I select an image, when I paste, then the image is restored correctly

### US-002: HTML Clipboard
**As a** user
**I want to** copy formatted text and have the HTML preserved
**So that** when I paste, the formatting is retained

**Acceptance Criteria**:
- Given I copy HTML content, when I search for text, then the content is findable
- Given I select an HTML item, when I paste in a rich text editor, then formatting is preserved

### US-003: File URI List
**As a** user
**I want to** copy file selections and see them in history
**So that** I can re-copy files without navigating to them

**Acceptance Criteria**:
- Given I copy files in a file manager, when I open the picker, then I see file names and icons
- Given I select a file item, when I paste, then the file URI is restored

---

## Content Type Handling

| Type | Capture | Store | Restore |
|------|---------|-------|---------|
| Text | `text/plain` | Plain text | `wl-copy --type text/plain` |
| HTML | `text/html` | HTML + plain text | `wl-copy --type text/html` |
| Image | `image/*` | File in `images/` + thumbnail | `wl-copy --type <mime>` |
| Files | `text/uri-list` | Parsed file metadata | `wl-copy --type text/uri-list` |

---

## Out of Scope

- OCR for images (Phase 15)
- Drag-and-drop clipboard (Phase 16)
- Multi-format paste selection (Phase 16)

---

**Last Updated**: Phase 3 Complete