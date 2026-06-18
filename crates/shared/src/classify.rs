//! Content classification for clipboard items.
//!
//! Classifies text content into categories like code, command, URL, path,
//! JSON, SQL, image reference, file list, snippet, and secret.
//!
//! # Examples
//!
//! ```
//! use author_clipboard_shared::classify::{classify, ContentClass};
//!
//! let cls = classify("https://example.com");
//! assert!(matches!(cls, ContentClass::Url));
//!
//! let cls = classify("SELECT * FROM users WHERE id = 1");
//! assert!(matches!(cls, ContentClass::Sql));
//! ```

use serde::{Deserialize, Serialize};

/// The classification of a piece of content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentClass {
    /// Plain text with no special structure detected.
    Text,
    /// Source code in a programming language.
    Code,
    /// A shell command (looks like `cmd args` or starts with special chars).
    Command,
    /// A web URL (http://, https://, etc.).
    Url,
    /// A file or directory path (absolute or relative).
    Path,
    /// JSON data structure.
    Json,
    /// SQL query or statement.
    Sql,
    /// Email address.
    Email,
    /// A programming-related identifier or variable name.
    Identifier,
    /// A sensitive value (password, token, key, etc.).
    Secret,
    /// A file list (multiple lines that look like paths or URIs).
    FileList,
    /// An image reference or data URI.
    ImageRef,
    /// A snippet template (contains `${variable}` patterns).
    Snippet,
}

impl ContentClass {
    /// Human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Code => "code",
            Self::Command => "command",
            Self::Url => "URL",
            Self::Path => "path",
            Self::Json => "JSON",
            Self::Sql => "SQL",
            Self::Email => "email",
            Self::Identifier => "identifier",
            Self::Secret => "secret",
            Self::FileList => "files",
            Self::ImageRef => "image",
            Self::Snippet => "snippet",
        }
    }

    /// Whether this classification typically requires privacy handling.
    pub fn is_sensitive(&self) -> bool {
        matches!(self, Self::Secret)
    }
}

impl std::fmt::Display for ContentClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Classify a string into a content class.
///
/// Detection order:
/// 1. Secret patterns (passwords, tokens, keys) → `Secret`
/// 2. JSON → `Json`
/// 3. SQL → `Sql`
/// 4. URL patterns → `Url`
/// 5. Email patterns → `Email`
/// 6. Image data URIs → `ImageRef`
/// 7. Snippet template syntax `${...}` → `Snippet`
/// 8. Command-like patterns → `Command`
/// 9. File path patterns → `Path`
/// 10. Code-like patterns (braces, semicolons, keywords) → `Code`
/// 11. File list (multiple lines with paths) → `FileList`
/// 12. Otherwise → `Text`
pub fn classify(content: &str) -> ContentClass {
    let trimmed = content.trim();

    // 1. Check for secrets first (highest priority for detection)
    if is_secret_pattern(trimmed) {
        return ContentClass::Secret;
    }

    // 2. JSON detection
    if is_json(trimmed) {
        return ContentClass::Json;
    }

    // 3. SQL detection
    if is_sql(trimmed) {
        return ContentClass::Sql;
    }

    // 4. Email detection (before URL, as emails look like URLs but aren't)
    if is_email(trimmed) {
        return ContentClass::Email;
    }

    // 5. File list detection (before URL, to catch file:// URIs)
    if is_file_list(content) {
        return ContentClass::FileList;
    }

    // 6. URL detection
    if is_url(trimmed) {
        return ContentClass::Url;
    }

    // 7. Image data URI
    if is_image_data_uri(trimmed) {
        return ContentClass::ImageRef;
    }

    // 8. Snippet template
    if is_snippet_template(trimmed) {
        return ContentClass::Snippet;
    }

    // 9. Command detection
    if is_command(trimmed) {
        return ContentClass::Command;
    }

    // 10. Path detection
    if is_path(trimmed) {
        return ContentClass::Path;
    }

    // 10. Code detection
    if is_code(trimmed) {
        return ContentClass::Code;
    }

    // 11. File list detection (multiple lines)
    if is_file_list(content) {
        return ContentClass::FileList;
    }

    // 12. Default
    ContentClass::Text
}

