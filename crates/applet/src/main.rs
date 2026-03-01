//! author-clipboard: COSMIC clipboard manager applet
//!
//! A graphical interface for browsing and selecting from clipboard history,
//! with emoji and symbol pickers.

mod emoji;
mod kaomoji;
mod symbols;

use author_clipboard_shared::config::Config;
use author_clipboard_shared::image_store;
use author_clipboard_shared::types::{AuditEventKind, ClipboardItem};
use author_clipboard_shared::Database;
use cosmic::app::{Core, Settings, Task};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::keyboard::Key;
use cosmic::iced::{Length, Size, Subscription};
use cosmic::widget::{self, column, container, row, scrollable, text, text_input};
use cosmic::{executor, iced, ApplicationExt, Element};
use tracing::{error, info, warn};

const APP_ID: &str = "com.namikofficial.author-clipboard";
const SEARCH_INPUT_ID: fn() -> widget::Id = || widget::Id::new("search-input");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("author-clipboard applet starting...");

    let settings = Settings::default()
        .size(Size::new(480.0, 640.0))
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
    incognito: bool,
}

// ── Messages ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Message {
    Tick,
    TabSelected(widget::segmented_button::Entity),
    SearchChanged(String),
    CopyItem(i64),
    CopyText(String),
    TogglePin(i64),
    DeleteItem(i64),
    ClearAll,
    SelectItem(usize),
    SelectNext,
    SelectPrevious,
    CopySelected,
    EmojiCategory(usize),
    SymbolCategory(usize),
    KaomojiCategory(usize),
    ToggleIncognito,
    ExportData,
    ImportData,
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
        let config = Config::default();

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

        let tab_model = widget::segmented_button::Model::builder()
            .insert(|b| b.text("📋 Clipboard").data(AppTab::Clipboard).activate())
            .insert(|b| b.text("😀 Emoji").data(AppTab::Emoji))
            .insert(|b| b.text("🔣 Symbols").data(AppTab::Symbols))
            .insert(|b| b.text("顔 Kaomoji").data(AppTab::Kaomoji))
            .insert(|b| b.text("⚙️ Settings").data(AppTab::Settings))
            .build();

        let incognito = config.is_incognito();

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
            incognito,
        };

        let command = app.update_title();

        (app, command)
    }

    fn on_escape(&mut self) -> Task<Self::Message> {
        if !self.search_query.is_empty() {
            self.search_query.clear();
            if self.active_tab == AppTab::Clipboard {
                self.refresh_items();
            }
            self.selected_index = None;
        }
        Task::none()
    }

    fn on_search(&mut self) -> Task<Self::Message> {
        cosmic::widget::text_input::focus(SEARCH_INPUT_ID())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let keyboard = cosmic::iced::keyboard::on_key_press(|key, _modifiers| match key {
            Key::Named(iced::keyboard::key::Named::ArrowDown) => Some(Message::SelectNext),
            Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(Message::SelectPrevious),
            Key::Named(iced::keyboard::key::Named::Enter) => Some(Message::CopySelected),
            _ => None,
        });

        let tick =
            cosmic::iced::time::every(std::time::Duration::from_secs(2)).map(|_| Message::Tick);

        Subscription::batch([keyboard, tick])
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick => {
                if self.active_tab == AppTab::Clipboard {
                    self.refresh_items();
                }
            }

            Message::TabSelected(entity) => {
                self.tab_model.activate(entity);
                if let Some(&tab) = self.tab_model.data::<AppTab>(entity) {
                    self.active_tab = tab;
                    self.search_query.clear();
                    self.selected_index = None;
                }
            }

            Message::SearchChanged(query) => {
                self.search_query = query;
                if self.active_tab == AppTab::Clipboard {
                    self.refresh_items();
                }
                self.selected_index = None;
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
                        Ok(()) => info!("Copied item {} to clipboard", id),
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
                                AppTab::Clipboard | AppTab::Settings => None,
                            };
                            if let Some(cat) = category {
                                if let Err(e) = db.record_usage(cat, &content) {
                                    warn!("Failed to record usage: {e}");
                                }
                            }
                        }
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

            Message::SelectItem(index) => {
                self.selected_index = Some(index);
            }

            Message::SelectNext => {
                let len = self.visible_item_count();
                if len > 0 {
                    self.selected_index = Some(match self.selected_index {
                        Some(i) if i + 1 < len => i + 1,
                        _ => 0,
                    });
                }
            }

            Message::SelectPrevious => {
                let len = self.visible_item_count();
                if len > 0 {
                    self.selected_index = Some(match self.selected_index {
                        Some(0) | None => len.saturating_sub(1),
                        Some(i) => i - 1,
                    });
                }
            }

            Message::CopySelected => {
                if let Some(index) = self.selected_index {
                    match self.active_tab {
                        AppTab::Clipboard => {
                            if let Some(item) = self.items.get(index) {
                                let result = if item.is_image() {
                                    set_clipboard_image(
                                        &image_store::image_path(
                                            &self.config.data_dir,
                                            &item.content,
                                        ),
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
                                if let Err(e) = result {
                                    warn!("Failed to copy: {e}");
                                }
                            }
                        }
                        AppTab::Emoji => {
                            let emojis = self.filtered_emojis();
                            if let Some(&e) = emojis.get(index) {
                                let _ = set_clipboard_text(e);
                            }
                        }
                        AppTab::Symbols => {
                            let syms = self.filtered_symbols();
                            if let Some(&(s, _)) = syms.get(index) {
                                let _ = set_clipboard_text(s);
                            }
                        }
                        AppTab::Kaomoji => {
                            let items = self.filtered_kaomoji();
                            if let Some(&k) = items.get(index) {
                                let _ = set_clipboard_text(k);
                            }
                        }
                        AppTab::Settings => {}
                    }
                }
            }

            Message::EmojiCategory(idx) => {
                self.emoji_category_idx = idx;
                self.selected_index = None;
            }

            Message::SymbolCategory(idx) => {
                self.symbol_category_idx = idx;
                self.selected_index = None;
            }

            Message::KaomojiCategory(idx) => {
                self.kaomoji_category_idx = idx;
                self.selected_index = None;
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
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let tab_bar =
            widget::tab_bar::horizontal(&self.tab_model).on_activate(Message::TabSelected);

        let search_placeholder = match self.active_tab {
            AppTab::Clipboard => "Search clipboard history...",
            AppTab::Emoji => "Search emoji...",
            AppTab::Symbols => "Search symbols...",
            AppTab::Kaomoji => "Search kaomoji...",
            AppTab::Settings => "Search settings...",
        };

        let search_bar = text_input(search_placeholder, &self.search_query)
            .on_input(Message::SearchChanged)
            .id(SEARCH_INPUT_ID())
            .width(Length::Fill)
            .padding(8);

        let incognito_btn = {
            let label = if self.incognito { "🕶️" } else { "👁️" };
            let btn = widget::button::text(label).padding([6, 8]);
            btn.on_press(Message::ToggleIncognito)
        };

        let header = match self.active_tab {
            AppTab::Clipboard => row()
                .spacing(8)
                .push(search_bar)
                .push(incognito_btn)
                .push(
                    widget::button::destructive("Clear")
                        .on_press(Message::ClearAll)
                        .padding([6, 12]),
                )
                .align_y(iced::Alignment::Center),
            _ => row()
                .spacing(8)
                .push(search_bar)
                .push(incognito_btn)
                .align_y(iced::Alignment::Center),
        };

        let tab_content: Element<'_, Message> = match self.active_tab {
            AppTab::Clipboard => self.view_clipboard(),
            AppTab::Emoji => self.view_emoji(),
            AppTab::Symbols => self.view_symbols(),
            AppTab::Kaomoji => self.view_kaomoji(),
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
            AppTab::Settings => String::from("Settings"),
        };

        let mut status_parts = vec![status_text];
        if self.incognito {
            status_parts.push("🕶️ Incognito".to_string());
        }
        let full_status = status_parts.join(" • ");

        let status_bar = container(text(full_status).size(12.0))
            .width(Length::Fill)
            .padding([4, 8]);

        let content = column()
            .spacing(8)
            .padding(12)
            .push(tab_bar)
            .push(header)
            .push(tab_content)
            .push(status_bar);

        Element::from(content)
    }
}

