# Domain Model: Unified GTK4 UI

---

## New Crate Layout

```
crates/
├── ui-gtk/                    # NEW — the one UI library
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/
│   │   ├── lib.rs             # public API: run_popup, run_manager
│   │   ├── app.rs             # AppState, Reducer, Action
│   │   ├── model.rs           # GObject models: ClipboardItemObject, PickerEntryObject
│   │   ├── actions.rs         # GAction registrations
│   │   ├── controller/
│   │   │   ├── mod.rs
│   │   │   ├── focus.rs       # FocusChain + Esc handler
│   │   │   ├── key.rs         # global key controller
│   │   │   └── search.rs      # debounced search
│   │   ├── window/
│   │   │   ├── mod.rs
│   │   │   ├── popup.rs       # AdwWindow + layer-shell init
│   │   │   └── manager.rs     # AdwApplicationWindow + NavigationView
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── search.rs      # SearchEntry with `/` focus + Esc
│   │   │   ├── filter_bar.rs  # All/Text/Images/Files/Pinned/Sensitive
│   │   │   ├── item_row.rs    # one row, all content types
│   │   │   ├── picker_grid.rs # emoji/symbol/kaomoji grid
│   │   │   ├── preview.rs     # right-pane preview (manager only)
│   │   │   ├── empty.rs       # AdwStatusPage variants
│   │   │   ├── chip.rs        # source-app / age / type chip
│   │   │   ├── toast.rs       # AdwToast wrapper
│   │   │   └── shortcuts_overlay.rs
│   │   ├── pages/
│   │   │   ├── mod.rs
│   │   │   ├── clipboard.rs
│   │   │   ├── emoji.rs
│   │   │   ├── symbols.rs
│   │   │   ├── kaomoji.rs
│   │   │   ├── snippets.rs
│   │   │   └── settings.rs    # AdwPreferencesWindow content
│   │   ├── theme.rs           # design tokens, AdwStyleManager
│   │   └── settings.rs        # GSettings schema + bindings
│   ├── assets/
│   │   ├── style.css
│   │   └── icons/*.svg
│   └── data/
│       ├── resources.gresource.xml
│       ├── app.ui
│       ├── manager.ui
│       └── com.namikofficial.author-clipboard.gschema.xml
├── applet/                    # SLIMMED — ~80 LOC
│   ├── Cargo.toml             # depends on ui-gtk
│   └── src/main.rs            # cli → ui_gtk::run_popup|run_manager
├── hypr-picker/               # SLIMMED — ~40 LOC
│   ├── Cargo.toml             # depends on ui-gtk
│   └── src/main.rs            # cli → ui_gtk::run_popup
├── shared/                    # UNCHANGED except picker.rs gets new enum
│   └── src/picker.rs          # adds PickerFilter enum
├── ctl/                       # UNCHANGED except uses new PickerFilter
├── clipboard-daemon/          # UNCHANGED
└── mcp-server/                # UNCHANGED
```

## State Machine

```rust
#[derive(Debug, Clone, Default, glib::Properties)]
pub struct AppState {
    #[property(get, set)]
    pub active_page: PageId,                 // Clipboard | Emoji | ... | Settings
    #[property(get, set)]
    pub filter: PickerFilter,                // All | Text | ... | Sensitive
    #[property(get, set)]
    pub search_query: String,
    #[property(get, set)]
    pub selected_index: Option<u32>,
    #[property(get, set)]
    pub sort: SortOrder,                     // NewestFirst | OldestFirst
    #[property(get, set)]
    pub show_redacted: bool,                 // sensitive reveal toggle
    #[property(get, set)]
    pub daemon_running: bool,
    #[property(get, set)]
    pub incognito: bool,
    pub items: gio::ListStore,               // of ClipboardItemObject
    pub snippets: gio::ListStore,            // of SnippetObject
    pub model: gtk::SingleSelection,         // bound to items
    pub focus: FocusTarget,                  // List | Search | Modal
}

pub enum PageId {
    Clipboard,
    Emoji,
    Symbols,
    Kaomoji,
    Snippets,
    Settings,
}

pub enum SortOrder {
    NewestFirst,
    OldestFirst,
    MostUsed,
}

pub enum FocusTarget {
    List,
    Search,
    Modal,
}

pub enum Action {
    Select(Option<u32>),
    MoveBy(i32),
    MoveTo(usize),                           // Home / End
    MovePage(i32),                           // PgUp / PgDn
    SetSearch(String),
    ClearSearch,
    SetFilter(PickerFilter),
    SetPage(PageId),
    CyclePage(i32),                          // Ctrl+Tab
    Focus(FocusTarget),
    Copy,
    QuickPaste,
    TogglePin(i64),
    ToggleStar(i64),
    Delete(i64),
    RevealRedacted,                          // Ctrl+Shift+R
    HideRedacted,
    Toast(String),
    Quit,
}

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect>;

pub enum Effect {
    CopyItem(i64, CopyMode),
    QuickPasteItem(i64),
    PinItem(i64),
    UnpinItem(i64),
    StarItem(i64),
    UnstarItem(i64),
    DeleteItem(i64),
    ClearUnpinned,
    AddToast(String),
    SaveConfig,
    Quit,
}
```