/// Classify with confidence score (0.0 to 1.0).
///
/// Returns (class, confidence) where confidence indicates how certain
/// the classifier is about the result.
pub fn classify_with_confidence(content: &str) -> (ContentClass, f32) {
    let trimmed = content.trim();

    // High confidence detections
    if is_secret_pattern(trimmed) {
        return (ContentClass::Secret, 0.95);
    }

    if is_json(trimmed) {
        return (ContentClass::Json, 0.90);
    }

    if is_sql(trimmed) {
        return (ContentClass::Sql, 0.90);
    }

    if is_url(trimmed) {
        return (ContentClass::Url, 0.95);
    }

    if is_email(trimmed) {
        return (ContentClass::Email, 0.90);
    }

    if is_image_data_uri(trimmed) {
        return (ContentClass::ImageRef, 0.95);
    }

    if is_snippet_template(trimmed) {
        return (ContentClass::Snippet, 0.85);
    }

    if is_command(trimmed) {
        return (ContentClass::Command, 0.75);
    }

    if is_path(trimmed) {
        return (ContentClass::Path, 0.70);
    }

    if is_code(trimmed) {
        return (ContentClass::Code, 0.65);
    }

    if is_file_list(content) {
        return (ContentClass::FileList, 0.70);
    }

    (ContentClass::Text, 0.50)
}

// ── Detection helpers ────────────────────────────────────────────────

fn is_secret_pattern(s: &str) -> bool {
    // Check for common secret patterns
    let secret_indicators = [
        "password=",
        "passwd=",
        "pass=",
        "secret=",
        "token=",
        "api_key=",
        "apikey=",
        "api-key=",
        "bearer ",
        "authorization:",
        "private_key=",
        "privatekey=",
        "ssh-rsa",
        "-----begin",
        "-----end",
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_", // GitHub tokens
        "sk-",  // OpenAI API keys
    ];

    let lower = s.to_lowercase();
    for indicator in &secret_indicators {
        if lower.contains(indicator) {
            return true;
        }
    }

    // Check for JWT-like patterns
    if (s.starts_with("eyJ") || s.starts_with("eyI"))
        && s.chars().filter(|c| *c == '.').count() == 2
    {
        return true;
    }

    // Check for UUID/GUID (often used as session IDs, tokens)
    if s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4 {
        // Could be a UUID - moderate confidence
    }

    false
}

fn is_json(s: &str) -> bool {
    let s = s.trim();
    if !((s.starts_with('{') && s.ends_with('}')) || (s.starts_with('[') && s.ends_with(']'))) {
        return false;
    }

    // Try to parse as JSON
    serde_json::from_str::<serde_json::Value>(s).is_ok()
}