// ── Helper methods ────────────────────────────────────────────────────

impl App {
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

    fn visible_item_count(&self) -> usize {
        match self.active_tab {
            AppTab::Clipboard => self.items.len(),
            AppTab::Emoji => self.filtered_emojis().len(),
            AppTab::Symbols => self.filtered_symbols().len(),
            AppTab::Kaomoji => self.filtered_kaomoji().len(),
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
        if self.items.is_empty() {
            let empty_msg = if self.search_query.is_empty() {
                "No clipboard items yet.\nCopy something to get started!"
            } else {
                "No items match your search."
            };

            container(text(empty_msg).size(14.0).align_x(Horizontal::Center))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .into()
        } else {
            let mut list = column().spacing(4).padding([0, 4]);

            for (index, item) in self.items.iter().enumerate() {
                list = list.push(self.clipboard_item_row(item, index));
            }

            scrollable(list)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn clipboard_item_row(&self, item: &ClipboardItem, index: usize) -> Element<'_, Message> {
        let time_ago = format_time_ago(item.timestamp);

        let pin_icon = if item.pinned { "📌" } else { "○" };
        let pin_btn = widget::button::text(pin_icon)
            .on_press(Message::TogglePin(item.id))
            .padding([4, 8]);

        let delete_btn = widget::button::text("✕")
            .on_press(Message::DeleteItem(item.id))
            .padding([4, 8]);

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
            col = col.push(text(format!("🖼️ Image ({})", &item.mime_type)).size(12.0));
            col.push(text(time_ago).size(11.0))
        } else if item.is_html() {
            let preview_text = item.plain_text.as_deref().unwrap_or(&item.content);
            let preview = truncate_content(preview_text, 120);
            let mut col = column().spacing(2);
            col = col.push(text(preview).size(13.0));
            col = col.push(text("📄 HTML content").size(11.0));
            col.push(text(time_ago).size(11.0))
        } else if item.is_files() {
            let names = item.file_names();
            let file_count = names.len();
            let preview = if names.is_empty() {
                "No files".to_string()
            } else if names.len() <= 3 {
                names.join(", ")
            } else {
                format!("{}, {} and {} more", names[0], names[1], file_count - 2)
            };
            let mut col = column().spacing(2);
            col = col.push(text(preview).size(13.0));
            col = col.push(text(format!("📁 {file_count} file(s)")).size(11.0));
            col.push(text(time_ago).size(11.0))
        } else {
            let preview = truncate_content(&item.content, 120);
            let mut col = column().spacing(2).push(text(preview).size(13.0));
            if item.sensitive {
                col = col.push(text("⚠️ Sensitive content").size(11.0));
            }
            col.push(text(time_ago).size(11.0))
        };

        let row_content = row()
            .spacing(8)
            .push(pin_btn)
            .push(container(content_col).width(Length::Fill))
            .push(delete_btn)
            .align_y(iced::Alignment::Center);

        let item_btn = widget::button::custom(row_content)
            .width(Length::Fill)
            .padding([8, 8])
            .on_press(Message::CopyItem(item.id));

        widget::mouse_area(item_btn)
            .on_press(Message::SelectItem(index))
            .into()
    }

    // ── Emoji tab view ────────────────────────────────────────────────

    fn view_emoji(&self) -> Element<'_, Message> {
        let mut content = column().spacing(8);

        // Category buttons (only when not searching)
        if self.search_query.is_empty() {
            let mut cat_row = row().spacing(4);
            for (idx, cat) in emoji::CATEGORIES.iter().enumerate() {
                let btn = if idx == self.emoji_category_idx {
                    widget::button::suggested(cat.icon)
                        .on_press(Message::EmojiCategory(idx))
                        .padding([4, 8])
                } else {
                    widget::button::text(cat.icon)
                        .on_press(Message::EmojiCategory(idx))
                        .padding([4, 8])
                };
                cat_row = cat_row.push(btn);
            }
            content = content.push(widget::scrollable::horizontal(cat_row));

            // Category label
            let cat = &emoji::CATEGORIES[self.emoji_category_idx];
            content = content.push(text(cat.name).size(13.0));
        }

        // Emoji grid
        let emojis: Vec<&str> = if self.search_query.is_empty() {
            let cat = &emoji::CATEGORIES[self.emoji_category_idx];
            cat.emojis.to_vec()
        } else {
            emoji::search(&self.search_query)
        };

        let mut grid = column().spacing(4);
        let cols = 8;
        for chunk in emojis.chunks(cols) {
            let mut grid_row = row().spacing(4);
            for &emoji_char in chunk {
                let btn = widget::button::text(emoji_char)
                    .on_press(Message::CopyText(emoji_char.to_string()))
                    .padding([6, 8]);
                grid_row = grid_row.push(btn);
            }
            grid = grid.push(grid_row);
        }

        content = content.push(scrollable(grid).width(Length::Fill).height(Length::Fill));

        content.into()
    }

