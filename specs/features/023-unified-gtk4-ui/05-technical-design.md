# Technical Design: Unified GTK4 UI

---

## Overview

Add `crates/ui-gtk/` as the single GTK4 + libadwaita UI library.
The two existing UI binaries (`applet`, `hypr-picker`) become thin
glue that calls into `ui-gtk`. The third (`ctl picker`) keeps its
external menu, but shares the new `shared::picker::PickerFilter`
enum so all three UIs speak the same filter language.

## Cargo Changes

### Root `Cargo.toml`

```toml
[workspace.dependencies]
gtk4 = { version = "0.9", features = ["v4_10"] }
libadwaita = { version = "0.7", features = ["v1_4"] }
gtk4-layer-shell = "0.4"
glib = "0.20"
gio = "0.20"
gdk-pixbuf = "0.20"
sourceview5 = "0.9"
webkit6 = "0.5"
glib-build-tools = "0.20"
```

### New `crates/ui-gtk/Cargo.toml`

```toml
[package]
name = "author-clipboard-ui-gtk"
version.workspace = true
edition.workspace = true
description.workspace = true

[lib]
name = "ui_gtk"
path = "src/lib.rs"

[dependencies]
author-clipboard-shared = { path = "../shared" }
libadwaita.workspace = true
gtk4.workspace = true
gtk4-layer-shell.workspace = true
glib.workspace = true
gio.workspace = true
gdk-pixbuf.workspace = true
sourceview5.workspace = true
webkit6.workspace = true
tokio.workspace = true
tracing.workspace = true
anyhow.workspace = true
thiserror.workspace = true
chrono.workspace = true
clap.workspace = true

[build-dependencies]
glib-build-tools.workspace = true

[lints]
workspace = true
```

### Slimmed `crates/applet/Cargo.toml`

```toml
[package]
name = "author-clipboard-applet"
version.workspace = true
edition.workspace = true
description.workspace = true

[[bin]]
name = "author-clipboard"
path = "src/main.rs"

[dependencies]
author-clipboard-ui-gtk = { path = "../ui-gtk" }
clap.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[lints]
workspace = true
```

### Slimmed `crates/hypr-picker/Cargo.toml`

```toml
[package]
name = "author-clipboard-hypr-picker"
version.workspace = true
edition.workspace = true
description.workspace = true

[[bin]]
name = "author-clipboard-hypr-picker"
path = "src/main.rs"

[dependencies]
author-clipboard-ui-gtk = { path = "../ui-gtk" }
clap.workspace = true
tokio.workspace = true

[lints]
workspace = true
```

## Affected Files

| File | Action |
|---|---|
| `Cargo.toml` | Add GTK deps to `[workspace.dependencies]` |
| `crates/ui-gtk/Cargo.toml` | NEW |
| `crates/ui-gtk/build.rs` | NEW — glib-build-tools for GResource |
| `crates/ui-gtk/src/lib.rs` | NEW — public API |
| `crates/ui-gtk/src/app.rs` | NEW — AppState, Action, reduce |
| `crates/ui-gtk/src/model.rs` | NEW — GObject models |
| `crates/ui-gtk/src/actions.rs` | NEW — GAction |
| `crates/ui-gtk/src/controller/focus.rs` | NEW |
| `crates/ui-gtk/src/controller/key.rs` | NEW |
| `crates/ui-gtk/src/controller/search.rs` | NEW |
| `crates/ui-gtk/src/window/popup.rs` | NEW |
| `crates/ui-gtk/src/window/manager.rs` | NEW |
| `crates/ui-gtk/src/widgets/search.rs` | NEW |
| `crates/ui-gtk/src/widgets/filter_bar.rs` | NEW |
| `crates/ui-gtk/src/widgets/item_row.rs` | NEW |
| `crates/ui-gtk/src/widgets/picker_grid.rs` | NEW |
| `crates/ui-gtk/src/widgets/preview.rs` | NEW |
| `crates/ui-gtk/src/widgets/empty.rs` | NEW |
| `crates/ui-gtk/src/widgets/chip.rs` | NEW |
| `crates/ui-gtk/src/widgets/toast.rs` | NEW |
| `crates/ui-gtk/src/widgets/shortcuts_overlay.rs` | NEW |
| `crates/ui-gtk/src/pages/clipboard.rs` | NEW |
| `crates/ui-gtk/src/pages/emoji.rs` | NEW |
| `crates/ui-gtk/src/pages/symbols.rs` | NEW |
| `crates/ui-gtk/src/pages/kaomoji.rs` | NEW |
| `crates/ui-gtk/src/pages/snippets.rs` | NEW |
| `crates/ui-gtk/src/pages/settings.rs` | NEW |
| `crates/ui-gtk/src/theme.rs` | NEW |
| `crates/ui-gtk/src/settings.rs` | NEW — GSettings |
| `crates/ui-gtk/assets/style.css` | NEW |
| `crates/ui-gtk/assets/icons/*.svg` | NEW (22 SVGs) |
| `crates/ui-gtk/data/com.namikofficial.author-clipboard.gschema.xml` | NEW |
| `crates/ui-gtk/data/resources.gresource.xml` | NEW |
| `crates/applet/Cargo.toml` | rewrite to depend on ui-gtk |
| `crates/applet/src/main.rs` | shrink to ~80 LOC |
| `crates/hypr-picker/Cargo.toml` | rewrite to depend on ui-gtk |
| `crates/hypr-picker/src/main.rs` | shrink to ~40 LOC |
| `crates/shared/src/picker.rs` | add `PickerFilter`, update `filter_entries` |
| `crates/ctl/src/main.rs` | add `--filter` flag to picker subcommand |
| `packaging/arch/PKGBUILD` | add `glib2-devel` makedep |
| `packaging/debian/control` | add `libglib2.0-dev-bin` makedep |
| `flake.nix` | add `gtk4`, `glib` to build inputs |
| `docs/UI.md` | NEW — design tokens, widget catalog |
| `justfile` | add `just ui-check` (glib-compile-schemas) |

