//! author-clipboard: COSMIC clipboard manager applet
//!
//! A graphical interface for browsing and selecting from clipboard history,
//! with emoji and symbol pickers.

mod emoji;
mod kaomoji;
mod symbols;

use author_clipboard_shared::config::Config;
use author_clipboard_shared::file_handler;
use author_clipboard_shared::image_store;
use author_clipboard_shared::quick_paste::{self, PasteBackend};
use author_clipboard_shared::types::{AuditEventKind, ClipboardItem, Snippet};
use author_clipboard_shared::Database;
use cosmic::app::{Core, Settings, Task};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::event;
use cosmic::iced::keyboard::Key;
use cosmic::iced::{Length, Size, Subscription};
use cosmic::widget::{self, column, container, icon, row, scrollable, text, text_input};
use cosmic::{executor, iced, ApplicationExt, Element};
use tracing::{error, info, warn};

const APP_ID: &str = "com.namikofficial.author-clipboard";
const SEARCH_INPUT_ID: fn() -> widget::Id = || widget::Id::new("search-input");

fn clipboard_scroll_id() -> widget::Id {
    widget::Id::new("clipboard-scroll")
}

fn emoji_scroll_id() -> widget::Id {
    widget::Id::new("emoji-scroll")
}

fn symbol_scroll_id() -> widget::Id {
    widget::Id::new("symbol-scroll")
}

fn kaomoji_scroll_id() -> widget::Id {
    widget::Id::new("kaomoji-scroll")
}

/// Picker grid columns used for keyboard navigation and rendering layout.
const EMOJI_COLS: usize = 1;
const SYMBOL_COLS: usize = 1;
const KAOMOJI_COLS: usize = 1;
/// Number of rows jumped by PageUp/PageDown in picker tabs.
const PICKER_PAGE_SIZE: usize = 5;
const UI_TITLE_SIZE: f32 = 17.0;
const UI_SECTION_SIZE: f32 = 15.0;
const UI_META_SIZE: f32 = 11.0;

fn map_key_to_message(key: &Key, modifiers: iced::keyboard::Modifiers) -> Option<Message> {
    match key.as_ref() {
        Key::Named(iced::keyboard::key::Named::ArrowDown) => Some(Message::MoveDown),
        Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(Message::MoveUp),
        Key::Named(iced::keyboard::key::Named::ArrowLeft) => Some(Message::MoveLeft),
        Key::Named(iced::keyboard::key::Named::ArrowRight) => Some(Message::MoveRight),
        Key::Named(iced::keyboard::key::Named::Enter) => Some(Message::CopySelected),
        Key::Named(iced::keyboard::key::Named::Home) => Some(Message::SelectFirst),
        Key::Named(iced::keyboard::key::Named::End) => Some(Message::SelectLast),
        Key::Named(iced::keyboard::key::Named::PageDown) => Some(Message::PageDown),
        Key::Named(iced::keyboard::key::Named::PageUp) => Some(Message::PageUp),
        Key::Named(iced::keyboard::key::Named::Delete) => Some(Message::DeleteSelected),
        Key::Named(iced::keyboard::key::Named::Tab)
            if modifiers.shift() && !modifiers.control() =>
        {
            Some(Message::PreviousTab)
        }
        Key::Named(iced::keyboard::key::Named::Tab) if modifiers.control() => {
            if modifiers.shift() {
                Some(Message::PreviousTab)
            } else {
                Some(Message::NextTab)
            }
        }
        Key::Character("1") if modifiers.control() => Some(Message::QuickSelect(0)),
        Key::Character("2") if modifiers.control() => Some(Message::QuickSelect(1)),
        Key::Character("3") if modifiers.control() => Some(Message::QuickSelect(2)),
        Key::Character("4") if modifiers.control() => Some(Message::QuickSelect(3)),
        Key::Character("5") if modifiers.control() => Some(Message::QuickSelect(4)),
        Key::Character("6") if modifiers.control() => Some(Message::QuickSelect(5)),
        Key::Character("7") if modifiers.control() => Some(Message::QuickSelect(6)),
        Key::Character("8") if modifiers.control() => Some(Message::QuickSelect(7)),
        Key::Character("9") if modifiers.control() => Some(Message::QuickSelect(8)),
        Key::Character("d") if modifiers.control() => Some(Message::DeleteSelected),
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("author-clipboard applet starting...");

    let settings = Settings::default()
        .size(Size::new(520.0, 700.0))
        .debug(false);

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

// ── Tab definitions ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    Clipboard,
    Emoji,
    Symbols,
    Kaomoji,
    Snippets,
    Settings,
}

// ── Application state ─────────────────────────────────────────────────

struct App {
    core: Core,
    db: Option<Database>,
    config: Config,
    items: Vec<ClipboardItem>,
    search_query: String,
    selected_index: Option<usize>,
    active_tab: AppTab,
    tab_model: widget::segmented_button::SingleSelectModel,
    emoji_category_idx: usize,
    symbol_category_idx: usize,
    kaomoji_category_idx: usize,
    emoji_selected_idx: Option<usize>,
    symbol_selected_idx: Option<usize>,
    kaomoji_selected_idx: Option<usize>,
    incognito: bool,
    quick_paste_enabled: bool,
    paste_backend: Option<PasteBackend>,
    daemon_running: bool,
    snippets: Vec<Snippet>,
    snippet_search: String,
    snippet_name_input: String,
    snippet_content_input: String,
    scroll_offset_y: f32,
    denylist_input: String,
    content_denylist_input: String,
}

// ── Messages ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Message {
    KeyPressed {
        key: Key,
        modifiers: iced::keyboard::Modifiers,
        captured: bool,
    },
    Tick,
    TabSelected(widget::segmented_button::Entity),
    SearchChanged(String),
    CopyItem(i64),
    CopyText(String),
    TogglePin(i64),
    DeleteItem(i64),
    ClearAll,
    MoveDown,
    MoveUp,
    MoveLeft,
    MoveRight,
    CopySelected,
    EmojiCategory(usize),
    SymbolCategory(usize),
    KaomojiCategory(usize),
    ToggleIncognito,
    ExportData,
    ImportData,
    #[allow(dead_code)]
    QuickPaste(i64),
    ToggleQuickPaste,
    #[allow(dead_code)]
    OpenFileManager(String),
    NextTab,
    PreviousTab,
    QuickSelect(usize),
    DeleteSelected,
    SelectFirst,
    SelectLast,
    PageDown,
    PageUp,
    SnippetSearchChanged(String),
    SnippetAdd(String, String),
    SnippetDelete(i64),
    SnippetCopy(i64),
    SnippetNameInput(String),
    SnippetContentInput(String),
    ScrollOffsetChanged(f32),
    SettingMaxItems(usize),
    SettingMaxAgeDays(u64),
    SettingDedupWindow(u64),
    SettingToggleClearOnLock,
    SettingDenylistAdd(String),
    SettingDenylistRemove(usize),
    SettingDenylistInput(String),
    SettingContentDenylistAdd(String),
    SettingContentDenylistRemove(usize),
    SettingContentDenylistInput(String),
    SettingToggleEncryptSensitive,
    SettingMaxItemSize(usize),
    SettingCleanupInterval(u64),
}

