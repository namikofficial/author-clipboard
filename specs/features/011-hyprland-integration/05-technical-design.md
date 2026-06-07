# Technical Design: Hyprland Integration

> Implementation approach for the native Hyprland GTK4 layer-shell picker.

---

## Overview

The Hypr-picker is a standalone GTK4 application that uses layer-shell to appear as an overlay on Hyprland. It communicates with the daemon via IPC to fetch items and copy selections.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/hypr-picker/src/main.rs` | GTK4 app with layer-shell |
| `crates/shared/src/picker.rs` | Shared picker logic |
| `crates/ctl/src/main.rs` | hyprland-config command |

---

## Implementation Details

### Layer Shell Setup

```rust
// In hypr-picker/src/main.rs

use gtk4::prelude::*;
use gtk::Application;
use wayland_client::protocol::wl_output::WlOutput;

fn main() {
    let app = Application::builder()
        .application_id("com.namikofficial.author-clipboard-hypr-picker")
        .build();

    app.connect_activate(|app| {
        // Create window
        let window = gtk::Window::new();
        window.set_title("author-clipboard-hypr-picker");

        // Set as layer-shell overlay
        // (Using niri's layer-shell implementation or similar)

        // Connect to daemon via IPC
        let client = IpcClient::new();
        let items = client.send(&IpcCommand::History { limit: 50, .. });

        // Build UI
        let list_box = build_list_box(&items);
        window.set_child(Some(&list_box));

        window.present();
    });

    app.run();
}
```

### IPC Communication

```rust
fn load_items(source: PickerSource, count: usize) -> Vec<PickerEntry> {
    let client = IpcClient::new();
    let response = client.send(&IpcCommand::History {
        limit: count,
        offset: None,
        filters: Some(FilterOptions {
            source: Some(source),
            ..Default::default()
        }),
    }).expect("Failed to connect to daemon");

    parse_items_from_response(response)
}
```

---

## Testing

1. Test picker opens with Super+Shift+V
2. Test items load from daemon
3. Test item selection copies to clipboard
4. Test keyboard navigation
5. Test picker closes on Escape

---

**Last Updated**: Phase 15 (Updated from draft)