## Bug-Fix Implementation Detail

### Esc semantics (US-001)

`controller/focus.rs`:
```rust
pub fn install_esc_handler(window: &adw::ApplicationWindow) {
    let controller = gtk::EventControllerKey::new();
    let win = window.clone();
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            handle_escape(&win);
            return glib::Propagation::Stop; // we always win
        }
        glib::Propagation::Proceed
    });
    window.add_controller(controller);
}

fn handle_escape(window: &Window) {
    let state = window.app_state();
    if state.search_has_focus() && !state.search_is_empty() {
        state.clear_search();
        state.focus_list();
    } else if state.search_has_focus() {
        // search has focus but is empty → leave search, keep window
        state.focus_list();
    } else {
        window.close();
    }
}
```

The `Capture` phase is the key: it fires before the `text_input`'s
built-in Esc handler, so we always get a chance to act. We then
explicitly `Stop` propagation so the widget doesn't see it.

### Search focus (US-002)

`window/popup.rs`:
```rust
fn build_popup(app: &App) -> adw::Window {
    let win = AdwWindow::new(app);
    // ... layout ...
    win.set_default_widget(Some(&list));   // Enter on list = copy
    win.set_focus_widget(Some(&list));     // open with list focused
    win.present();
    win
}
```

The `/` key is captured globally by `controller/key.rs`:
```rust
Key::Slash => {
    if !state.search_has_focus() {
        state.focus_search();
        Propagation::Stop
    } else { Propagation::Proceed }
}
```

### Manager window (US-003)

`window/manager.rs`:
```rust
pub fn build_manager(app: &App) -> adw::ApplicationWindow {
    let win = AdwApplicationWindow::builder()
        .application(app)
        .title("Clipboard Manager")
        .default_width(1100)
        .default_height(720)
        .build();
    let nav = AdwNavigationView::new();
    nav.push(AdwNavigationPage::new(&build_main_page(), "clipboard"));
    win.set_content(Some(&nav));
    win.present();
    win
}
```

The previous `cosmic::app::run` is gone. The manager is a real
`AdwApplicationWindow` with titlebar, sidebar, status page, and
`AdwNavigationView` for page switching.

## Design Tokens (CSS)

```css
:root {
  --accent:       @accent_bg_color;
  --accent-fg:    @accent_fg_color;
  --surface-0:    @window_bg_color;
  --surface-1:    @card_bg_color;
  --surface-2:    @view_bg_color;
  --text-0:       @window_fg_color;
  --text-1:       @view_fg_color;
  --text-2:       @dim_label_fg_color;
  --border:       @borders_color;
  --danger:       @error_bg_color;
  --success:      @success_bg_color;

  --radius-sm:    6px;
  --radius-md:    12px;
  --radius-lg:    16px;
  --radius-pill:  999px;

  --shadow-sm:    0 1px 2px rgba(0,0,0,0.06);
  --shadow-md:    0 4px 12px rgba(0,0,0,0.10);

  --motion-fast:  120ms;
  --motion-base:  200ms;
  --motion-slow:  320ms;
  --ease-out:     cubic-bezier(0.16, 1, 0.3, 1);
  --ease-spring:  cubic-bezier(0.34, 1.56, 0.64, 1);
}

* { transition:
    background-color var(--motion-fast) var(--ease-out),
    border-color    var(--motion-fast) var(--ease-out),
    box-shadow      var(--motion-base) var(--ease-out);
}

.item-row {
  border-radius: var(--radius-md);
  padding: 10px 14px;
  margin: 2px 0;
}
.item-row:hover   { background: alpha(@accent_bg_color, 0.08); }
.item-row.selected {
  background: alpha(@accent_bg_color, 0.18);
  box-shadow: var(--shadow-sm);
}
.item-row.sensitive { border-left: 3px solid var(--danger); }

.chip {
  border-radius: var(--radius-pill);
  padding: 2px 10px;
  font-size: 11px;
  font-weight: 500;
  background: alpha(@accent_bg_color, 0.12);
  color: @accent_fg_color;
}

.search-entry {
  border-radius: var(--radius-pill);
  padding: 4px 14px;
  background: @view_bg_color;
}

button.suggested-action {
  border-radius: var(--radius-md);
  font-weight: 600;
}
```

