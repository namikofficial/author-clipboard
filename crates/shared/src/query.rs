//! Query string parser for developer filters.
//!
//! Parses query strings with prefixes like `type:`, `app:`, `project:`,
//! `collection:` and returns a structured query that can be used to
//! filter clipboard items.
//!
//! # Examples
//!
//! ```
//! use author_clipboard_shared::query::{Query, ContentTypeFilter};
//!
//! let q = Query::parse("type:text password");
//! assert!(matches!(q.content_type, Some(ContentTypeFilter::Text)));
//! assert_eq!(q.text(), Some("password"));
//!
//! let q = Query::parse("collection:work type:image");
//! assert!(matches!(q.content_type, Some(ContentTypeFilter::Image)));
//! assert_eq!(q.collection.as_deref(), Some("work"));
//! ```

use serde::{Deserialize, Serialize};

/// A parsed query string with structured filters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    /// The raw text search terms (without prefix tokens).
    pub text: Option<String>,
    /// Filter by content type (type: text|image|files|html).
    pub content_type: Option<ContentTypeFilter>,
    /// Filter by source application (app: firefox).
    pub app: Option<String>,
    /// Filter by project tag (project: myproject).
    pub project: Option<String>,
    /// Filter by collection name (collection: work).
    pub collection: Option<String>,
    /// Filter by pinned status.
    pub pinned: Option<bool>,
    /// Filter by starred status.
    pub starred: Option<bool>,
    /// Filter by sensitive status.
    pub sensitive: Option<bool>,
}

/// Content type filter values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentTypeFilter {
    Text,
    Image,
    Files,
    Html,
}

impl ContentTypeFilter {
    /// Parse from a string like "text", "image", "files", "html".
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "image" | "images" => Some(Self::Image),
            "file" | "files" => Some(Self::Files),
            "html" | "htm" => Some(Self::Html),
            _ => None,
        }
    }

    /// Return the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Files => "files",
            Self::Html => "html",
        }
    }
}

impl std::fmt::Display for ContentTypeFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Query {
    /// Parse a query string into structured filters.
    ///
    /// Supports prefixes:
    /// - `type:` — content type (text, image, files, html)
    /// - `app:` — source application name
    /// - `project:` — project tag
    /// - `collection:` — collection name
    /// - `pinned:` — pinned status (true/false)
    /// - `starred:` — starred status (true/false)
    /// - `sensitive:` — sensitive status (true/false)
    ///
    /// The remaining text without a prefix is stored as the search text.
    ///
    /// # Examples
    ///
    /// ```
    /// use author_clipboard_shared::query::Query;
    ///
    /// let q = Query::parse("type:text password");
    /// assert!(matches!(q.content_type, Some(author_clipboard_shared::query::ContentTypeFilter::Text)));
    /// ```
    pub fn parse(input: &str) -> Self {
        let mut query = Self::default();
        let input = input.trim();

        if input.is_empty() {
            return query;
        }

        let mut current_text = String::new();

        for token in input.split_whitespace() {
            let (prefix, value) = if let Some(pos) = token.find(':') {
                (&token[..pos], &token[pos + 1..])
            } else {
                ("", token)
            };

            match prefix {
                "type" => {
                    if let Some(ct) = ContentTypeFilter::parse(value) {
                        query.content_type = Some(ct);
                    }
                }
                "app" => {
                    if !value.is_empty() {
                        query.app = Some(value.to_string());
                    }
                }
                "project" => {
                    if !value.is_empty() {
                        query.project = Some(value.to_string());
                    }
                }
                "collection" | "col" => {
                    if !value.is_empty() {
                        query.collection = Some(value.to_string());
                    }
                }
                "pinned" | "pin" => {
                    query.pinned = Some(parse_bool(value));
                }
                "starred" | "star" => {
                    query.starred = Some(parse_bool(value));
                }
                "sensitive" | "secret" => {
                    query.sensitive = Some(parse_bool(value));
                }
                "" => {
                    // Plain text token
                    if !current_text.is_empty() {
                        current_text.push(' ');
                    }
                    current_text.push_str(token);
                }
                _ => {
                    // Unknown prefix, treat as text
                    if !current_text.is_empty() {
                        current_text.push(' ');
                    }
                    current_text.push_str(token);
                }
            }
        }

        if !current_text.is_empty() {
            query.text = Some(current_text);
        }

        query
    }

    /// Returns true if no filters are set (empty query).
    pub fn is_empty(&self) -> bool {
        self.text.is_none()
            && self.content_type.is_none()
            && self.app.is_none()
            && self.project.is_none()
            && self.collection.is_none()
            && self.pinned.is_none()
            && self.starred.is_none()
            && self.sensitive.is_none()
    }

    /// Returns the plain text search terms, if any.
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Build a human-readable description of the query.
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref ct) = self.content_type {
            parts.push(format!("type:{ct}"));
        }
        if let Some(ref app) = self.app {
            parts.push(format!("app:{app}"));
        }
        if let Some(ref project) = self.project {
            parts.push(format!("project:{project}"));
        }
        if let Some(ref col) = self.collection {
            parts.push(format!("collection:{col}"));
        }
        if let Some(pinned) = self.pinned {
            parts.push(format!("pinned:{pinned}"));
        }
        if let Some(starred) = self.starred {
            parts.push(format!("starred:{starred}"));
        }
        if let Some(sensitive) = self.sensitive {
            parts.push(format!("sensitive:{sensitive}"));
        }
        if let Some(ref text) = self.text {
            parts.push(format!("\"{text}\""));
        }

        if parts.is_empty() {
            "empty query".to_string()
        } else {
            parts.join(" ")
        }
    }
}