`Effect` is a non-pure side effect that the runtime executes after `reduce()`.

## Filter Enum (shared between popup, manager, external picker)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerFilter {
    All,
    Text,
    Images,
    Files,
    Pinned,
    Starred,
    Sensitive,
}
```

The old `ContentFilter` enum in `hypr-picker/src/main.rs` is replaced
by this. `shared::picker::filter_entries` is updated to take
`PickerFilter` and gain parity with the new filter bar.

## GSettings Schema

```xml
<schema id="com.namikofficial.author-clipboard.state" path="/com/namikofficial/author-clipboard/state/">
  <key name="filter" enum="acb-filter">
    <default>"all"</default>
  </key>
  <key name="sort" enum="acb-sort">
    <default>"newest"</default>
  </key>
  <key name="last-page" enum="acb-page">
    <default>"clipboard"</default>
  </key>
  <key name="window-width" type="i">
    <default>1100</default>
  </key>
  <key name="window-height" type="i">
    <default>720</default>
  </key>
  <key name="popup-width" type="i">
    <default>720</default>
  </key>
  <key name="popup-height" type="i">
    <default>520</default>
  </key>
</schema>

<enum id="acb-filter">
  <value value="all" />
  <value value="text" />
  <value value="images" />
  <value value="files" />
  <value value="pinned" />
  <value value="starred" />
  <value value="sensitive" />
</enum>

<enum id="acb-sort">
  <value value="newest" />
  <value value="oldest" />
  <value value="most-used" />
</enum>

<enum id="acb-page">
  <value value="clipboard" />
  <value value="emoji" />
  <value value="symbols" />
  <value value="kaomoji" />
  <value value="snippets" />
  <value value="settings" />
</enum>
```

`active_page` (popup) does not persist; only the manager does. The
popup is transient and should always open to the last filter / sort
the user used, not the last page.

## ClipboardItemObject (GObject wrapper)

```rust
#[derive(Debug, Clone, glib::Properties, Default)]
#[properties(
    getter, setter, type = ItemId
)]
pub struct ClipboardItemObject {
    #[property(get, set)]
    pub id: i64,
    #[property(get, set)]
    pub title: String,         // truncated preview
    #[property(get, set)]
    pub subtitle: String,      // mime, age, chars
    #[property(get, set)]
    pub content_type: String,  // "text" | "image" | "html" | "files"
    #[property(get, set)]
    pub mime_type: String,
    #[property(get, set)]
    pub timestamp: i64,
    #[property(get, set)]
    pub pinned: bool,
    #[property(get, set)]
    pub starred: bool,
    #[property(get, set)]
    pub sensitive: bool,
    #[property(get, set)]
    pub source_app: String,
    #[property(get, set)]
    pub thumbnail: Option<gdk_pixbuf::Pixbuf>,
    #[property(get, set)]
    pub full_content: String,        // never displayed unless revealed
    pub redacted_preview: String,    // always displayed for sensitive
    pub file_size: i64,
    pub file_paths: Vec<String>,
}
```

## IPC Command Set (unchanged)

The new UI consumes the same `IpcCommand::History`, `Pin`, `Unpin`,
`Delete`, `Copy`, `ToggleStar`, `ClearUnpinned`, `ListSnippets`,
`UpsertSnippet`, `DeleteSnippet`, `Status` commands the applet
already uses. No new IPC commands are required.

---

**Last Updated**: 2026-06-12