fn is_sql(s: &str) -> bool {
    let s = s.trim();
    // Short strings are unlikely to be SQL queries
    if s.len() < 10 {
        return false;
    }

    let upper = s.to_uppercase();
    let sql_keywords = [
        "SELECT",
        "INSERT",
        "UPDATE",
        "DELETE",
        "CREATE",
        "DROP",
        "ALTER",
        "TABLE",
        "INDEX",
        "VIEW",
        "WHERE",
        "JOIN",
        "LEFT JOIN",
        "RIGHT JOIN",
        "INNER JOIN",
        "OUTER JOIN",
        "GROUP BY",
        "ORDER BY",
        "HAVING",
        "LIMIT",
        "OFFSET",
        "UNION",
        "DISTINCT",
        "COUNT",
        "SUM",
        "AVG",
        "MAX",
        "MIN",
        "PRIMARY KEY",
        "FOREIGN KEY",
        "REFERENCES",
        "CONSTRAINT",
    ];

    for keyword in &sql_keywords {
        if upper.contains(keyword) {
            // Check word boundary - the keyword should not be part of a longer word
            let start = upper.find(keyword).unwrap();
            let end = start + keyword.len();

            // Check that it's not preceded by a letter or digit
            let valid_start = start == 0
                || !upper[..start]
                    .chars()
                    .last()
                    .is_some_and(char::is_alphanumeric);

            // Check that it's not followed by a letter or digit
            let valid_end = end >= upper.len()
                || !upper[end..]
                    .chars()
                    .next()
                    .is_some_and(char::is_alphanumeric);

            if valid_start && valid_end {
                return true;
            }
        }
    }

    // Also detect SQL comment style
    upper.starts_with("--") || upper.starts_with("/*")
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn is_url(s: &str) -> bool {
    let s = s.trim();
    let url_prefixes = [
        "http://", "https://", "ftp://", "file://", "mailto:", "tel:", "ssh://", "git://", "ws://",
        "wss://", // WebSocket
    ];

    for prefix in &url_prefixes {
        if s.to_lowercase().starts_with(prefix) {
            return true;
        }
    }

    // Domain name pattern (basic check) - but be strict about it
    // Must have recognizable TLD and common URL structure
    if s.contains('.') && !s.contains(' ') && s.len() < 2000 {
        // Check for common TLDs or known domain patterns
        let lower = s.to_lowercase();
        let has_tld = [
            ".com",
            ".org",
            ".net",
            ".io",
            ".co",
            ".edu",
            ".gov",
            ".mil",
            ".info",
            ".biz",
            ".name",
            ".museum",
            ".travel",
            ".pro",
            ".aero",
            ".xyz",
            ".online",
            ".site",
            ".web",
            ".tech",
            ".dev",
            ".ru",
            ".cn",
            ".jp",
            ".de",
            ".fr",
            ".uk",
            ".br",
            ".au",
            ".localhost",
        ]
        .iter()
        .any(|tld| lower.ends_with(tld));

        if has_tld {
            // Looks like a domain - but must have URL-like structure
            // (contains /, or ?, or ends with common TLD)
            if lower.ends_with(".com")
                || lower.ends_with(".org")
                || lower.ends_with(".net")
                || lower.ends_with(".io")
                || lower.ends_with(".dev")
                || lower.ends_with(".tech")
                || s.contains('/')
                || s.contains('?')
            {
                return true;
            }
        }
    }

    false
}

fn is_email(s: &str) -> bool {
    let s = s.trim();
    // Basic email pattern
    s.contains('@') && s.contains('.') && !s.contains(' ') && s.len() < 254
}

fn is_image_data_uri(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("data:image/")
}

fn is_snippet_template(s: &str) -> bool {
    // Look for ${variable} or $variable patterns
    s.contains("${") || s.contains("$}") || s.contains("${}")
}

fn is_command(s: &str) -> bool {
    let s = s.trim();

    // Shell metacharacter prefixes
    if s.starts_with('|')
        || s.starts_with('>')
        || s.starts_with('<')
        || s.starts_with('&')
        || s.starts_with(';')
        || s.starts_with('$')
        || s.starts_with('!')
    {
        return true;
    }

    // Command-like patterns: "command subcommand args"
    let command_indicators = [
        "git ",
        "docker ",
        "kubectl ",
        "cargo ",
        "npm ",
        "yarn ",
        "pnpm ",
        "python ",
        "python3 ",
        "ruby ",
        "perl ",
        "php ",
        "curl ",
        "wget ",
        "ssh ",
        "scp ",
        "rsync ",
        "chmod ",
        "chown ",
        "ln ",
        "cp ",
        "mv ",
        "rm ",
        "find ",
        "grep ",
        "sed ",
        "awk ",
        "cat ",
        "echo ",
        "ls ",
        "cd ",
        "pwd ",
        "mkdir ",
        "rmdir ",
        "systemctl ",
        "service ",
        "journalctl ",
        "apt ",
        "yum ",
        "dnf ",
        "pacman ",
        "snap ",
        "make ",
        "cmake ",
        "gcc ",
        "g++ ",
        "rustc ",
        "go ",
        "java ",
        "node ",
        "ruby ",
        "lua ",
    ];

    let lower = s.to_lowercase();
    for indicator in &command_indicators {
        if lower.starts_with(indicator) {
            return true;
        }
    }

    // Piped commands
    if s.contains(" | ") {
        return true;
    }

    false
}

fn is_path(s: &str) -> bool {
    let s = s.trim();

    // Absolute Unix path
    if s.starts_with('/') {
        return !s.contains("://"); // Not a URL
    }

    // Windows path
    if s.len() >= 3 {
        let bytes = s.as_bytes();
        if bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
            return true;
        }
    }

    // Home directory shorthand
    if s.starts_with("~/") || s.starts_with("~\\") {
        return true;
    }

    false
}