/// Parse a boolean value from a string.
fn parse_bool(s: &str) -> bool {
    matches!(
        s.trim().to_lowercase().as_str(),
        "true" | "1" | "yes" | "y" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let q = Query::parse("");
        assert!(q.is_empty());
        assert!(q.text.is_none());
        assert!(q.content_type.is_none());
    }

    #[test]
    fn test_plain_text_only() {
        let q = Query::parse("hello world");
        assert_eq!(q.text.as_deref(), Some("hello world"));
        assert!(q.content_type.is_none());
    }

    #[test]
    fn test_type_filter_text() {
        let q = Query::parse("type:text");
        assert!(matches!(q.content_type, Some(ContentTypeFilter::Text)));
        assert!(q.text.is_none());
    }

    #[test]
    fn test_type_filter_image() {
        let q = Query::parse("type:image");
        assert!(matches!(q.content_type, Some(ContentTypeFilter::Image)));
    }

    #[test]
    fn test_type_filter_files() {
        let q = Query::parse("type:files");
        assert!(matches!(q.content_type, Some(ContentTypeFilter::Files)));
    }

    #[test]
    fn test_type_filter_with_text() {
        let q = Query::parse("type:image screenshot");
        assert!(matches!(q.content_type, Some(ContentTypeFilter::Image)));
        assert_eq!(q.text.as_deref(), Some("screenshot"));
    }

    #[test]
    fn test_app_filter() {
        let q = Query::parse("app:firefox");
        assert_eq!(q.app.as_deref(), Some("firefox"));
    }

    #[test]
    fn test_project_filter() {
        let q = Query::parse("project:work");
        assert_eq!(q.project.as_deref(), Some("work"));
    }

    #[test]
    fn test_collection_filter() {
        let q = Query::parse("collection:important");
        assert_eq!(q.collection.as_deref(), Some("important"));
    }

    #[test]
    fn test_collection_filter_short() {
        let q = Query::parse("col:important");
        assert_eq!(q.collection.as_deref(), Some("important"));
    }

    #[test]
    fn test_pinned_filter_true() {
        let q = Query::parse("pinned:true");
        assert_eq!(q.pinned, Some(true));
    }

    #[test]
    fn test_pinned_filter_false() {
        let q = Query::parse("pinned:false");
        assert_eq!(q.pinned, Some(false));
    }

    #[test]
    fn test_starred_filter() {
        let q = Query::parse("starred:true");
        assert_eq!(q.starred, Some(true));
    }

    #[test]
    fn test_sensitive_filter() {
        let q = Query::parse("sensitive:true");
        assert_eq!(q.sensitive, Some(true));
    }

    #[test]
    fn test_multiple_filters() {
        let q = Query::parse("type:text app:code project:myproj hello");
        assert!(matches!(q.content_type, Some(ContentTypeFilter::Text)));
        assert_eq!(q.app.as_deref(), Some("code"));
        assert_eq!(q.project.as_deref(), Some("myproj"));
        assert_eq!(q.text.as_deref(), Some("hello"));
    }

    #[test]
    fn test_unknown_prefix_treated_as_text() {
        let q = Query::parse("unknown:value");
        assert_eq!(q.text.as_deref(), Some("unknown:value"));
    }

    #[test]
    fn test_bool_parsing() {
        assert!(parse_bool("true"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("y"));
        assert!(parse_bool("on"));
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("off"));
    }

    #[test]
    fn test_query_description() {
        let q = Query::parse("type:text app:firefox password");
        let desc = q.description();
        assert!(desc.contains("type:text"));
        assert!(desc.contains("app:firefox"));
        assert!(desc.contains("\"password\""));
    }

    #[test]
    fn test_query_is_empty() {
        let q = Query::default();
        assert!(q.is_empty());
        let q = Query::parse("");
        assert!(q.is_empty());
        let q = Query::parse("type:text");
        assert!(!q.is_empty());
    }

    #[test]
    fn test_text_method() {
        let q = Query::parse("hello world");
        assert_eq!(q.text(), Some("hello world"));
        let q = Query::parse("");
        assert_eq!(q.text(), None);
    }

    #[test]
    fn test_content_type_filter_display() {
        assert_eq!(ContentTypeFilter::Text.to_string(), "text");
        assert_eq!(ContentTypeFilter::Image.to_string(), "image");
        assert_eq!(ContentTypeFilter::Files.to_string(), "files");
        assert_eq!(ContentTypeFilter::Html.to_string(), "html");
    }

    #[test]
    fn test_content_type_filter_parse() {
        assert!(matches!(
            ContentTypeFilter::parse("text"),
            Some(ContentTypeFilter::Text)
        ));
        assert!(matches!(
            ContentTypeFilter::parse("image"),
            Some(ContentTypeFilter::Image)
        ));
        assert!(matches!(
            ContentTypeFilter::parse("images"),
            Some(ContentTypeFilter::Image)
        ));
        assert!(matches!(
            ContentTypeFilter::parse("files"),
            Some(ContentTypeFilter::Files)
        ));
        assert!(matches!(
            ContentTypeFilter::parse("html"),
            Some(ContentTypeFilter::Html)
        ));
        assert!(ContentTypeFilter::parse("unknown").is_none());
    }
}
