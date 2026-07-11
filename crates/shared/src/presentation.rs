//! Pure, bounded clipboard presentation classification for UI/CLI/MCP adapters.

use serde::{Deserialize, Serialize};

use crate::classify::{classify, ContentClass};
use crate::file_handler::parse_uri_list;
use crate::types::{ClipboardItem, ContentType};

const CLASSIFY_LIMIT: usize = 64 * 1024;
const PREVIEW_CHARS: usize = 240;

/// A local-only, serializable description of how clipboard content is shown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentPresentation {
    /// Ordinary text.
    Text { preview: String },
    /// HTTP(S) URL with a display domain.
    Url { preview: String, domain: String },
    /// CSS-style hexadecimal color.
    Color { hex: String, rgba: (u8, u8, u8, u8) },
    /// Source code, SQL, or a shell command.
    Code {
        preview: String,
        language_hint: Option<String>,
    },
    /// Valid JSON and its root shape.
    Json { preview: String, root_kind: String },
    /// HTML rendered as a safe plain-text fallback.
    Html { text_preview: String },
    /// Stored image metadata.
    Image { path: String, mime: String },
    /// File URI metadata.
    File {
        name: String,
        path_hint: String,
        exists: bool,
    },
    /// Redacted sensitive content.
    Secret { redacted_preview: String },
    /// Unsupported or oversized content.
    Unknown { preview: String },
}

impl ContentPresentation {
    /// Compact badge label used by result cards.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Url { .. } => "URL",
            Self::Color { .. } => "color",
            Self::Code { .. } => "code",
            Self::Json { .. } => "JSON",
            Self::Html { .. } => "HTML",
            Self::Image { .. } => "image",
            Self::File { .. } => "file",
            Self::Secret { .. } => "secret",
            Self::Unknown { .. } => "unknown",
        }
    }
}

/// Classify an item with privacy checks before all content parsing.
pub fn present(item: &ClipboardItem) -> ContentPresentation {
    if item.sensitive || item.encrypted {
        return ContentPresentation::Secret {
            redacted_preview: item
                .redacted_preview
                .clone()
                .unwrap_or_else(|| "Sensitive item".to_string()),
        };
    }
    if item.content.len() > CLASSIFY_LIMIT {
        return ContentPresentation::Unknown {
            preview: bounded(&item.content),
        };
    }
    if matches!(classify(&item.content), ContentClass::Secret) {
        return ContentPresentation::Secret {
            redacted_preview: item
                .redacted_preview
                .clone()
                .unwrap_or_else(|| "Sensitive item".to_string()),
        };
    }
    match item.content_type {
        ContentType::Image => ContentPresentation::Image {
            path: item.content.clone(),
            mime: item.mime_type.clone(),
        },
        ContentType::Html => ContentPresentation::Html {
            text_preview: bounded(item.plain_text.as_deref().unwrap_or("HTML content")),
        },
        ContentType::Files => {
            let file = parse_uri_list(&item.content).into_iter().next();
            file.map_or_else(
                || ContentPresentation::Unknown {
                    preview: bounded(&item.content),
                },
                |file| ContentPresentation::File {
                    name: file.name,
                    path_hint: file.path.display().to_string(),
                    exists: file.exists,
                },
            )
        }
        ContentType::Text => present_text(&item.content),
    }
}

fn present_text(text: &str) -> ContentPresentation {
    let trimmed = text.trim();
    if let Some((hex, rgba)) = parse_color(trimmed) {
        return ContentPresentation::Color { hex, rgba };
    }
    match classify(trimmed) {
        ContentClass::Url => ContentPresentation::Url {
            preview: bounded(trimmed),
            domain: url_domain(trimmed),
        },
        ContentClass::Json => {
            let value = serde_json::from_str::<serde_json::Value>(trimmed);
            value.map_or_else(
                |_| ContentPresentation::Text {
                    preview: bounded(trimmed),
                },
                |value| ContentPresentation::Json {
                    preview: bounded(trimmed),
                    root_kind: match value {
                        serde_json::Value::Object(_) => "object",
                        serde_json::Value::Array(_) => "array",
                        _ => "scalar",
                    }
                    .to_string(),
                },
            )
        }
        ContentClass::Code | ContentClass::Command | ContentClass::Sql => {
            ContentPresentation::Code {
                preview: bounded(trimmed),
                language_hint: match classify(trimmed) {
                    ContentClass::Sql => Some("sql".to_string()),
                    ContentClass::Command => Some("shell".to_string()),
                    _ => None,
                },
            }
        }
        _ => ContentPresentation::Text {
            preview: bounded(trimmed),
        },
    }
}

fn bounded(value: &str) -> String {
    let mut result: String = value.chars().take(PREVIEW_CHARS).collect();
    if value.chars().count() > PREVIEW_CHARS {
        result.push('…');
    }
    result
}

fn url_domain(value: &str) -> String {
    value
        .split_once("://")
        .map_or(value, |(_, rest)| rest)
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_string()
}

fn parse_color(value: &str) -> Option<(String, (u8, u8, u8, u8))> {
    let hex = value.strip_prefix('#')?;
    let expanded = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 | 8 => hex.to_string(),
        _ => return None,
    };
    if !expanded.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |range| u8::from_str_radix(&expanded[range], 16).ok();
    Some((
        format!("#{expanded}"),
        (
            byte(0..2)?,
            byte(2..4)?,
            byte(4..6)?,
            if expanded.len() == 8 {
                byte(6..8)?
            } else {
                255
            },
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(text: &str) -> ClipboardItem {
        ClipboardItem::new_text(text.to_string())
    }

    #[test]
    fn sensitive_wins_before_url_or_json() {
        let mut value = item("https://example.com");
        value.sensitive = true;
        assert!(matches!(
            present(&value),
            ContentPresentation::Secret { .. }
        ));
    }

    #[test]
    fn recognizes_url_color_json_and_code() {
        assert!(
            matches!(present(&item("https://example.com/a")), ContentPresentation::Url { domain, .. } if domain == "example.com")
        );
        assert!(matches!(
            present(&item("#0af")),
            ContentPresentation::Color {
                rgba: (0, 170, 255, 255),
                ..
            }
        ));
        assert!(matches!(
            present(&item(r#"{"a":1}"#)),
            ContentPresentation::Json { .. }
        ));
        assert!(matches!(
            present(&item("SELECT * FROM users")),
            ContentPresentation::Code { .. }
        ));
    }

    #[test]
    fn oversized_content_is_bounded_unknown() {
        let value = item(&"x".repeat(CLASSIFY_LIMIT + 1));
        assert!(
            matches!(present(&value), ContentPresentation::Unknown { preview } if preview.chars().count() <= PREVIEW_CHARS + 1)
        );
    }
}
