//! author-clipboard: COSMIC clipboard manager applet
//!
//! A graphical interface for browsing and selecting from clipboard history.

use author_clipboard_shared::config::Config;
use author_clipboard_shared::types::ClipboardItem;
use author_clipboard_shared::Database;
use cosmic::app::{Core, Settings, Task};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::{Length, Size};
use cosmic::widget::{self, column, container, row, scrollable, text, text_input};
use cosmic::{executor, iced, ApplicationExt, Element};
use tracing::{error, info, warn};

const APP_ID: &str = "com.namik.author-clipboard";
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
        .size(Size::new(450.0, 600.0))
        .debug(false);

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

// ── Application state ─────────────────────────────────────────────────

struct App {
    core: Core,
    db: Option<Database>,
    config: Config,
    items: Vec<ClipboardItem>,
    search_query: String,
    selected_index: Option<usize>,
}

// ── Messages ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Message {
    SearchChanged(String),
    CopyItem(i64),
    TogglePin(i64),
    DeleteItem(i64),
    ClearAll,
    SelectItem(usize),
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

        let mut app = App {
            core,
            db,
            config,
            items,
            search_query: String::new(),
            selected_index: None,
        };

        let command = app.update_title();

        (app, command)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.refresh_items();
                self.selected_index = None;
            }

            Message::CopyItem(id) => {
                if let Some(item) = self.items.iter().find(|i| i.id == id) {
                    // Set clipboard via wl-copy (simple approach for now)
                    if let Err(e) = set_clipboard(&item.content) {
                        warn!("Failed to set clipboard: {e}");
                    } else {
                        info!("Copied item {} to clipboard", id);
                    }
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
                    }
                    self.refresh_items();
                }
            }

            Message::ClearAll => {
                if let Some(db) = &self.db {
                    if let Err(e) = db.clear_unpinned() {
                        warn!("Failed to clear items: {e}");
                    }
                    self.refresh_items();
                }
            }

            Message::SelectItem(index) => {
                self.selected_index = Some(index);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let search_bar = text_input("Search clipboard history...", &self.search_query)
            .on_input(Message::SearchChanged)
            .id(SEARCH_INPUT_ID())
            .width(Length::Fill)
            .padding(8);

        let header_row = row()
            .spacing(8)
            .push(search_bar)
            .push(
                widget::button::destructive("Clear")
                    .on_press(Message::ClearAll)
                    .padding([6, 12]),
            )
            .align_y(iced::Alignment::Center);

        let items_list: Element<'_, Message> = if self.items.is_empty() {
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
                list = list.push(Self::item_row(item, index));
            }

            scrollable(list)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let status_bar = {
            let count = self.items.len();
            let pinned = self.items.iter().filter(|i| i.pinned).count();
            let status = if pinned > 0 {
                format!("{count} items ({pinned} pinned)")
            } else {
                format!("{count} items")
            };
            container(text(status).size(12.0))
                .width(Length::Fill)
                .padding([4, 8])
        };

        let content = column()
            .spacing(8)
            .padding(12)
            .push(header_row)
            .push(items_list)
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

    fn item_row(item: &ClipboardItem, index: usize) -> Element<'_, Message> {
        let preview = truncate_content(&item.content, 120);
        let time_ago = format_time_ago(item.timestamp);

        let pin_icon = if item.pinned { "📌" } else { "○" };
        let pin_btn = widget::button::text(pin_icon)
            .on_press(Message::TogglePin(item.id))
            .padding([4, 8]);

        let delete_btn = widget::button::text("✕")
            .on_press(Message::DeleteItem(item.id))
            .padding([4, 8]);

        let content_col = column()
            .spacing(2)
            .push(text(preview).size(13.0))
            .push(text(time_ago).size(11.0));

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

fn set_clipboard(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }

    child.wait()?;
    Ok(())
}