## Custom Icon Set

22 symbolic SVGs at 16/24/32 px, designed in a 24×24 grid with 2px
stroke, rounded line caps, and the same visual weight. Sources of
inspiration: GNOME symbolic icons, but with slightly more rounded
geometry to feel "cute". Each SVG is < 1KB.

Files (in `crates/ui-gtk/assets/icons/`):
```
clipboard.svg pin.svg star.svg lock.svg search.svg trash.svg
image.svg code.svg files.svg link.svg emoji.svg kaomoji.svg
symbol.svg snippet.svg gear.svg chevron-down.svg x.svg plus.svg
copy.svg empty-clipboard.svg empty-search.svg empty-warning.svg
```

## IPC Integration

The UI calls the same `IpcClient` from `shared::ipc` for every
mutation. The only addition is `ui_gtk::Ipc::with_toast(...)` —
a helper that wraps an IPC call and emits an `AdwToast` on success
or failure. Used everywhere we currently `std::process::exit(0)`,
so we can actually show a "Copied!" toast before the popup closes
(closes itself after 800ms instead of immediately, which feels
smoother).

```rust
impl Ipc {
    pub async fn copy(&self, id: i64, mode: CopyMode, overlay: &ToastOverlay) -> Result<()> {
        self.send(&IpcCommand::Copy { id, mode }).await?;
        overlay.show_toast("Copied to clipboard");
        Ok(())
    }
}
```

## Performance

- `gio::ListStore` + `gtk::SingleSelection` for the list. GTK
  recycles row widgets automatically.
- Thumbnails are loaded from `image_store::thumbnail_path` once
  via `gdk_pixbuf::Pixbuf::from_file_at_scale` and cached in
  the `ItemObject` GObject.
- Search debounce: 150ms via `glib::timeout_add_local` (already
  used in hypr-picker; copy the pattern).
- Preview pane in the manager uses `sourceview5::View` for
  text, scaled `gdk_pixbuf::Pixbuf` for images (with
  `gtk::Picture::set_keep_aspect_ratio`), and `webkit6` for HTML
  (sandboxed via `WebContext`).

## GResource Build

`crates/ui-gtk/build.rs`:
```rust
fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "compiled.gresource",
    );
}
```

`data/resources.gresource.xml`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<gresources>
  <gresource prefix="/com/namikofficial/author-clipboard">
    <file>style.css</file>
    <file alias="icons/clipboard.svg">icons/clipboard.svg</file>
    <file>app.ui</file>
    <file>manager.ui</file>
  </gresource>
</gresources>
```

`GSettings` schemas are compiled by `just ui-check` →
`glib-compile-schemas crates/ui-gtk/data/`.

## Security Considerations

- [x] Sensitive data handled correctly: `redacted_preview` is the
      default; reveal is an explicit, time-boxed user action.
- [x] No data exposure in logs: `tracing` calls in `ui-gtk` only
      emit item IDs and types, never content.
- [x] Input validation on all boundaries: `PickerFilter`,
      `PickerSource`, `CopyMode` are exhaustive enums parsed via
      `serde`; invalid values produce a clear `INVALID_ARG` error.
- [x] IPC permissions checked: `IpcClient` honors existing
      `XDG_RUNTIME_DIR` 0700 directory and 0600 socket checks.

## Error Handling

| Error Condition | Handling Strategy |
|---|---|
| Daemon not running | Toast "Daemon offline" + degrade to read-only DB; never crash |
| IPC timeout (>2s) | Cancel, show toast "Daemon is slow — retry?" |
| Invalid `--filter` | CLI prints `INVALID_ARG` and exits 2 with `--help` hint |
| Layer-shell unavailable | Fall back to XDG window with headerbar |
| GSettings unavailable | Use in-memory defaults |
| GResource load failure | Hard fail at startup; CI catches via build.rs |
| Invalid content (corrupt image, broken UTF-8) | Render with `???` chip, log warn |

## Performance Considerations

- List uses `gio::ListStore` + `gtk::SingleSelection` (recycled rows).
- Search debounce avoids filter storms while typing.
- Thumbnails are scaled to 80×60 at `ItemObject` construction.
- IPC `History` call is rate-limited to one per 250ms via a
  `tokio::time::interval` guard.

## Migration Strategy

No DB migration is required. The `PickerFilter` enum is purely
additive to `shared::picker`. The old `ContentFilter` in
`hypr-picker` is deleted and replaced.

## Testing Strategy

See `07-test-plan.md` for detailed test cases. Short version:
unit tests for `reduce()` and `filter_entries`, golden tests for
each widget (rendered to a `cairo` surface and diffed), and a
shell-based smoke test that drives the live UI with `xdotool`
under `xvfb-run`.

---

**Last Updated**: 2026-06-12