// ── Application trait ─────────────────────────────────────────────────

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = Config::load();

        let db = match std::fs::create_dir_all(&config.data_dir) {
            Ok(()) => match Database::open(&config.db_path()) {
                Ok(db) => {
                    info!("Database opened at {}", config.db_path().display());
                    Some(db)
                }
                Err(e) => {
                    error!("Failed to open database: {e}");
                    None
                }
            },
            Err(e) => {
                error!("Failed to create data dir: {e}");
                None
            }
        };

        let items = db
            .as_ref()
            .and_then(|db| db.get_recent(config.max_items).ok())
            .unwrap_or_default();

        let snippets = db
            .as_ref()
            .and_then(|db| db.list_snippets().ok())
            .unwrap_or_default();

        let tab_model = widget::segmented_button::Model::builder()
            .insert(|b| b.text("Clipboard").data(AppTab::Clipboard).activate())
            .insert(|b| b.text("Emoji").data(AppTab::Emoji))
            .insert(|b| b.text("Symbols").data(AppTab::Symbols))
            .insert(|b| b.text("Kaomoji").data(AppTab::Kaomoji))
            .insert(|b| b.text("Snippets").data(AppTab::Snippets))
            .insert(|b| b.text("Settings").data(AppTab::Settings))
            .build();

        let incognito = config.is_incognito();
        let daemon_running = check_daemon_running();

        let mut app = App {
            core,
            db,
            config,
            items,
            search_query: String::new(),
            selected_index: None,
            active_tab: AppTab::Clipboard,
            tab_model,
            emoji_category_idx: 0,
            symbol_category_idx: 0,
            kaomoji_category_idx: 0,
            emoji_selected_idx: None,
            symbol_selected_idx: None,
            kaomoji_selected_idx: None,
            incognito,
            quick_paste_enabled: false,
            paste_backend: quick_paste::detect_backend(),
            daemon_running,
            snippets,
            snippet_search: String::new(),
            snippet_name_input: String::new(),
            snippet_content_input: String::new(),
            scroll_offset_y: 0.0,
            denylist_input: String::new(),
            content_denylist_input: String::new(),
        };

        let command = app.update_title();

        (app, command)
    }

    fn on_escape(&mut self) -> Task<Self::Message> {
        if self.search_query.is_empty() {
            std::process::exit(0);
        }
        self.search_query.clear();
        if self.active_tab == AppTab::Clipboard {
            self.refresh_items();
        }
        self.selected_index = None;
        self.emoji_selected_idx = None;
        self.symbol_selected_idx = None;
        self.kaomoji_selected_idx = None;
        Task::none()
    }

    fn on_search(&mut self) -> Task<Self::Message> {
        cosmic::widget::text_input::focus(SEARCH_INPUT_ID())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let keyboard = event::listen_with(|event, status, _| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(Message::KeyPressed {
                    key,
                    modifiers,
                    captured: matches!(status, event::Status::Captured),
                })
            }
            _ => None,
        });

        let tick =
            cosmic::iced::time::every(std::time::Duration::from_secs(2)).map(|_| Message::Tick);

        Subscription::batch([keyboard, tick])
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::KeyPressed {
                key,
                modifiers,
                captured,
            } => {
                if let Some(mapped) = map_key_to_message(&key, modifiers) {
                    // Let tab switching work globally. When a widget (e.g. search input) has captured
                    // the key event, don't also fire navigation — prevents desync on picker tabs.
                    if captured && !matches!(mapped, Message::NextTab | Message::PreviousTab) {
                        return Task::none();
                    }
                    return self.update(mapped);
                }
                return Task::none();
            }
            Message::Tick => {
                if self.active_tab == AppTab::Clipboard {
                    self.smart_refresh_items();
                }
                self.daemon_running = check_daemon_running();
            }
            Message::TabSelected(entity) => {
                self.tab_model.activate(entity);
                if let Some(&tab) = self.tab_model.data::<AppTab>(entity) {
                    self.active_tab = tab;
                    self.search_query.clear();
                    self.selected_index = None;
                    self.scroll_offset_y = 0.0;
                    self.emoji_selected_idx = None;
                    self.symbol_selected_idx = None;
                    self.kaomoji_selected_idx = None;
                    return self.scroll_active_tab_to_top();
                }
            }

            Message::SearchChanged(query) => {
                self.search_query = query;
                if self.active_tab == AppTab::Clipboard {
                    self.refresh_items();
                }
                self.selected_index = None;
                self.scroll_offset_y = 0.0;
                self.emoji_selected_idx = None;
                self.symbol_selected_idx = None;
                self.kaomoji_selected_idx = None;
                return self.scroll_active_tab_to_top();
            }

            Message::CopyItem(id) => {
                if let Some(item) = self.items.iter().find(|i| i.id == id) {
                    let result = if item.is_image() {
                        set_clipboard_image(
                            &image_store::image_path(&self.config.data_dir, &item.content),
                            &item.mime_type,
                        )
                    } else if item.is_html() {
                        set_clipboard_html(&item.content, item.plain_text.as_deref().unwrap_or(""))
                    } else {
                        set_clipboard_text(&item.content)
                    };
                    match result {
                        Ok(()) => {
                            info!("Copied item {} to clipboard", id);
                            std::process::exit(0);
                        }
                        Err(e) => warn!("Failed to set clipboard: {e}"),
                    }
                }
            }

            Message::CopyText(content) => {
                match set_clipboard_text(&content) {
                    Ok(()) => {
                        info!("Copied to clipboard: {}", truncate_content(&content, 20));
                        // Track recently used for pickers
                        if let Some(db) = &self.db {
                            let category = match self.active_tab {
                                AppTab::Emoji => Some("emoji"),
                                AppTab::Symbols => Some("symbol"),
                                AppTab::Kaomoji => Some("kaomoji"),
                                AppTab::Clipboard | AppTab::Settings | AppTab::Snippets => None,
                            };
                            if let Some(cat) = category {
                                if let Err(e) = db.record_usage(cat, &content) {
                                    warn!("Failed to record usage: {e}");
                                }
                            }
                        }
                        std::process::exit(0);
                    }
                    Err(e) => warn!("Failed to copy: {e}"),
                }
            }

            Message::TogglePin(id) => {
                if let Some(db) = &self.db {
                    if let Err(e) = db.toggle_pin(id) {
                        warn!("Failed to toggle pin: {e}");
                    }
                    self.refresh_items();
                }
            }

            Message::DeleteItem(id) => {
                if let Some(db) = &self.db {
                    if let Err(e) = db.delete_item(id) {
                        warn!("Failed to delete item: {e}");
                    } else {
                        let _ = db.log_audit_event(
                            &AuditEventKind::ItemDeleted,
                            Some(&format!("Item {id} deleted")),
                        );
                    }
                    self.refresh_items();
                }
            }

            Message::ClearAll => {
                if let Some(db) = &self.db {
                    if let Err(e) = db.clear_unpinned() {
                        warn!("Failed to clear items: {e}");
                    } else {
                        let _ = db.log_audit_event(&AuditEventKind::HistoryCleared, None);
                    }
                    self.refresh_items();
                }
            }

            Message::MoveDown => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        let len = self.filtered_emojis().len();
                        if len > 0 {
                            let next = match self.emoji_selected_idx {
                                Some(i) => i + EMOJI_COLS,
                                None => 0,
                            };
                            self.emoji_selected_idx = Some(next.min(len - 1));
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        let len = self.filtered_symbols().len();
                        if len > 0 {
                            let next = match self.symbol_selected_idx {
                                Some(i) => i + SYMBOL_COLS,
                                None => 0,
                            };
                            self.symbol_selected_idx = Some(next.min(len - 1));
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        let len = self.filtered_kaomoji().len();
                        if len > 0 {
                            let next = match self.kaomoji_selected_idx {
                                Some(i) => i + KAOMOJI_COLS,
                                None => 0,
                            };
                            self.kaomoji_selected_idx = Some(next.min(len - 1));
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => {
                        let len = self.visible_item_count();
                        if len > 0 {
                            self.selected_index = Some(match self.selected_index {
                                Some(i) if i + 1 < len => i + 1,
                                _ => 0,
                            });
                        }
                        self.scroll_to_selected()
                    }
                };
            }

            Message::MoveUp => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        let len = self.filtered_emojis().len();
                        if len > 0 {
                            self.emoji_selected_idx = Some(
                                self.emoji_selected_idx
                                    .unwrap_or(len)
                                    .saturating_sub(EMOJI_COLS),
                            );
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        let len = self.filtered_symbols().len();
                        if len > 0 {
                            self.symbol_selected_idx = Some(
                                self.symbol_selected_idx
                                    .unwrap_or(len)
                                    .saturating_sub(SYMBOL_COLS),
                            );
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        let len = self.filtered_kaomoji().len();
                        if len > 0 {
                            self.kaomoji_selected_idx = Some(
                                self.kaomoji_selected_idx
                                    .unwrap_or(len)
                                    .saturating_sub(KAOMOJI_COLS),
                            );
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => {
                        let len = self.visible_item_count();
                        if len > 0 {
                            self.selected_index = Some(match self.selected_index {
                                Some(0) | None => len.saturating_sub(1),
                                Some(i) => i - 1,
                            });
                        }
                        self.scroll_to_selected()
                    }
                };
            }

            Message::MoveLeft => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        if !self.filtered_emojis().is_empty() {
                            self.emoji_selected_idx =
                                Some(self.emoji_selected_idx.unwrap_or(0).saturating_sub(1));
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        if !self.filtered_symbols().is_empty() {
                            self.symbol_selected_idx =
                                Some(self.symbol_selected_idx.unwrap_or(0).saturating_sub(1));
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        if !self.filtered_kaomoji().is_empty() {
                            self.kaomoji_selected_idx =
                                Some(self.kaomoji_selected_idx.unwrap_or(0).saturating_sub(1));
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => Task::none(),
                };
            }

            Message::MoveRight => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        let len = self.filtered_emojis().len();
                        if len > 0 {
                            let next = match self.emoji_selected_idx {
                                Some(i) => i + 1,
                                None => 0,
                            };
                            self.emoji_selected_idx = Some(next.min(len - 1));
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        let len = self.filtered_symbols().len();
                        if len > 0 {
                            let next = match self.symbol_selected_idx {
                                Some(i) => i + 1,
                                None => 0,
                            };
                            self.symbol_selected_idx = Some(next.min(len - 1));
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        let len = self.filtered_kaomoji().len();
                        if len > 0 {
                            let next = match self.kaomoji_selected_idx {
                                Some(i) => i + 1,
                                None => 0,
                            };
                            self.kaomoji_selected_idx = Some(next.min(len - 1));
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => Task::none(),
                };
            }

            Message::CopySelected => {
                // Auto-select first item if nothing selected
                match self.active_tab {
                    AppTab::Emoji => {
                        if self.emoji_selected_idx.is_none() && !self.filtered_emojis().is_empty() {
                            self.emoji_selected_idx = Some(0);
                        }
                    }
                    AppTab::Symbols => {
                        if self.symbol_selected_idx.is_none() && !self.filtered_symbols().is_empty()
                        {
                            self.symbol_selected_idx = Some(0);
                        }
                    }
                    AppTab::Kaomoji => {
                        if self.kaomoji_selected_idx.is_none()
                            && !self.filtered_kaomoji().is_empty()
                        {
                            self.kaomoji_selected_idx = Some(0);
                        }
                    }
                    _ => {
                        if self.selected_index.is_none() && self.visible_item_count() > 0 {
                            self.selected_index = Some(0);
                        }
                    }
                }
                return self.copy_selected_item();
            }

            Message::EmojiCategory(idx) => {
                self.emoji_category_idx = idx;
                self.selected_index = None;
                self.emoji_selected_idx = None;
                return self.scroll_active_tab_to_top();
            }

            Message::SymbolCategory(idx) => {
                self.symbol_category_idx = idx;
                self.selected_index = None;
                self.symbol_selected_idx = None;
                return self.scroll_active_tab_to_top();
            }

            Message::KaomojiCategory(idx) => {
                self.kaomoji_category_idx = idx;
                self.selected_index = None;
                self.kaomoji_selected_idx = None;
                return self.scroll_active_tab_to_top();
            }

            Message::ToggleIncognito => {
                let new_state = !self.incognito;
                match self.config.set_incognito(new_state) {
                    Ok(state) => {
                        self.incognito = state;
                        info!(
                            "🕶️  Incognito mode {}",
                            if state { "enabled" } else { "disabled" }
                        );
                        if let Some(db) = &self.db {
                            let _ = db.log_audit_event(
                                &AuditEventKind::IncognitoToggled,
                                Some(if state { "enabled" } else { "disabled" }),
                            );
                        }
                    }
                    Err(e) => warn!("Failed to toggle incognito: {e}"),
                }
            }

            Message::ExportData => {
                if let Some(db) = &self.db {
                    let export_path = self.config.data_dir.join("clipboard_export.json");
                    match db.export_items() {
                        Ok(json) => match std::fs::write(&export_path, &json) {
                            Ok(()) => {
                                info!("📤 Exported clipboard data to {}", export_path.display());
                                if let Some(db) = &self.db {
                                    let _ = db.log_audit_event(
                                        &AuditEventKind::DataExported,
                                        Some(&format!("Exported to {}", export_path.display())),
                                    );
                                }
                            }
                            Err(e) => warn!("Failed to write export file: {e}"),
                        },
                        Err(e) => warn!("Failed to export data: {e}"),
                    }
                }
            }

            Message::ImportData => {
                if let Some(db) = &self.db {
                    let import_path = self.config.data_dir.join("clipboard_export.json");
                    if import_path.exists() {
                        match std::fs::read_to_string(&import_path) {
                            Ok(json) => match db.import_items(&json) {
                                Ok(count) => {
                                    info!("📥 Imported {count} clipboard items");
                                    self.refresh_items();
                                    if let Some(db) = &self.db {
                                        let _ = db.log_audit_event(
                                            &AuditEventKind::DataImported,
                                            Some(&format!("Imported {count} items")),
                                        );
                                    }
                                }
                                Err(e) => warn!("Failed to import data: {e}"),
                            },
                            Err(e) => warn!("Failed to read import file: {e}"),
                        }
                    } else {
                        warn!("No export file found at {}", import_path.display());
                    }
                }
            }
            Message::QuickPaste(id) => {
                if let Some(db) = &self.db {
                    if let Ok(Some(item)) = db.get_by_id(id) {
                        if item.sensitive && !self.quick_paste_enabled {
                            warn!(
                                "⚠️ Quick paste blocked: sensitive content requires confirmation"
                            );
                        } else if let Some(backend) = &self.paste_backend {
                            match quick_paste::quick_paste(&item.content, backend) {
                                Ok(result) => {
                                    if result.success {
                                        info!("⌨️ Quick pasted via {:?}", result.backend_used);
                                    } else {
                                        warn!(
                                            "Quick paste failed: {}",
                                            result.message.unwrap_or_default()
                                        );
                                    }
                                }
                                Err(e) => warn!("Quick paste error: {e}"),
                            }
                        } else {
                            warn!("No paste backend available (install wtype or ydotool)");
                        }
                    }
                }
            }
            Message::ToggleQuickPaste => {
                self.quick_paste_enabled = !self.quick_paste_enabled;
                info!(
                    "Quick paste: {}",
                    if self.quick_paste_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            Message::OpenFileManager(path) => {
                if let Err(e) = std::process::Command::new("xdg-open").arg(&path).spawn() {
                    warn!("Failed to open file manager: {e}");
                }
            }

            Message::NextTab => {
                self.cycle_tab(true);
                return self.scroll_active_tab_to_top();
            }

            Message::PreviousTab => {
                self.cycle_tab(false);
                return self.scroll_active_tab_to_top();
            }

            Message::QuickSelect(idx) => {
                let len = self.visible_item_count();
                if idx < len {
                    self.selected_index = Some(idx);
                    return self.copy_selected_item();
                }
            }

            Message::DeleteSelected => {
                if self.active_tab == AppTab::Clipboard {
                    if let Some(index) = self.selected_index {
                        if let Some(item) = self.items.get(index) {
                            let id = item.id;
                            if let Some(db) = &self.db {
                                if let Err(e) = db.delete_item(id) {
                                    warn!("Failed to delete item: {e}");
                                } else {
                                    let _ = db.log_audit_event(
                                        &AuditEventKind::ItemDeleted,
                                        Some(&format!("Item {id} deleted via keyboard")),
                                    );
                                }
                                self.refresh_items();
                                // Adjust selection if needed
                                let len = self.items.len();
                                if len == 0 {
                                    self.selected_index = None;
                                } else if index >= len {
                                    self.selected_index = Some(len - 1);
                                }
                            }
                        }
                    }
                }
            }

            Message::SelectFirst => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        if !self.filtered_emojis().is_empty() {
                            self.emoji_selected_idx = Some(0);
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        if !self.filtered_symbols().is_empty() {
                            self.symbol_selected_idx = Some(0);
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        if !self.filtered_kaomoji().is_empty() {
                            self.kaomoji_selected_idx = Some(0);
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => {
                        let len = self.visible_item_count();
                        if len > 0 {
                            self.selected_index = Some(0);
                        }
                        self.scroll_to_selected()
                    }
                };
            }

            Message::SelectLast => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        let len = self.filtered_emojis().len();
                        if len > 0 {
                            self.emoji_selected_idx = Some(len - 1);
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        let len = self.filtered_symbols().len();
                        if len > 0 {
                            self.symbol_selected_idx = Some(len - 1);
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        let len = self.filtered_kaomoji().len();
                        if len > 0 {
                            self.kaomoji_selected_idx = Some(len - 1);
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => {
                        let len = self.visible_item_count();
                        if len > 0 {
                            self.selected_index = Some(len - 1);
                        }
                        self.scroll_to_selected()
                    }
                };
            }

            Message::PageDown => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        let len = self.filtered_emojis().len();
                        if len > 0 {
                            let current = self.emoji_selected_idx.unwrap_or(0);
                            self.emoji_selected_idx =
                                Some((current + PICKER_PAGE_SIZE).min(len - 1));
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        let len = self.filtered_symbols().len();
                        if len > 0 {
                            let current = self.symbol_selected_idx.unwrap_or(0);
                            self.symbol_selected_idx =
                                Some((current + PICKER_PAGE_SIZE).min(len - 1));
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        let len = self.filtered_kaomoji().len();
                        if len > 0 {
                            let current = self.kaomoji_selected_idx.unwrap_or(0);
                            self.kaomoji_selected_idx =
                                Some((current + PICKER_PAGE_SIZE).min(len - 1));
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => {
                        let len = self.visible_item_count();
                        if len > 0 {
                            let page_size = 10;
                            let current = self.selected_index.unwrap_or(0);
                            self.selected_index = Some((current + page_size).min(len - 1));
                        }
                        self.scroll_to_selected()
                    }
                };
            }

            Message::PageUp => {
                return match self.active_tab {
                    AppTab::Emoji => {
                        if !self.filtered_emojis().is_empty() {
                            let current = self.emoji_selected_idx.unwrap_or(0);
                            self.emoji_selected_idx =
                                Some(current.saturating_sub(PICKER_PAGE_SIZE));
                        }
                        self.scroll_emoji_to_selected()
                    }
                    AppTab::Symbols => {
                        if !self.filtered_symbols().is_empty() {
                            let current = self.symbol_selected_idx.unwrap_or(0);
                            self.symbol_selected_idx =
                                Some(current.saturating_sub(PICKER_PAGE_SIZE));
                        }
                        self.scroll_symbol_to_selected()
                    }
                    AppTab::Kaomoji => {
                        if !self.filtered_kaomoji().is_empty() {
                            let current = self.kaomoji_selected_idx.unwrap_or(0);
                            self.kaomoji_selected_idx =
                                Some(current.saturating_sub(PICKER_PAGE_SIZE));
                        }
                        self.scroll_kaomoji_to_selected()
                    }
                    _ => {
                        let len = self.visible_item_count();
                        if len > 0 {
                            let page_size = 10;
                            let current = self.selected_index.unwrap_or(0);
                            self.selected_index = Some(current.saturating_sub(page_size));
                        }
                        self.scroll_to_selected()
                    }
                };
            }

            Message::SnippetSearchChanged(q) => {
                self.snippet_search.clone_from(&q);
                if let Some(db) = &self.db {
                    self.snippets = if q.is_empty() {
                        db.list_snippets().unwrap_or_default()
                    } else {
                        db.search_snippets(&q).unwrap_or_default()
                    };
                }
                return Task::none();
            }

            Message::SnippetAdd(name, content) => {
                if let Some(db) = &self.db {
                    if let Err(e) = db.upsert_snippet(&name, &content) {
                        warn!("Failed to save snippet: {e}");
                    } else {
                        self.snippet_name_input.clear();
                        self.snippet_content_input.clear();
                        self.snippets = db.list_snippets().unwrap_or_default();
                    }
                }
                return Task::none();
            }

            Message::SnippetDelete(id) => {
                if let Some(db) = &self.db {
                    if let Err(e) = db.delete_snippet(id) {
                        warn!("Failed to delete snippet: {e}");
                    } else {
                        self.snippets = db.list_snippets().unwrap_or_default();
                    }
                }
                return Task::none();
            }

            Message::SnippetCopy(id) => {
                if let Some(s) = self.snippets.iter().find(|s| s.id == id) {
                    match set_clipboard_text(&s.content) {
                        Ok(()) => std::process::exit(0),
                        Err(e) => warn!("Failed to copy snippet: {e}"),
                    }
                }
                return Task::none();
            }

            Message::SnippetNameInput(v) => {
                self.snippet_name_input = v;
                return Task::none();
            }

            Message::SnippetContentInput(v) => {
                self.snippet_content_input = v;
                return Task::none();
            }

            Message::ScrollOffsetChanged(y) => {
                self.scroll_offset_y = y;
                return Task::none();
            }

            Message::SettingMaxItems(val) => {
                self.config.max_items = val;
                let _ = self.config.save();
            }

            Message::SettingMaxAgeDays(val) => {
                self.config.ttl_seconds = val * 86400;
                let _ = self.config.save();
            }

            Message::SettingDedupWindow(val) => {
                self.config.dedup_window_seconds = val;
                let _ = self.config.save();
            }

            Message::SettingToggleClearOnLock => {
                self.config.clear_on_lock = !self.config.clear_on_lock;
                let _ = self.config.save();
            }

            Message::SettingDenylistAdd(rule) => {
                if !rule.trim().is_empty() {
                    self.config.mime_denylist.push(rule.trim().to_string());
                    let _ = self.config.save();
                    self.denylist_input.clear();
                }
            }

            Message::SettingDenylistRemove(idx) => {
                if idx < self.config.mime_denylist.len() {
                    self.config.mime_denylist.remove(idx);
                    let _ = self.config.save();
                }
            }

            Message::SettingDenylistInput(s) => {
                self.denylist_input = s;
            }

            Message::SettingContentDenylistAdd(rule) => {
                let trimmed = rule.trim();
                if !trimmed.is_empty() {
                    self.config.content_regex_denylist.push(trimmed.to_string());
                    let _ = self.config.save();
                    self.content_denylist_input.clear();
                }
            }

            Message::SettingContentDenylistRemove(idx) => {
                if idx < self.config.content_regex_denylist.len() {
                    self.config.content_regex_denylist.remove(idx);
                    let _ = self.config.save();
                }
            }

            Message::SettingContentDenylistInput(s) => {
                self.content_denylist_input = s;
            }

            Message::SettingToggleEncryptSensitive => {
                self.config.encrypt_sensitive = !self.config.encrypt_sensitive;
                let _ = self.config.save();
            }

            Message::SettingMaxItemSize(val) => {
                self.config.max_item_size = val;
                let _ = self.config.save();
            }

            Message::SettingCleanupInterval(val) => {
                self.config.cleanup_interval_seconds = val;
                let _ = self.config.save();
            }
        }

        Task::none()
    }

    #[allow(clippy::too_many_lines)]
    fn view(&self) -> Element<'_, Self::Message> {
        let tab_bar =
            widget::tab_bar::horizontal(&self.tab_model).on_activate(Message::TabSelected);

        let search_placeholder = match self.active_tab {
            AppTab::Clipboard => "Search clipboard history...",
            AppTab::Emoji => "Search emoji...",
            AppTab::Symbols => "Search symbols...",
            AppTab::Kaomoji => "Search kaomoji...",
            AppTab::Snippets | AppTab::Settings => "",
        };

        let search_bar = text_input(search_placeholder, &self.search_query)
            .on_input(Message::SearchChanged)
            .on_submit(|_| Message::CopySelected)
            .id(SEARCH_INPUT_ID())
            .leading_icon(
                icon::from_name("system-search-symbolic")
                    .size(16)
                    .icon()
                    .into(),
            )
            .width(Length::Fill)
            .padding(10);

        let incognito_btn = {
            let icon_name = if self.incognito {
                "object-locked-symbolic"
            } else {
                "object-unlocked-symbolic"
            };
            widget::button::icon(icon::from_name(icon_name).size(18))
                .on_press(Message::ToggleIncognito)
                .padding(6)
        };

        let header = match self.active_tab {
            AppTab::Clipboard => row()
                .spacing(6)
                .push(search_bar)
                .push(incognito_btn)
                .push(widget::tooltip(
                    widget::button::icon(icon::from_name("edit-clear-symbolic").size(18))
                        .on_press(Message::ClearAll)
                        .padding(6),
                    text("Clear unpinned").size(12.0),
                    widget::tooltip::Position::Bottom,
                ))
                .align_y(iced::Alignment::Center),
            AppTab::Snippets => row()
                .spacing(6)
                .push(
                    container(
                        text("Use the snippets search/input fields below").size(UI_META_SIZE),
                    )
                    .width(Length::Fill)
                    .padding([10, 12]),
                )
                .push(incognito_btn)
                .align_y(iced::Alignment::Center),
            AppTab::Settings => row()
                .spacing(6)
                .push(
                    container(text("Settings are shown below").size(UI_META_SIZE))
                        .width(Length::Fill)
                        .padding([10, 12]),
                )
                .push(incognito_btn)
                .align_y(iced::Alignment::Center),
            _ => row()
                .spacing(6)
                .push(search_bar)
                .push(incognito_btn)
                .align_y(iced::Alignment::Center),
        };

        let tab_content: Element<'_, Message> = match self.active_tab {
            AppTab::Clipboard => self.view_clipboard(),
            AppTab::Emoji => self.view_emoji(),
            AppTab::Symbols => self.view_symbols(),
            AppTab::Kaomoji => self.view_kaomoji(),
            AppTab::Snippets => self.view_snippets(),
            AppTab::Settings => self.view_settings(),
        };

        let status_text = match self.active_tab {
            AppTab::Clipboard => {
                let count = self.items.len();
                let pinned = self.items.iter().filter(|i| i.pinned).count();
                if pinned > 0 {
                    format!("{count} items ({pinned} pinned)")
                } else {
                    format!("{count} items")
                }
            }
            AppTab::Emoji => {
                let count = self.filtered_emojis().len();
                format!("{count} emoji")
            }
            AppTab::Symbols => {
                let count = self.filtered_symbols().len();
                format!("{count} symbols")
            }
            AppTab::Kaomoji => {
                let count = self.filtered_kaomoji().len();
                format!("{count} kaomoji")
            }
            AppTab::Snippets => {
                let count = self.snippets.len();
                format!("{count} snippets")
            }
            AppTab::Settings => String::from("Settings"),
        };

        let mut status_parts = vec![status_text];
        if self.daemon_running {
            status_parts.push("● Daemon".to_string());
        } else {
            status_parts.push("○ No Daemon".to_string());
        }
        if self.incognito {
            status_parts.push("Incognito".to_string());
        }
        if self.quick_paste_enabled {
            status_parts.push("Quick Paste".to_string());
        }
        let full_status = status_parts.join(" · ");

        let hints = match self.active_tab {
            AppTab::Clipboard => {
                "↑↓ Nav · PgUp/Dn · Home/End · Enter Paste · Del Remove · Esc Close"
            }
            AppTab::Emoji | AppTab::Symbols | AppTab::Kaomoji => {
                "↑↓←→ Nav · PgUp/Dn · Home/End · Enter Copy · Ctrl+Tab Switch · Esc Close"
            }
            _ => "Esc Close",
        };

        let status_bar = container(
            row()
                .push(text(full_status).size(11.0))
                .push(cosmic::iced::widget::horizontal_space())
                .push(text(hints).size(10.0))
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .padding([4, 8])
        .style(|theme| {
            let cosmic = theme.cosmic();
            let [r, g, b, _] = cosmic.bg_divider().into();
            cosmic::iced_widget::container::Style {
                background: Some(cosmic::iced::Color::from_rgba(r, g, b, 0.4).into()),
                border: cosmic::iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        let content = column()
            .spacing(10)
            .padding(12)
            .push(container(tab_bar).padding([4, 6]).style(|theme| {
                let cosmic = theme.cosmic();
                let [r, g, b, _] = cosmic.bg_divider().into();
                cosmic::iced_widget::container::Style {
                    background: Some(cosmic::iced::Color::from_rgba(r, g, b, 0.25).into()),
                    border: cosmic::iced::Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }))
            .push(container(header).padding([2, 2]).style(|theme| {
                let cosmic = theme.cosmic();
                let [r, g, b, _] = cosmic.bg_divider().into();
                cosmic::iced_widget::container::Style {
                    background: Some(cosmic::iced::Color::from_rgba(r, g, b, 0.15).into()),
                    border: cosmic::iced::Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }))
            .push(tab_content)
            .push(status_bar);

        Element::from(content)
    }
}

// ── Helper methods ────────────────────────────────────────────────────

impl App {
    fn scroll_active_tab_to_top(&self) -> Task<Message> {
        match self.active_tab {
            AppTab::Clipboard => cosmic::iced_widget::scrollable::scroll_to(
                clipboard_scroll_id(),
                cosmic::iced_widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            ),
            AppTab::Emoji => cosmic::iced_widget::scrollable::scroll_to(
                emoji_scroll_id(),
                cosmic::iced_widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            ),
            AppTab::Symbols => cosmic::iced_widget::scrollable::scroll_to(
                symbol_scroll_id(),
                cosmic::iced_widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            ),
            AppTab::Kaomoji => cosmic::iced_widget::scrollable::scroll_to(
                kaomoji_scroll_id(),
                cosmic::iced_widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            ),
            AppTab::Snippets | AppTab::Settings => Task::none(),
        }
    }

    fn refresh_items(&mut self) {
        if let Some(db) = &self.db {
            let result = if self.search_query.is_empty() {
                db.get_recent(self.config.max_items)
            } else {
                db.search(&self.search_query, self.config.max_items)
            };

            match result {
                Ok(items) => self.items = items,
                Err(e) => warn!("Failed to load items: {e}"),
            }
        }
    }

    /// Refresh items only when the data has actually changed to preserve scroll position.
    fn smart_refresh_items(&mut self) {
        if let Some(db) = &self.db {
            let result = if self.search_query.is_empty() {
                db.get_recent(self.config.max_items)
            } else {
                db.search(&self.search_query, self.config.max_items)
            };

            match result {
                Ok(new_items) => {
                    // Only update if items changed (compare IDs and pin states)
                    let changed = new_items.len() != self.items.len()
                        || new_items
                            .iter()
                            .zip(self.items.iter())
                            .any(|(a, b)| a.id != b.id || a.pinned != b.pinned);
                    if changed {
                        self.items = new_items;
                    }
                }
                Err(e) => warn!("Failed to load items: {e}"),
            }
        }
    }

    fn visible_item_count(&self) -> usize {
        match self.active_tab {
            AppTab::Clipboard => self.items.len(),
            AppTab::Emoji => self.filtered_emojis().len(),
            AppTab::Symbols => self.filtered_symbols().len(),
            AppTab::Kaomoji => self.filtered_kaomoji().len(),
            AppTab::Snippets => self.snippets.len(),
            AppTab::Settings => 0,
        }
    }

    fn filtered_emojis(&self) -> Vec<&'static str> {
        if self.search_query.is_empty() {
            let cat = &emoji::CATEGORIES[self.emoji_category_idx];
            cat.emojis.to_vec()
        } else {
            emoji::search(&self.search_query)
        }
    }

    fn filtered_symbols(&self) -> Vec<(&str, &str)> {
        if self.search_query.is_empty() {
            let cat = &symbols::CATEGORIES[self.symbol_category_idx];
            cat.symbols.to_vec()
        } else {
            symbols::search(&self.search_query)
        }
    }

    fn filtered_kaomoji(&self) -> Vec<&'static str> {
        if self.search_query.is_empty() {
            let cat = &kaomoji::CATEGORIES[self.kaomoji_category_idx];
            cat.items.to_vec()
        } else {
            kaomoji::search(&self.search_query)
        }
    }

    // ── Clipboard tab view ────────────────────────────────────────────

    fn view_clipboard(&self) -> Element<'_, Message> {
        let total_items = self.items.len();
        let pinned_items = self.items.iter().filter(|i| i.pinned).count();
        let sensitive_items = self.items.iter().filter(|i| i.sensitive).count();
        let summary_row = row()
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .push(
                widget::button::standard(format!("Total {total_items}"))
                    .padding([4, 10])
                    .width(Length::Shrink),
            )
            .push(
                widget::button::standard(format!("Pinned {pinned_items}"))
                    .padding([4, 10])
                    .width(Length::Shrink),
            )
            .push(
                widget::button::standard(format!("Sensitive {sensitive_items}"))
                    .padding([4, 10])
                    .width(Length::Shrink),
            );

        if self.items.is_empty() {
            let (icon_name, msg) = if self.search_query.is_empty() {
                (
                    "edit-paste-symbolic",
                    "No clipboard items yet\nCopy something to get started!",
                )
            } else {
                ("system-search-symbolic", "No items match your search")
            };

            container(
                column().spacing(12).push(summary_row).push(
                    container(
                        column()
                            .spacing(12)
                            .align_x(Horizontal::Center)
                            .push(icon::from_name(icon_name).size(48).icon())
                            .push(text(msg).size(14.0).align_x(Horizontal::Center)),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center),
                ),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            let mut list = column().spacing(4).padding([0, 4]);

            for (index, item) in self.items.iter().enumerate() {
                list = list.push(self.clipboard_item_row(item, index));
            }

            column()
                .spacing(8)
                .push(summary_row)
                .push(
                    container(
                        scrollable(list)
                            .id(clipboard_scroll_id())
                            .on_scroll(|viewport| {
                                Message::ScrollOffsetChanged(viewport.absolute_offset().y)
                            })
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .padding([4, 6])
                    .style(|theme| {
                        let cosmic = theme.cosmic();
                        let [r, g, b, _] = cosmic.bg_divider().into();
                        cosmic::iced_widget::container::Style {
                            background: Some(cosmic::iced::Color::from_rgba(r, g, b, 0.25).into()),
                            border: cosmic::iced::Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .height(Length::Fill),
                )
                .into()
        }
    }

    #[allow(clippy::too_many_lines)]
    fn clipboard_item_row(&self, item: &ClipboardItem, index: usize) -> Element<'_, Message> {
        let time_ago = format_time_ago(item.timestamp);

        // Position number (1-9 for quick select, blank for rest)
        let position_label = if index < 9 {
            format!("{}", index + 1)
        } else {
            String::from(" ")
        };

        let pin_btn = widget::tooltip(
            widget::button::icon(
                icon::from_name(if item.pinned {
                    "pin-symbolic"
                } else {
                    "mail-mark-important-symbolic"
                })
                .size(16),
            )
            .on_press(Message::TogglePin(item.id))
            .padding(4),
            text(if item.pinned { "Unpin" } else { "Pin" }).size(11.0),
            widget::tooltip::Position::Bottom,
        );

        let delete_btn = widget::tooltip(
            widget::button::icon(icon::from_name("edit-delete-symbolic").size(16))
                .on_press(Message::DeleteItem(item.id))
                .padding(4),
            text("Delete").size(11.0),
            widget::tooltip::Position::Bottom,
        );

        let content_col = if item.is_image() {
            let thumb_path = image_store::thumbnail_path(&self.config.data_dir, &item.content);
            let mut col = column().spacing(2);
            if thumb_path.exists() {
                let handle = widget::image::Handle::from_path(&thumb_path);
                col = col.push(
                    widget::image(handle)
                        .width(Length::Fixed(80.0))
                        .height(Length::Fixed(60.0)),
                );
            }
            col = col.push(
                row()
                    .spacing(4)
                    .push(icon::from_name("image-x-generic-symbolic").size(12).icon())
                    .push(text(format!("Image ({})", &item.mime_type)).size(12.0))
                    .align_y(iced::Alignment::Center),
            );
            col.push(text(time_ago).size(11.0))
        } else if item.is_html() {
            let preview_text = item.plain_text.as_deref().unwrap_or(&item.content);
            let preview = truncate_content(preview_text, 120);
            let mut col = column().spacing(2);
            col = col.push(text(preview).size(13.0));
            col = col.push(
                row()
                    .spacing(4)
                    .push(icon::from_name("text-html-symbolic").size(12).icon())
                    .push(text("HTML content").size(11.0))
                    .align_y(iced::Alignment::Center),
            );
            col.push(text(time_ago).size(11.0))
        } else if item.is_files() {
            let file_infos = file_handler::parse_uri_list(&item.content);
            let file_count = file_infos.len();
            let mut col = column().spacing(2);

            if file_infos.is_empty() {
                col = col.push(text("No files").size(13.0));
            } else {
                for info in file_infos.iter().take(3) {
                    let size_str = file_handler::format_file_size(info.size);
                    let status = if info.exists { "" } else { " ⚠️ missing" };
                    let file_text = format!("{} ({}){}", info.name, size_str, status);
                    col = col.push(text(file_text).size(12.0));
                }
                if file_count > 3 {
                    col = col.push(text(format!("  ... and {} more", file_count - 3)).size(11.0));
                }
            }
            col = col.push(
                row()
                    .spacing(4)
                    .push(icon::from_name("document-open-symbolic").size(12).icon())
                    .push(text(format!("{file_count} file(s)")).size(11.0))
                    .align_y(iced::Alignment::Center),
            );
            col.push(text(time_ago).size(11.0))
        } else {
            let preview = truncate_content(&item.content, 120);
            let char_count = item.content.len();
            let word_count = item.content.split_whitespace().count();
            let meta = format!("{char_count} chars · {word_count} words");
            let mut col = column().spacing(2).push(text(preview).size(13.0));
            if item.sensitive {
                col = col.push(
                    row()
                        .spacing(4)
                        .push(icon::from_name("dialog-warning-symbolic").size(12).icon())
                        .push(text("Sensitive content").size(11.0))
                        .align_y(iced::Alignment::Center),
                );
            }
            col = col.push(
                row()
                    .spacing(8)
                    .push(text(time_ago).size(11.0))
                    .push(text("·").size(11.0))
                    .push(text(meta).size(10.0))
                    .align_y(iced::Alignment::Center),
            );
            col
        };

        let row_content = row()
            .spacing(8)
            .push(text(position_label).size(11.0).width(Length::Fixed(14.0)))
            .push(pin_btn)
            .push(container(content_col).width(Length::Fill))
            .push(delete_btn)
            .align_y(iced::Alignment::Center);

        let is_selected = self.selected_index == Some(index);

        let item_btn = widget::button::custom(row_content)
            .width(Length::Fill)
            .padding([8, 8])
            .on_press(Message::CopyItem(item.id));

        if is_selected {
            container(item_btn)
                .style(|theme| {
                    let cosmic = theme.cosmic();
                    let [r, g, b, a] = cosmic.accent.base.into();
                    cosmic::iced_widget::container::Style {
                        background: Some(cosmic::iced::Color::from_rgba(r, g, b, a * 0.3).into()),
                        border: cosmic::iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .into()
        } else {
            item_btn.into()
        }
    }

    // ── Emoji tab view ────────────────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    fn view_emoji(&self) -> Element<'_, Message> {
        let mut content = column().spacing(8);

        let emojis: Vec<&str> = if self.search_query.is_empty() {
            let cat = &emoji::CATEGORIES[self.emoji_category_idx];
            cat.emojis.to_vec()
        } else {
            emoji::search(&self.search_query)
        };
        let valid_emoji_idx = self.emoji_selected_idx.filter(|&i| i < emojis.len());
        let selected_label = valid_emoji_idx.map_or(String::from("No selection"), |i| {
            format!("Selected {}", i + 1)
        });
        let selected_emoji = valid_emoji_idx
            .and_then(|i| emojis.get(i).copied())
            .unwrap_or("—");
        let codepoints = if selected_emoji == "—" {
            String::from("U+----")
        } else {
            selected_emoji
                .chars()
                .map(|c| format!("U+{:04X}", u32::from(c)))
                .collect::<Vec<_>>()
                .join(" ")
        };
        content = content.push(
            row()
                .align_y(iced::Alignment::Center)
                .push(text("Emoji").size(UI_TITLE_SIZE))
                .push(cosmic::iced::widget::horizontal_space())
                .push(text(selected_label).size(UI_META_SIZE)),
        );
        content = content.push(
            row()
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .push(widget::button::standard(selected_emoji).padding([4, 10]))
                .push(text(codepoints).size(UI_META_SIZE)),
        );

        if let Some(db) = &self.db {
            if let Ok(recent) = db.get_recently_used("emoji", 8) {
                if !recent.is_empty() {
                    let mut recent_row = row().spacing(4).align_y(iced::Alignment::Center);
                    recent_row = recent_row.push(text("Recent:").size(12.0));
                    for e in recent {
                        let label = e;
                        recent_row = recent_row.push(
                            widget::button::text(label.clone())
                                .on_press(Message::CopyText(label))
                                .padding([4, 8]),
                        );
                    }
                    content = content.push(recent_row);
                }
            }
        }
        content = content.push(text("────────────────────────────────").size(UI_META_SIZE));

        if self.search_query.is_empty() {
            let mut cat_row = row().spacing(8);
            for (idx, cat) in emoji::CATEGORIES.iter().enumerate() {
                let label = format!("{} {}", cat.icon, cat.name);
                let btn = if idx == self.emoji_category_idx {
                    widget::button::suggested(label)
                        .on_press(Message::EmojiCategory(idx))
                        .padding([6, 14])
                } else {
                    widget::button::text(label)
                        .on_press(Message::EmojiCategory(idx))
                        .padding([6, 14])
                };
                cat_row = cat_row.push(btn);
            }
            content = content.push(widget::scrollable::horizontal(cat_row));
            content = content.push(text(format!("{} emoji", emojis.len())).size(UI_META_SIZE));
        } else {
            content = content.push(text(format!("{} results", emojis.len())).size(UI_META_SIZE));
        }

        if emojis.is_empty() {
            return container(
                column()
                    .spacing(8)
                    .align_x(Horizontal::Center)
                    .push(icon::from_name("face-smile-symbolic").size(40).icon())
                    .push(
                        text("No emoji found")
                            .size(13.0)
                            .align_x(Horizontal::Center),
                    ),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into();
        }

        let mut grid = column().spacing(6);
        let mut flat_idx = 0usize;
        for chunk in emojis.chunks(EMOJI_COLS) {
            let mut grid_row = row().spacing(6);
            for &emoji_char in chunk {
                let is_selected = self.emoji_selected_idx == Some(flat_idx);
                let btn = if is_selected {
                    widget::button::suggested(emoji_char)
                        .on_press(Message::CopyText(emoji_char.to_string()))
                        .width(Length::FillPortion(1))
                        .padding([12, 8])
                } else {
                    widget::button::text(emoji_char)
                        .on_press(Message::CopyText(emoji_char.to_string()))
                        .width(Length::FillPortion(1))
                        .padding([12, 8])
                };
                grid_row = grid_row.push(btn);
                flat_idx += 1;
            }
            for _ in chunk.len()..EMOJI_COLS {
                grid_row = grid_row.push(
                    container(cosmic::iced::widget::horizontal_space())
                        .width(Length::FillPortion(1)),
                );
            }
            grid = grid.push(grid_row);
        }

        content = content.push(
            container(
                scrollable(grid)
                    .id(emoji_scroll_id())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .padding([6, 8])
            .height(Length::Fill)
            .style(|theme| {
                let cosmic = theme.cosmic();
                let [r, g, b, _] = cosmic.bg_divider().into();
                cosmic::iced_widget::container::Style {
                    background: Some(cosmic::iced::Color::from_rgba(r, g, b, 0.25).into()),
                    border: cosmic::iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
        );

        content.into()
    }

    // ── Symbols tab view ──────────────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    fn view_symbols(&self) -> Element<'_, Message> {
        let mut content = column().spacing(8);

        let syms: Vec<(&str, &str)> = if self.search_query.is_empty() {
            let cat = &symbols::CATEGORIES[self.symbol_category_idx];
            cat.symbols.to_vec()
        } else {
            symbols::search(&self.search_query)
        };
        let valid_symbol_idx = self.symbol_selected_idx.filter(|&i| i < syms.len());
        let selected_label = valid_symbol_idx.map_or(String::from("No selection"), |i| {
            format!("Selected {}", i + 1)
        });
        let selected_symbol = valid_symbol_idx
            .and_then(|i| syms.get(i).copied())
            .unwrap_or(("—", "No symbol selected"));
        content = content.push(
            row()
                .align_y(iced::Alignment::Center)
                .push(text("Symbols").size(UI_TITLE_SIZE))
                .push(cosmic::iced::widget::horizontal_space())
                .push(text(selected_label).size(UI_META_SIZE)),
        );
        content = content.push(
            row()
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .push(widget::button::standard(selected_symbol.0).padding([4, 10]))
                .push(text(selected_symbol.1).size(UI_META_SIZE)),
        );

        if let Some(db) = &self.db {
            if let Ok(recent) = db.get_recently_used("symbol", 8) {
                if !recent.is_empty() {
                    let mut recent_row = row().spacing(4).align_y(iced::Alignment::Center);
                    recent_row = recent_row.push(text("Recent:").size(12.0));
                    for s in recent {
                        let label = s;
                        recent_row = recent_row.push(
                            widget::button::text(label.clone())
                                .on_press(Message::CopyText(label))
                                .padding([4, 8]),
                        );
                    }
                    content = content.push(recent_row);
                }
            }
        }
        content = content.push(text("────────────────────────────────").size(UI_META_SIZE));

        if self.search_query.is_empty() {
            let mut cat_row = row().spacing(8);
            for (idx, cat) in symbols::CATEGORIES.iter().enumerate() {
                let label = format!("{} {}", cat.icon, cat.name);
                let btn = if idx == self.symbol_category_idx {
                    widget::button::suggested(label)
                        .on_press(Message::SymbolCategory(idx))
                        .padding([6, 14])
                } else {
                    widget::button::text(label)
                        .on_press(Message::SymbolCategory(idx))
                        .padding([6, 14])
                };
                cat_row = cat_row.push(btn);
            }
            content = content.push(widget::scrollable::horizontal(cat_row));
            content = content.push(text(format!("{} symbols", syms.len())).size(UI_META_SIZE));
        } else {
            content = content.push(text(format!("{} results", syms.len())).size(UI_META_SIZE));
        }

        if syms.is_empty() {
            return container(
                column()
                    .spacing(8)
                    .align_x(Horizontal::Center)
                    .push(icon::from_name("insert-text-symbolic").size(40).icon())
                    .push(
                        text("No symbols found")
                            .size(13.0)
                            .align_x(Horizontal::Center),
                    ),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into();
        }

        let mut list = column().spacing(6);
        let mut flat_idx = 0usize;
        for chunk in syms.chunks(SYMBOL_COLS) {
            let mut grid_row = row().spacing(6);
            for &(sym, desc) in chunk {
                let is_selected = self.symbol_selected_idx == Some(flat_idx);
                let sym_btn = if is_selected {
                    widget::button::suggested(sym)
                        .on_press(Message::CopyText(sym.to_string()))
                        .width(Length::FillPortion(1))
                        .padding([12, 10])
                } else {
                    widget::button::text(sym)
                        .on_press(Message::CopyText(sym.to_string()))
                        .width(Length::FillPortion(1))
                        .padding([12, 10])
                };
                let btn = widget::tooltip(
                    sym_btn,
                    text(desc).size(12.0),
                    widget::tooltip::Position::Bottom,
                );
                grid_row = grid_row.push(btn);
                flat_idx += 1;
            }
            for _ in chunk.len()..SYMBOL_COLS {
                grid_row = grid_row.push(
                    container(cosmic::iced::widget::horizontal_space())
                        .width(Length::FillPortion(1)),
                );
            }
            list = list.push(grid_row);
        }

        content = content.push(
            container(
                scrollable(list)
                    .id(symbol_scroll_id())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .padding([6, 8])
            .height(Length::Fill)
            .style(|theme| {
                let cosmic = theme.cosmic();
                let [r, g, b, _] = cosmic.bg_divider().into();
                cosmic::iced_widget::container::Style {
                    background: Some(cosmic::iced::Color::from_rgba(r, g, b, 0.25).into()),
                    border: cosmic::iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
        );

        content.into()
    }

    // ── Kaomoji tab view ──────────────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    fn view_kaomoji(&self) -> Element<'_, Message> {
        let mut content = column().spacing(8).width(Length::Fill);

        let items = self.filtered_kaomoji();
        let valid_kaomoji_idx = self.kaomoji_selected_idx.filter(|&i| i < items.len());
        let selected_label = valid_kaomoji_idx.map_or(String::from("No selection"), |i| {
            format!("Selected {}", i + 1)
        });
        let selected_kaomoji = valid_kaomoji_idx
            .and_then(|i| items.get(i).copied())
            .unwrap_or("—");
        let selected_len = if selected_kaomoji == "—" {
            0
        } else {
            selected_kaomoji.chars().count()
        };
        content = content.push(
            row()
                .align_y(iced::Alignment::Center)
                .push(text("Kaomoji").size(UI_TITLE_SIZE))
                .push(cosmic::iced::widget::horizontal_space())
                .push(text(selected_label).size(UI_META_SIZE)),
        );
        content = content.push(
            row()
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .push(text(selected_kaomoji).size(14.0))
                .push(text(format!("{selected_len} chars")).size(UI_META_SIZE)),
        );

        if let Some(db) = &self.db {
            if let Ok(recent) = db.get_recently_used("kaomoji", 8) {
                if !recent.is_empty() {
                    let mut recent_row = row().spacing(4).align_y(iced::Alignment::Center);
                    recent_row = recent_row.push(text("Recent:").size(12.0));
                    for k in recent {
                        let label = k;
                        recent_row = recent_row.push(
                            widget::button::text(label.clone())
                                .on_press(Message::CopyText(label))
                                .padding([4, 8]),
                        );
                    }
                    content = content.push(recent_row);
                }
            }
        }
        content = content.push(text("────────────────────────────────").size(UI_META_SIZE));

        if self.search_query.is_empty() {
            let mut cat_row = row().spacing(8);
            for (idx, cat) in kaomoji::CATEGORIES.iter().enumerate() {
                let label = format!("{} {}", cat.icon, cat.name);
                let btn = if idx == self.kaomoji_category_idx {
                    widget::button::suggested(label)
                        .on_press(Message::KaomojiCategory(idx))
                        .padding([6, 14])
                } else {
                    widget::button::text(label)
                        .on_press(Message::KaomojiCategory(idx))
                        .padding([6, 14])
                };
                cat_row = cat_row.push(btn);
            }
            content = content.push(widget::scrollable::horizontal(cat_row));
            content = content.push(text(format!("{} kaomoji", items.len())).size(UI_META_SIZE));
        } else {
            content = content.push(text(format!("{} results", items.len())).size(UI_META_SIZE));
        }

        if items.is_empty() {
            content = content.push(
                container(
                    text("No results found")
                        .size(13.0)
                        .align_x(Horizontal::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            );
        } else {
            let mut list = column().spacing(6).width(Length::Fill);
            let mut flat_idx = 0usize;
            for chunk in items.chunks(KAOMOJI_COLS) {
                let mut grid_row = row().spacing(6);
                for &kaomoji_str in chunk {
                    let is_selected = self.kaomoji_selected_idx == Some(flat_idx);
                    let btn = if is_selected {
                        widget::button::suggested(kaomoji_str)
                            .on_press(Message::CopyText(kaomoji_str.to_string()))
                            .width(Length::FillPortion(1))
                            .padding([10, 14])
                    } else {
                        widget::button::text(kaomoji_str)
                            .on_press(Message::CopyText(kaomoji_str.to_string()))
                            .width(Length::FillPortion(1))
                            .padding([10, 14])
                    };
                    grid_row = grid_row.push(btn);
                    flat_idx += 1;
                }
                for _ in chunk.len()..KAOMOJI_COLS {
                    grid_row = grid_row.push(
                        container(cosmic::iced::widget::horizontal_space())
                            .width(Length::FillPortion(1)),
                    );
                }
                list = list.push(grid_row);
            }
            content = content.push(
                container(
                    scrollable(list)
                        .id(kaomoji_scroll_id())
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .padding([6, 8])
                .height(Length::Fill)
                .style(|theme| {
                    let cosmic = theme.cosmic();
                    let [r, g, b, _] = cosmic.bg_divider().into();
                    cosmic::iced_widget::container::Style {
                        background: Some(cosmic::iced::Color::from_rgba(r, g, b, 0.25).into()),
                        border: cosmic::iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
            );
        }

        content.into()
    }

    // ── Snippets tab view ─────────────────────────────────────────────

    fn view_snippets(&self) -> Element<'_, Message> {
        let mut content = column().spacing(8).width(Length::Fill);
        content = content.push(
            row()
                .align_y(iced::Alignment::Center)
                .push(text("Snippets").size(UI_TITLE_SIZE))
                .push(cosmic::iced::widget::horizontal_space())
                .push(text(format!("{} total", self.snippets.len())).size(UI_META_SIZE)),
        );

        // Search bar for snippets
        let search = text_input("Search snippets...", &self.snippet_search)
            .on_input(Message::SnippetSearchChanged)
            .leading_icon(
                icon::from_name("system-search-symbolic")
                    .size(16)
                    .icon()
                    .into(),
            )
            .width(Length::Fill)
            .padding(6);
        content = content.push(search);

        // Add snippet form
        let name_input = text_input("Name", &self.snippet_name_input)
            .on_input(Message::SnippetNameInput)
            .width(Length::Fill)
            .padding(6);
        let content_input = text_input("Content", &self.snippet_content_input)
            .on_input(Message::SnippetContentInput)
            .width(Length::Fill)
            .padding(6);
        let can_add = !self.snippet_name_input.is_empty() && !self.snippet_content_input.is_empty();
        let add_btn = if can_add {
            widget::button::suggested("Add")
                .on_press(Message::SnippetAdd(
                    self.snippet_name_input.clone(),
                    self.snippet_content_input.clone(),
                ))
                .padding([6, 12])
        } else {
            widget::button::suggested("Add").padding([6, 12])
        };
        let form_row = row()
            .spacing(6)
            .push(name_input)
            .push(content_input)
            .push(add_btn)
            .align_y(iced::Alignment::Center);
        content = content.push(form_row);

        // Snippet list
        if self.snippets.is_empty() {
            let msg = if self.snippet_search.is_empty() {
                "No snippets yet — add one above!"
            } else {
                "No snippets match your search"
            };
            let empty = container(
                column()
                    .spacing(8)
                    .align_x(Horizontal::Center)
                    .push(icon::from_name("edit-copy-symbolic").size(36).icon())
                    .push(text(msg).size(13.0).align_x(Horizontal::Center)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
            content = content.push(empty);
        } else {
            let mut list = column().spacing(4).padding([0, 4]);
            for snippet in &self.snippets {
                let preview = truncate_content(&snippet.content, 60);
                let copy_btn = widget::tooltip(
                    widget::button::icon(icon::from_name("edit-copy-symbolic").size(16))
                        .on_press(Message::SnippetCopy(snippet.id))
                        .padding(4),
                    text("Copy").size(11.0),
                    widget::tooltip::Position::Bottom,
                );
                let delete_btn = widget::tooltip(
                    widget::button::icon(icon::from_name("edit-delete-symbolic").size(16))
                        .on_press(Message::SnippetDelete(snippet.id))
                        .padding(4),
                    text("Delete").size(11.0),
                    widget::tooltip::Position::Bottom,
                );
                let info_col = column()
                    .spacing(2)
                    .push(text(&snippet.name).size(13.0))
                    .push(text(preview).size(11.0));
                let snippet_row = row()
                    .spacing(8)
                    .push(container(info_col).width(Length::Fill))
                    .push(copy_btn)
                    .push(delete_btn)
                    .align_y(iced::Alignment::Center);
                let snippet_btn = widget::button::custom(snippet_row)
                    .width(Length::Fill)
                    .padding([8, 8])
                    .on_press(Message::SnippetCopy(snippet.id));
                list = list.push(snippet_btn);
            }
            content = content.push(scrollable(list).width(Length::Fill).height(Length::Fill));
        }

        content.into()
    }

    // ── Settings tab view ─────────────────────────────────────────────

    #[allow(clippy::too_many_lines)]
    fn view_settings(&self) -> Element<'_, Message> {
        let version = env!("CARGO_PKG_VERSION");
        let mut content = column().spacing(14).width(Length::Fill);
        content = content.push(
            row()
                .align_y(iced::Alignment::Center)
                .push(text("Settings").size(UI_TITLE_SIZE))
                .push(cosmic::iced::widget::horizontal_space())
                .push(text("Configuration & Diagnostics").size(UI_META_SIZE)),
        );
        content = content.push(
            row()
                .spacing(8)
                .push(
                    widget::button::standard(if self.daemon_running {
                        "Daemon Online"
                    } else {
                        "Daemon Offline"
                    })
                    .padding([5, 10]),
                )
                .push(
                    widget::button::standard(if self.incognito {
                        "Incognito On"
                    } else {
                        "Incognito Off"
                    })
                    .padding([5, 10]),
                )
                .push(
                    widget::button::standard(if self.quick_paste_enabled {
                        "Quick Paste On"
                    } else {
                        "Quick Paste Off"
                    })
                    .padding([5, 10]),
                ),
        );

        // ── Status ───────────────────────────────────────────────────
        content = content.push(text("Status").size(UI_SECTION_SIZE));
        let daemon_status = if self.daemon_running {
            "● Daemon is running — clipboard changes are being captured"
        } else {
            "○ Daemon is not running — run: author-clipboard-daemon"
        };
        let daemon_btn = if self.daemon_running {
            widget::button::suggested(daemon_status)
                .width(Length::Fill)
                .padding([10, 16])
        } else {
            widget::button::standard(daemon_status)
                .width(Length::Fill)
                .padding([10, 16])
        };
        content = content.push(daemon_btn);

        // ── Privacy ──────────────────────────────────────────────────
        content = content.push(text("Privacy").size(UI_SECTION_SIZE));
        let incognito_label = if self.incognito {
            "Incognito Mode: ON — clipboard history is paused"
        } else {
            "Incognito Mode: OFF — clipboard history is active"
        };
        let incognito_btn = if self.incognito {
            widget::button::suggested(incognito_label)
                .on_press(Message::ToggleIncognito)
                .width(Length::Fill)
                .padding([10, 16])
        } else {
            widget::button::text(incognito_label)
                .on_press(Message::ToggleIncognito)
                .width(Length::Fill)
                .padding([10, 16])
        };
        content = content.push(incognito_btn);

        let lock_label = if self.config.clear_on_lock {
            "Clear history on screen lock: ON"
        } else {
            "Clear history on screen lock: OFF"
        };
        let lock_btn = if self.config.clear_on_lock {
            widget::button::suggested(lock_label)
                .on_press(Message::SettingToggleClearOnLock)
                .width(Length::Fill)
                .padding([10, 16])
        } else {
            widget::button::text(lock_label)
                .on_press(Message::SettingToggleClearOnLock)
                .width(Length::Fill)
                .padding([10, 16])
        };
        content = content.push(lock_btn);

        let encrypt_label = if self.config.encrypt_sensitive {
            "Encrypt sensitive items at rest: ON"
        } else {
            "Encrypt sensitive items at rest: OFF"
        };
        let encrypt_btn = if self.config.encrypt_sensitive {
            widget::button::suggested(encrypt_label)
                .on_press(Message::SettingToggleEncryptSensitive)
                .width(Length::Fill)
                .padding([10, 16])
        } else {
            widget::button::text(encrypt_label)
                .on_press(Message::SettingToggleEncryptSensitive)
                .width(Length::Fill)
                .padding([10, 16])
        };
        content = content.push(encrypt_btn);

        // ── History ──────────────────────────────────────────────────
        content = content.push(text("History").size(UI_SECTION_SIZE));

        // Max items buttons
        let max_items_options: &[(usize, &str)] = &[
            (50, "50"),
            (100, "100"),
            (200, "200"),
            (500, "500"),
            (1000, "1000"),
        ];
        let mut max_row = row().spacing(6).align_y(iced::Alignment::Center);
        max_row = max_row.push(text("Max items:").size(13).width(Length::Fixed(130.0)));
        for &(val, label) in max_items_options {
            let btn = if self.config.max_items == val {
                widget::button::suggested(label)
                    .on_press(Message::SettingMaxItems(val))
                    .padding([4, 8])
            } else {
                widget::button::text(label)
                    .on_press(Message::SettingMaxItems(val))
                    .padding([4, 8])
            };
            max_row = max_row.push(btn);
        }
        content = content.push(max_row);

        // Keep history (TTL) buttons — stored internally as seconds
        let age_options: &[(u64, &str)] = &[
            (7, "7d"),
            (30, "30d"),
            (90, "90d"),
            (365, "1yr"),
            (0, "Never"),
        ];
        let current_days = if self.config.ttl_seconds == 0 {
            0u64
        } else {
            self.config.ttl_seconds / 86400
        };
        let mut age_row = row().spacing(6).align_y(iced::Alignment::Center);
        age_row = age_row.push(text("Keep history:").size(13).width(Length::Fixed(130.0)));
        for &(val_days, label) in age_options {
            let btn = if current_days == val_days {
                widget::button::suggested(label)
                    .on_press(Message::SettingMaxAgeDays(val_days))
                    .padding([4, 8])
            } else {
                widget::button::text(label)
                    .on_press(Message::SettingMaxAgeDays(val_days))
                    .padding([4, 8])
            };
            age_row = age_row.push(btn);
        }
        content = content.push(age_row);

        // Dedup window buttons
        let dedup_options: &[(u64, &str)] =
            &[(0, "Off"), (2, "2s"), (5, "5s"), (10, "10s"), (30, "30s")];
        let mut dedup_row = row().spacing(6).align_y(iced::Alignment::Center);
        dedup_row = dedup_row.push(text("Dedup window:").size(13).width(Length::Fixed(130.0)));
        for &(val, label) in dedup_options {
            let btn = if self.config.dedup_window_seconds == val {
                widget::button::suggested(label)
                    .on_press(Message::SettingDedupWindow(val))
                    .padding([4, 8])
            } else {
                widget::button::text(label)
                    .on_press(Message::SettingDedupWindow(val))
                    .padding([4, 8])
            };
            dedup_row = dedup_row.push(btn);
        }
        content = content.push(dedup_row);

        let max_size_options: &[(usize, &str)] = &[
            (64 * 1024, "64KB"),
            (256 * 1024, "256KB"),
            (1024 * 1024, "1MB"),
            (2 * 1024 * 1024, "2MB"),
            (5 * 1024 * 1024, "5MB"),
        ];
        let mut size_row = row().spacing(6).align_y(iced::Alignment::Center);
        size_row = size_row.push(text("Max item size:").size(13).width(Length::Fixed(130.0)));
        for &(val, label) in max_size_options {
            let btn = if self.config.max_item_size == val {
                widget::button::suggested(label)
                    .on_press(Message::SettingMaxItemSize(val))
                    .padding([4, 8])
            } else {
                widget::button::text(label)
                    .on_press(Message::SettingMaxItemSize(val))
                    .padding([4, 8])
            };
            size_row = size_row.push(btn);
        }
        content = content.push(size_row);

        let cleanup_options: &[(u64, &str)] = &[
            (60, "1m"),
            (300, "5m"),
            (900, "15m"),
            (1800, "30m"),
            (3600, "1h"),
        ];
        let mut cleanup_row = row().spacing(6).align_y(iced::Alignment::Center);
        cleanup_row = cleanup_row.push(text("Cleanup every:").size(13).width(Length::Fixed(130.0)));
        for &(val, label) in cleanup_options {
            let btn = if self.config.cleanup_interval_seconds == val {
                widget::button::suggested(label)
                    .on_press(Message::SettingCleanupInterval(val))
                    .padding([4, 8])
            } else {
                widget::button::text(label)
                    .on_press(Message::SettingCleanupInterval(val))
                    .padding([4, 8])
            };
            cleanup_row = cleanup_row.push(btn);
        }
        content = content.push(cleanup_row);

        // ── Quick Paste ──────────────────────────────────────────────
        content = content.push(text("Quick Paste").size(UI_SECTION_SIZE));
        let paste_status = match &self.paste_backend {
            Some(backend) => format!("Backend: {backend}"),
            None => "No backend found (install wtype or ydotool)".to_string(),
        };
        content = content.push(text(paste_status).size(13));
        let qp_label = if self.quick_paste_enabled {
            "Quick Paste: ON — items will be typed directly"
        } else {
            "Quick Paste: OFF — items copied to clipboard"
        };
        let qp_btn = if self.quick_paste_enabled {
            widget::button::suggested(qp_label)
                .on_press(Message::ToggleQuickPaste)
                .width(Length::Fill)
                .padding([10, 16])
        } else {
            widget::button::text(qp_label)
                .on_press(Message::ToggleQuickPaste)
                .width(Length::Fill)
                .padding([10, 16])
        };
        content = content.push(qp_btn);

        // ── Never Store (MIME Denylist) ───────────────────────────────
        content = content.push(text("Never Store (MIME Denylist)").size(UI_SECTION_SIZE));
        content = content
            .push(text("MIME types to ignore (e.g. application/x-kde-cutselection):").size(12));

        let can_add_rule = !self.denylist_input.trim().is_empty();
        let add_rule_btn = if can_add_rule {
            widget::button::suggested("Add")
                .on_press(Message::SettingDenylistAdd(self.denylist_input.clone()))
                .padding([6, 14])
        } else {
            widget::button::suggested("Add").padding([6, 14])
        };
        let rule_row = row()
            .spacing(8)
            .push(
                text_input("Add MIME type...", &self.denylist_input)
                    .on_input(Message::SettingDenylistInput)
                    .on_submit(Message::SettingDenylistAdd)
                    .padding([6, 10])
                    .width(Length::Fill),
            )
            .push(add_rule_btn);
        content = content.push(rule_row);

        for (i, rule) in self.config.mime_denylist.iter().enumerate() {
            let entry_row = row()
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .push(text(rule.clone()).size(12).width(Length::Fill))
                .push(
                    widget::button::destructive("✕")
                        .on_press(Message::SettingDenylistRemove(i))
                        .padding([4, 8]),
                );
            content = content.push(entry_row);
        }

        content =
            content.push(text("Never Store (Content Pattern Denylist)").size(UI_SECTION_SIZE));
        content = content.push(
            text("Patterns support prefix (^otp), suffix (token$), or substring matching.")
                .size(12),
        );
        let can_add_content_rule = !self.content_denylist_input.trim().is_empty();
        let add_content_rule_btn = if can_add_content_rule {
            widget::button::suggested("Add")
                .on_press(Message::SettingContentDenylistAdd(
                    self.content_denylist_input.clone(),
                ))
                .padding([6, 14])
        } else {
            widget::button::suggested("Add").padding([6, 14])
        };
        let content_rule_row = row()
            .spacing(8)
            .push(
                text_input("Add content pattern...", &self.content_denylist_input)
                    .on_input(Message::SettingContentDenylistInput)
                    .on_submit(Message::SettingContentDenylistAdd)
                    .padding([6, 10])
                    .width(Length::Fill),
            )
            .push(add_content_rule_btn);
        content = content.push(content_rule_row);
        for (i, rule) in self.config.content_regex_denylist.iter().enumerate() {
            let entry_row = row()
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .push(text(rule.clone()).size(12).width(Length::Fill))
                .push(
                    widget::button::destructive("✕")
                        .on_press(Message::SettingContentDenylistRemove(i))
                        .padding([4, 8]),
                );
            content = content.push(entry_row);
        }

        // ── Data ─────────────────────────────────────────────────────
        content = content.push(text("Data").size(UI_SECTION_SIZE));
        content = content.push(
            widget::button::destructive("Clear All Clipboard History")
                .on_press(Message::ClearAll)
                .width(Length::Fill)
                .padding([10, 16]),
        );
        content = content.push(
            widget::button::text("Export Clipboard History")
                .on_press(Message::ExportData)
                .width(Length::Fill)
                .padding([10, 16]),
        );
        content = content.push(
            widget::button::text("Import Clipboard History")
                .on_press(Message::ImportData)
                .width(Length::Fill)
                .padding([10, 16]),
        );
        content = content.push(
            text(format!(
                "Export/import path: {}",
                self.config.data_dir.join("clipboard_export.json").display()
            ))
            .size(11),
        );

        // ── Statistics ───────────────────────────────────────────────
        if let Some(db) = &self.db {
            if let Ok(stats) = db.get_stats() {
                content = content.push(text("Statistics").size(UI_SECTION_SIZE));
                #[allow(clippy::cast_precision_loss)]
                let size_kb = stats.total_size_bytes as f64 / 1024.0;
                content = content.push(
                    text(format!(
                        "{} items total · {} pinned · {size_kb:.1} KB stored",
                        stats.total_items, stats.pinned_items,
                    ))
                    .size(13),
                );
            }
        }

        // ── Security Log ─────────────────────────────────────────────
        content = content.push(text("Security Log").size(UI_SECTION_SIZE));
        if let Some(db) = &self.db {
            if let Ok(events) = db.get_audit_log(10) {
                if events.is_empty() {
                    content = content.push(text("No security events recorded").size(12));
                } else {
                    for event in &events {
                        let time = crate::format_time_ago(event.timestamp);
                        let detail = event.details.as_deref().unwrap_or("");
                        let line = if detail.is_empty() {
                            format!("• {} — {time}", event.event_kind)
                        } else {
                            format!("• {} — {detail} ({time})", event.event_kind)
                        };
                        content = content.push(text(line).size(12));
                    }
                }
            }
        }

        // ── Keyboard Shortcuts ───────────────────────────────────────
        content = content.push(text("Keyboard Shortcuts").size(UI_SECTION_SIZE));
        content = content.push(
            text(format!(
                "Global shortcut: {}",
                self.config.keyboard_shortcut
            ))
            .size(13),
        );
        let shortcuts = [
            ("↑ / ↓ / ← / →", "Navigate pickers"),
            ("Home / End", "Jump to first / last"),
            ("PgUp / PgDn", "Fast page jump"),
            ("Enter", "Copy to clipboard"),
            ("Del / Ctrl+D", "Delete selected"),
            ("Ctrl+1..9", "Quick copy #1-9"),
            ("Ctrl+Tab", "Next tab"),
            ("Esc", "Close applet"),
        ];
        for (key, desc) in &shortcuts {
            let shortcut_row = row()
                .spacing(12)
                .push(text(*key).size(12).width(Length::Fixed(100.0)))
                .push(text(*desc).size(12));
            content = content.push(shortcut_row);
        }

        // ── About ────────────────────────────────────────────────────
        content = content.push(text(format!("About — v{version}")).size(UI_SECTION_SIZE));
        content = content.push(text("Author Clipboard — Native COSMIC clipboard manager").size(12));
        content = content.push(text(format!("Data: {}", self.config.data_dir.display())).size(12));
        content =
            content.push(text(format!("Config: {}", Config::config_path().display())).size(11));
        content = content.push(text("License: GPL-3.0").size(11));
        content = content.push(text("https://github.com/namikofficial/author-clipboard").size(11));

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update_title(&mut self) -> Task<Message> {
        let title = String::from("Clipboard Manager");
        self.set_header_title(title.clone());
        self.set_window_title(title)
    }

    /// Scroll the clipboard list to keep the selected item visible.
    fn scroll_to_selected(&self) -> Task<Message> {
        if let Some(idx) = self.selected_index {
            let len = self.items.len();
            if len <= 1 {
                return cosmic::iced_widget::scrollable::snap_to(
                    clipboard_scroll_id(),
                    cosmic::iced_widget::scrollable::RelativeOffset::START,
                );
            }
            #[allow(clippy::cast_precision_loss)]
            let ratio = idx as f32 / (len.saturating_sub(1)) as f32;
            cosmic::iced_widget::scrollable::snap_to(
                clipboard_scroll_id(),
                cosmic::iced_widget::scrollable::RelativeOffset { x: 0.0, y: ratio },
            )
        } else {
            Task::none()
        }
    }

    /// Scroll the emoji grid to keep the selected item visible.
    fn scroll_emoji_to_selected(&self) -> Task<Message> {
        if let Some(idx) = self.emoji_selected_idx {
            let len = self.filtered_emojis().len();
            if len <= 1 {
                return cosmic::iced_widget::scrollable::snap_to(
                    emoji_scroll_id(),
                    cosmic::iced_widget::scrollable::RelativeOffset::START,
                );
            }
            let idx = idx.min(len - 1);
            #[allow(clippy::cast_precision_loss)]
            let ratio = idx as f32 / (len.saturating_sub(1)) as f32;
            cosmic::iced_widget::scrollable::snap_to(
                emoji_scroll_id(),
                cosmic::iced_widget::scrollable::RelativeOffset { x: 0.0, y: ratio },
            )
        } else {
            Task::none()
        }
    }

    /// Scroll the symbol grid to keep the selected item visible.
    fn scroll_symbol_to_selected(&self) -> Task<Message> {
        if let Some(idx) = self.symbol_selected_idx {
            let len = self.filtered_symbols().len();
            if len <= 1 {
                return cosmic::iced_widget::scrollable::snap_to(
                    symbol_scroll_id(),
                    cosmic::iced_widget::scrollable::RelativeOffset::START,
                );
            }
            let idx = idx.min(len - 1);
            #[allow(clippy::cast_precision_loss)]
            let ratio = idx as f32 / (len.saturating_sub(1)) as f32;
            cosmic::iced_widget::scrollable::snap_to(
                symbol_scroll_id(),
                cosmic::iced_widget::scrollable::RelativeOffset { x: 0.0, y: ratio },
            )
        } else {
            Task::none()
        }
    }

    /// Scroll the kaomoji list to keep the selected item visible.
    fn scroll_kaomoji_to_selected(&self) -> Task<Message> {
        if let Some(idx) = self.kaomoji_selected_idx {
            let len = self.filtered_kaomoji().len();
            if len <= 1 {
                return cosmic::iced_widget::scrollable::snap_to(
                    kaomoji_scroll_id(),
                    cosmic::iced_widget::scrollable::RelativeOffset::START,
                );
            }
            let idx = idx.min(len - 1);
            #[allow(clippy::cast_precision_loss)]
            let ratio = idx as f32 / (len.saturating_sub(1)) as f32;
            cosmic::iced_widget::scrollable::snap_to(
                kaomoji_scroll_id(),
                cosmic::iced_widget::scrollable::RelativeOffset { x: 0.0, y: ratio },
            )
        } else {
            Task::none()
        }
    }

    /// Cycle through tabs forward or backward.
    fn cycle_tab(&mut self, forward: bool) {
        const TAB_ORDER: [AppTab; 6] = [
            AppTab::Clipboard,
            AppTab::Emoji,
            AppTab::Symbols,
            AppTab::Kaomoji,
            AppTab::Snippets,
            AppTab::Settings,
        ];
        let current = TAB_ORDER
            .iter()
            .position(|t| *t == self.active_tab)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % TAB_ORDER.len()
        } else {
            (current + TAB_ORDER.len() - 1) % TAB_ORDER.len()
        };
        let next_tab = TAB_ORDER[next];
        // Collect matching entity first to avoid borrow conflict
        let target_entity = self
            .tab_model
            .iter()
            .find(|&entity| self.tab_model.data::<AppTab>(entity) == Some(&next_tab));
        if let Some(entity) = target_entity {
            self.tab_model.activate(entity);
        }
        self.active_tab = next_tab;
        self.search_query.clear();
        self.selected_index = None;
        self.emoji_selected_idx = None;
        self.symbol_selected_idx = None;
        self.kaomoji_selected_idx = None;
    }

    /// Copy the currently selected item and exit.
    fn copy_selected_item(&mut self) -> Task<Message> {
        match self.active_tab {
            AppTab::Clipboard => {
                if let Some(index) = self.selected_index {
                    if let Some(item) = self.items.get(index) {
                        let result = if item.is_image() {
                            set_clipboard_image(
                                &image_store::image_path(&self.config.data_dir, &item.content),
                                &item.mime_type,
                            )
                        } else if item.is_html() {
                            set_clipboard_html(
                                &item.content,
                                item.plain_text.as_deref().unwrap_or(""),
                            )
                        } else {
                            set_clipboard_text(&item.content)
                        };
                        if result.is_ok() {
                            std::process::exit(0);
                        }
                    }
                }
            }
            AppTab::Emoji => {
                if let Some(idx) = self.emoji_selected_idx {
                    let emojis = self.filtered_emojis();
                    if let Some(&e) = emojis.get(idx) {
                        if set_clipboard_text(e).is_ok() {
                            std::process::exit(0);
                        }
                    }
                }
            }
            AppTab::Symbols => {
                if let Some(idx) = self.symbol_selected_idx {
                    let syms = self.filtered_symbols();
                    if let Some(&(s, _)) = syms.get(idx) {
                        if set_clipboard_text(s).is_ok() {
                            std::process::exit(0);
                        }
                    }
                }
            }
            AppTab::Kaomoji => {
                if let Some(idx) = self.kaomoji_selected_idx {
                    let items = self.filtered_kaomoji();
                    if let Some(&k) = items.get(idx) {
                        if set_clipboard_text(k).is_ok() {
                            std::process::exit(0);
                        }
                    }
                }
            }
            AppTab::Snippets => {
                if let Some(index) = self.selected_index {
                    if let Some(s) = self.snippets.get(index) {
                        if set_clipboard_text(&s.content).is_ok() {
                            std::process::exit(0);
                        }
                    }
                }
            }
            AppTab::Settings => {}
        }
        Task::none()
    }
}

// ── Utility functions ─────────────────────────────────────────────────

/// Check if the clipboard daemon process is running.
fn check_daemon_running() -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", "author-clipboard-daemon"])
        .output()
        .is_ok_and(|o| o.status.success())
}

fn truncate_content(content: &str, max_len: usize) -> String {
    let single_line = content.replace('\n', " ").replace('\r', "");
    if single_line.len() > max_len {
        format!("{}…", &single_line[..max_len])
    } else {
        single_line
    }
}

fn format_time_ago(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(timestamp);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let m = duration.num_minutes();
        format!("{m}m ago")
    } else if duration.num_hours() < 24 {
        let h = duration.num_hours();
        format!("{h}h ago")
    } else {
        let d = duration.num_days();
        format!("{d}d ago")
    }
}

fn set_clipboard_text(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }

    child.wait()?;
    Ok(())
}

fn set_clipboard_image(
    path: &std::path::Path,
    mime_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let data = std::fs::read(path).map_err(|e| format!("Failed to read image: {e}"))?;

    let mut child = Command::new("wl-copy")
        .args(["--type", mime_type])
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&data)?;
    }

    child.wait()?;
    Ok(())
}

fn set_clipboard_html(html: &str, plain_text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Set HTML content as the primary type
    let mut child = Command::new("wl-copy")
        .args(["--type", "text/html"])
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(html.as_bytes())?;
    }

    child.wait()?;

    // Also set plain text as fallback (best effort)
    if !plain_text.is_empty() {
        let _ = set_clipboard_text(plain_text);
    }

    Ok(())
}