    // ── Symbols tab view ──────────────────────────────────────────────

    fn view_symbols(&self) -> Element<'_, Message> {
        let mut content = column().spacing(8);

        // Category buttons (only when not searching)
        if self.search_query.is_empty() {
            let mut cat_row = row().spacing(4);
            for (idx, cat) in symbols::CATEGORIES.iter().enumerate() {
                let btn = if idx == self.symbol_category_idx {
                    widget::button::suggested(cat.icon)
                        .on_press(Message::SymbolCategory(idx))
                        .padding([4, 8])
                } else {
                    widget::button::text(cat.icon)
                        .on_press(Message::SymbolCategory(idx))
                        .padding([4, 8])
                };
                cat_row = cat_row.push(btn);
            }
            content = content.push(widget::scrollable::horizontal(cat_row));

            // Category label
            let cat = &symbols::CATEGORIES[self.symbol_category_idx];
            content = content.push(text(cat.name).size(13.0));
        }

        // Symbol grid with descriptions
        let syms = if self.search_query.is_empty() {
            let cat = &symbols::CATEGORIES[self.symbol_category_idx];
            cat.symbols.to_vec()
        } else {
            symbols::search(&self.search_query)
        };

        let mut list = column().spacing(4);
        let cols = 6;
        for chunk in syms.chunks(cols) {
            let mut grid_row = row().spacing(4);
            for &(sym, desc) in chunk {
                let btn = widget::tooltip(
                    widget::button::text(sym)
                        .on_press(Message::CopyText(sym.to_string()))
                        .padding([8, 12]),
                    text(desc).size(12.0),
                    widget::tooltip::Position::Bottom,
                );
                grid_row = grid_row.push(btn);
            }
            list = list.push(grid_row);
        }