fn is_code(s: &str) -> bool {
    let s = s.trim();

    // Check for code-like characteristics
    let code_indicators = [
        "{}",
        "[]",
        "();",
        "fn ",
        "func ",
        "def ",
        "class ",
        "struct ",
        "enum ",
        "impl ",
        "pub ",
        "let ",
        "const ",
        "var ",
        "if ",
        "else ",
        "for ",
        "while ",
        "return ",
        "import ",
        "export ",
        "module",
        "package",
        "namespace",
        "using",
        "std::",
        "::",
        "->",
        "=>",
        "==",
        "!=",
        "<=",
        ">=",
        "&&",
        "||",
        "//",
        "/*",
        "*/",
        "#include",
        "#define",
        "#if",
        "#endif",
    ];

    let lower = s.to_lowercase();
    let mut score = 0;

    for indicator in &code_indicators {
        if lower.contains(indicator) {
            score += 1;
            if score >= 2 {
                return true;
            }
        }
    }

    // Check for bracket balance
    let open_braces = s
        .chars()
        .filter(|c| *c == '{' || *c == '[' || *c == '(')
        .count();
    let close_braces = s
        .chars()
        .filter(|c| *c == '}' || *c == ']' || *c == ')')
        .count();
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    if open_braces > 0
        && close_braces > 0
        && (open_braces as i32 - close_braces as i32).abs() <= 2
        && open_braces + close_braces >= 4
    {
        return true;
    }

    false
}