        content = content.push(scrollable(list).width(Length::Fill).height(Length::Fill));

        content.into()
    }

    // ── Kaomoji tab view ──────────────────────────────────────────────

    fn view_kaomoji(&self) -> Element<'_, Message> {
        let mut content = column().spacing(8).width(Length::Fill);

        // Category selector (horizontal scrolling row)
        if self.search_query.is_empty() {
            let mut cat_row = row().spacing(4);
            for (idx, cat) in kaomoji::CATEGORIES.iter().enumerate() {
                let label = format!("{} {}", cat.icon, cat.name);
                let btn = if idx == self.kaomoji_category_idx {
                    widget::button::suggested(label)
                        .on_press(Message::KaomojiCategory(idx))
                        .padding([4, 8])
                } else {
                    widget::button::text(label)
                        .on_press(Message::KaomojiCategory(idx))
                        .padding([4, 8])
                };
                cat_row = cat_row.push(btn);
            }
            content = content.push(widget::scrollable::horizontal(cat_row));

            let cat = &kaomoji::CATEGORIES[self.kaomoji_category_idx];
            content = content.push(text(cat.name).size(13.0));
        }

        // Kaomoji list (vertical, since they're wider than emoji)
        let items = self.filtered_kaomoji();
        let mut list = column().spacing(4).width(Length::Fill);
        for (idx, &kaomoji_str) in items.iter().enumerate() {
            let is_selected = self.selected_index == Some(idx);
            let btn = if is_selected {
                widget::button::suggested(kaomoji_str)
                    .on_press(Message::CopyText(kaomoji_str.to_string()))
                    .width(Length::Fill)
                    .padding([6, 12])
            } else {
                widget::button::text(kaomoji_str)
                    .on_press(Message::CopyText(kaomoji_str.to_string()))
                    .width(Length::Fill)
                    .padding([6, 12])
            };
            list = list.push(btn);
        }

        content = content.push(scrollable(list).width(Length::Fill).height(Length::Fill));

        content.into()
    }

    // ── Settings tab view ─────────────────────────────────────────────

    fn view_settings(&self) -> Element<'_, Message> {
        let mut content = column().spacing(12).width(Length::Fill);

        // Incognito toggle
        let incognito_label = if self.incognito {
            "🕶️ Incognito Mode: ON — clipboard history is paused"
        } else {
            "👁️ Incognito Mode: OFF — clipboard history is active"
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
        content = content.push(text("Privacy").size(16.0));
        content = content.push(incognito_btn);

        // Clear all button
        content = content.push(text("Data").size(16.0));
        content = content.push(
            widget::button::destructive("🗑️ Clear All Clipboard History")
                .on_press(Message::ClearAll)
                .width(Length::Fill)
                .padding([10, 16]),
        );
        content = content.push(
            widget::button::text("📤 Export Clipboard History")
                .on_press(Message::ExportData)
                .width(Length::Fill)
                .padding([10, 16]),
        );
        content = content.push(
            widget::button::text("📥 Import Clipboard History")
                .on_press(Message::ImportData)
                .width(Length::Fill)
                .padding([10, 16]),
        );
        content = content.push(
            text(format!(
                "Export/import path: {}",
                self.config.data_dir.join("clipboard_export.json").display()
            ))
            .size(11.0),
        );

        // Stats
        if let Some(db) = &self.db {
            if let Ok(stats) = db.get_stats() {
                content = content.push(text("Statistics").size(16.0));
                #[allow(clippy::cast_precision_loss)]
                let size_kb = stats.total_size_bytes as f64 / 1024.0;
                let stats_text = format!(
                    "📊 {} items total • {} pinned • {size_kb:.1} KB stored",
                    stats.total_items, stats.pinned_items,
                );
                content = content.push(text(stats_text).size(13.0));
            }
        }

        // Keyboard shortcut
        content = content.push(text("Keyboard").size(16.0));
        content = content
            .push(text(format!("⌨️ Shortcut: {}", self.config.keyboard_shortcut)).size(13.0));
        content = content
            .push(text("Press the shortcut to quickly open the clipboard picker").size(12.0));

        // Security audit log
        content = content.push(text("Security Log").size(16.0));
        if let Some(db) = &self.db {
            if let Ok(events) = db.get_audit_log(10) {
                if events.is_empty() {
                    content = content.push(text("No security events recorded").size(12.0));
                } else {
                    for event in &events {
                        let time = crate::format_time_ago(event.timestamp);
                        let detail = event.details.as_deref().unwrap_or("");
                        let line = if detail.is_empty() {
                            format!("• {} — {time}", event.event_kind)
                        } else {
                            format!("• {} — {detail} ({time})", event.event_kind)
                        };
                        content = content.push(text(line).size(12.0));
                    }
                }
            }
        }

        // Info
        content = content.push(text("About").size(16.0));
        content = content.push(text("Author Clipboard v0.1.0").size(13.0));
        content =
            content.push(text("COSMIC clipboard manager with emoji & symbol pickers").size(12.0));
        content =
            content.push(text(format!("Data: {}", self.config.data_dir.display())).size(12.0));

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
}

// ── Utility functions ─────────────────────────────────────────────────

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