fn is_file_list(s: &str) -> bool {
    // Multiple lines, each looking like a path or URI
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() < 2 {
        return false;
    }

    let mut path_count = 0;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('/')
            || trimmed.starts_with("~/")
            || trimmed.starts_with("file://")
            || trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("ftp://")
            || (trimmed.len() >= 3
                && trimmed.as_bytes()[1] == b':'
                && (trimmed.as_bytes()[2] == b'\\' || trimmed.as_bytes()[2] == b'/'))
        {
            path_count += 1;
        }
    }

    // If more than half the lines look like paths, it's a file list
    path_count >= lines.len() / 2 && path_count >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_secret_password() {
        let cls = classify("password=secret123");
        assert!(matches!(cls, ContentClass::Secret));
    }

    #[test]
    fn test_classify_secret_token() {
        let cls = classify("Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
        assert!(matches!(cls, ContentClass::Secret));
    }

    #[test]
    fn test_classify_secret_private_key() {
        let cls = classify(
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BA\n-----END PRIVATE KEY-----",
        );
        assert!(matches!(cls, ContentClass::Secret));
    }

    #[test]
    fn test_classify_json_object() {
        let cls = classify(r#"{"name": "test", "value": 123}"#);
        assert!(matches!(cls, ContentClass::Json));
    }

    #[test]
    fn test_classify_json_array() {
        let cls = classify(r#"[1, 2, 3, "four"]"#);
        assert!(matches!(cls, ContentClass::Json));
    }

    #[test]
    fn test_classify_sql_select() {
        let cls = classify("SELECT * FROM users WHERE id = 1");
        assert!(matches!(cls, ContentClass::Sql));
    }

    #[test]
    fn test_classify_sql_insert() {
        let cls = classify("INSERT INTO items (name) VALUES ('test')");
        assert!(matches!(cls, ContentClass::Sql));
    }

    #[test]
    fn test_classify_url_https() {
        let cls = classify("https://example.com/path?query=value");
        assert!(matches!(cls, ContentClass::Url));
    }

    #[test]
    fn test_classify_url_http() {
        let cls = classify("http://localhost:8080/api");
        assert!(matches!(cls, ContentClass::Url));
    }

    #[test]
    fn test_classify_url_ftp() {
        let cls = classify("ftp://files.example.com/pub");
        assert!(matches!(cls, ContentClass::Url));
    }

    #[test]
    fn test_classify_email() {
        let cls = classify("user@example.com");
        assert!(matches!(cls, ContentClass::Email));
    }

    #[test]
    fn test_classify_image_data_uri() {
        let cls = classify("data:image/png;base64,iVBORw0KGgoAAAANSUhEU");
        assert!(matches!(cls, ContentClass::ImageRef));
    }

    #[test]
    fn test_classify_snippet_template() {
        let cls = classify("Hello ${name}, today is ${date}");
        assert!(matches!(cls, ContentClass::Snippet));
    }

    #[test]
    fn test_classify_command_git() {
        let cls = classify("git commit -m 'fix: something'");
        assert!(matches!(cls, ContentClass::Command));
    }

    #[test]
    fn test_classify_command_docker() {
        let cls = classify("docker run -it ubuntu bash");
        assert!(matches!(cls, ContentClass::Command));
    }

    #[test]
    fn test_classify_command_with_pipe() {
        let cls = classify("cat file.txt | grep pattern");
        assert!(matches!(cls, ContentClass::Command));
    }

    #[test]
    fn test_classify_command_shell_redirect() {
        let cls = classify("echo hello > output.txt");
        assert!(matches!(cls, ContentClass::Command));
    }

    #[test]
    fn test_classify_path_unix() {
        let cls = classify("/home/user/Documents/file.txt");
        assert!(matches!(cls, ContentClass::Path));
    }

    #[test]
    fn test_classify_path_home() {
        let cls = classify("~/Projects/myapp/src/main.rs");
        assert!(matches!(cls, ContentClass::Path));
    }

    #[test]
    fn test_classify_path_windows() {
        let cls = classify("C:\\Users\\name\\Documents\\file.txt");
        assert!(matches!(cls, ContentClass::Path));
    }

    #[test]
    fn test_classify_code_rust() {
        let cls = classify("fn main() {\n    println!(\"hello\");\n}");
        assert!(matches!(cls, ContentClass::Code));
    }

    #[test]
    fn test_classify_code_javascript() {
        let cls = classify("function hello() {\n  return 'world';\n}");
        assert!(matches!(cls, ContentClass::Code));
    }

    #[test]
    fn test_classify_code_python() {
        let cls = classify("def hello():\n    print('hello')\n    return True");
        assert!(matches!(cls, ContentClass::Code));
    }

    #[test]
    fn test_classify_file_list() {
        let cls = classify("/path/to/file1.txt\n/path/to/file2.txt\n/path/to/file3.txt");
        assert!(matches!(cls, ContentClass::FileList));
    }

    #[test]
    fn test_classify_file_list_mixed() {
        let cls = classify("file:///path/one\nfile:///path/two\nfile:///path/three");
        assert!(matches!(cls, ContentClass::FileList));
    }

    #[test]
    fn test_classify_plain_text() {
        let cls = classify("Hello, this is just a plain text message.");
        assert!(matches!(cls, ContentClass::Text));
    }

    #[test]
    fn test_classify_empty() {
        let cls = classify("");
        assert!(matches!(cls, ContentClass::Text));
    }

    #[test]
    fn test_classify_whitespace() {
        let cls = classify("   \n\n  \t  ");
        assert!(matches!(cls, ContentClass::Text));
    }

    #[test]
    fn test_classify_with_confidence() {
        let (cls, conf) = classify_with_confidence("https://example.com");
        assert!(matches!(cls, ContentClass::Url));
        assert!(conf >= 0.9);
    }

    #[test]
    fn test_classify_secret_with_confidence() {
        let (cls, conf) = classify_with_confidence("api_key=sk-1234567890abcdef");
        assert!(matches!(cls, ContentClass::Secret));
        assert!(conf >= 0.9);
    }

    #[test]
    fn test_content_class_label() {
        assert_eq!(ContentClass::Text.label(), "text");
        assert_eq!(ContentClass::Code.label(), "code");
        assert_eq!(ContentClass::Secret.label(), "secret");
        assert_eq!(ContentClass::Url.label(), "URL");
    }

    #[test]
    fn test_content_class_is_sensitive() {
        assert!(!ContentClass::Text.is_sensitive());
        assert!(!ContentClass::Url.is_sensitive());
        assert!(ContentClass::Secret.is_sensitive());
    }

    #[test]
    fn test_content_class_display() {
        assert_eq!(format!("{}", ContentClass::Json), "JSON");
        assert_eq!(format!("{}", ContentClass::Sql), "SQL");
    }
}
