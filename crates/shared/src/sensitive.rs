//! Sensitive content detection for clipboard items
//!
//! Detects passwords, OTP codes, API keys, and other sensitive data
//! to warn users and optionally auto-expire these items.

/// Result of checking content for sensitive data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitivityCheck {
    /// Whether the content was classified as sensitive.
    pub is_sensitive: bool,
    /// Human-readable reason for the classification, if sensitive.
    pub reason: Option<String>,
}

impl SensitivityCheck {
    fn safe() -> Self {
        Self {
            is_sensitive: false,
            reason: None,
        }
    }

    fn sensitive(reason: impl Into<String>) -> Self {
        Self {
            is_sensitive: true,
            reason: Some(reason.into()),
        }
    }
}

/// Check if clipboard text content appears to be sensitive.
///
/// Detects:
/// - OTP/2FA codes (6-8 digit codes)
/// - API keys and tokens (long hex/base64 strings)
/// - Password-like patterns
/// - Private keys
/// - Connection strings with credentials
pub fn check_sensitivity(content: &str) -> SensitivityCheck {
    let trimmed = content.trim();

    // Private key blocks
    if trimmed.contains("-----BEGIN") && trimmed.contains("PRIVATE KEY-----") {
        return SensitivityCheck::sensitive("Private key detected");
    }

    // JWT tokens (three base64 segments separated by dots)
    if is_jwt_like(trimmed) {
        return SensitivityCheck::sensitive("JWT token detected");
    }

    // Connection strings with passwords
    if has_connection_credentials(trimmed) {
        return SensitivityCheck::sensitive("Connection string with credentials");
    }

    // API key patterns (common prefixes)
    if is_api_key(trimmed) {
        return SensitivityCheck::sensitive("API key or token detected");
    }

    // OTP codes (exactly 6-8 digits, standalone)
    if is_otp_code(trimmed) {
        return SensitivityCheck::sensitive("OTP/verification code detected");
    }

    // Long hex strings (likely tokens/hashes)
    if is_hex_token(trimmed) {
        return SensitivityCheck::sensitive("Hex token or hash detected");
    }

    // Password field content (from password managers)
    if looks_like_password(trimmed) {
        return SensitivityCheck::sensitive("Possible password detected");
    }

    SensitivityCheck::safe()
}

/// Check an HTML clipboard item for sensitive content.
///
/// `html_content` is the raw `text/html` payload (possibly containing
/// `value="..."` attributes, hidden fields, inline scripts, etc.).
/// `plain_text` is the browser-provided `text/plain` companion
/// (already pre-stripped by the source app).
///
/// Sensitive if any of the following looks like a credential, token,
/// or connection string with embedded password:
/// - the `text/plain` companion,
/// - the visible text of the HTML body (tags stripped),
/// - the value of any HTML attribute,
/// - the content of any HTML comment,
/// - the raw HTML payload as a last resort.
///
/// This is intentionally multi-layered because secret-bearing form
/// fields and OAuth-token URL fragments frequently hide in attributes
/// that are not part of the rendered text.
pub fn check_sensitive_html(html_content: &str, plain_text: &str) -> SensitivityCheck {
    // 1. Plain text companion.
    let plain_check = check_sensitivity(plain_text);
    if plain_check.is_sensitive {
        return plain_check;
    }

    // 2. Tag-stripped visible text.
    let stripped = strip_html_tags(html_content);
    let stripped_check = check_sensitivity(&stripped);
    if stripped_check.is_sensitive {
        return stripped_check;
    }

    // 3. Attribute values (catches <input value="hunter2"> etc.).
    for value in extract_attribute_values(html_content) {
        let check = check_sensitivity(&value);
        if check.is_sensitive {
            return check;
        }
    }

    // 4. HTML comments (catches <!-- API key: sk-... -->).
    for comment in extract_html_comments(html_content) {
        let check = check_sensitivity(&comment);
        if check.is_sensitive {
            return check;
        }
    }

    // 5. Raw HTML, as a last resort. Catches subtle cases where
    //    the secret is only identifiable when delimiters and
    //    surrounding text are visible together.
    check_sensitivity(html_content)
}

/// Extract the value of every quoted HTML attribute in `html`.
///
/// Handles both `"..."` and `'...'` quoting. The returned strings
/// are the raw attribute values; HTML entity decoding is **not**
/// performed (the sensitive-content checks operate on raw bytes).
pub fn extract_attribute_values(html: &str) -> Vec<String> {
    let bytes = html.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' && i + 1 < bytes.len() {
            let quote = bytes[i + 1];
            if quote == b'"' || quote == b'\'' {
                let start = i + 2;
                let mut j = start;
                while j < bytes.len() && bytes[j] != quote {
                    j += 1;
                }
                if j <= bytes.len() {
                    let value = String::from_utf8_lossy(&bytes[start..j]).into_owned();
                    if !value.is_empty() {
                        out.push(value);
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// Extract the content of every `<!-- ... -->` HTML comment in
/// `html`. Empty comments are skipped.
pub fn extract_html_comments(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if &bytes[i..i + 4] == b"<!--" {
            let start = i + 4;
            let mut j = start;
            while j + 2 < bytes.len() && &bytes[j..j + 3] != b"-->" {
                j += 1;
            }
            if j + 2 < bytes.len() {
                let content = String::from_utf8_lossy(&bytes[start..j]).into_owned();
                if !content.is_empty() {
                    out.push(content);
                }
                i = j + 3;
                continue;
            }
            break;
        }
        i += 1;
    }
    out
}

/// Strip HTML tags from a string.
///
/// This is **not** a security-grade HTML sanitizer. It is only used
/// to make textual content inside `text/html` payloads visible to the
/// sensitive-content text checks. Untrusted HTML must never be
/// rendered from this output.
///
/// The function is best-effort:
/// - `<script>` and `<style>` blocks are dropped entirely (their body
///   is almost always JS / CSS, not a secret in the form-field sense).
/// - All other `<...>` tags are removed, leaving the inner text.
/// - HTML comments `<!-- ... -->` are dropped.
/// - HTML entities are *not* decoded.
pub fn strip_html_tags(html: &str) -> String {
    // Step 1: drop <script> and <style> blocks.
    let mut rest = html.to_string();
    for block_tag in ["script", "style"] {
        rest = drop_block(&rest, block_tag);
    }
    // Step 2: drop HTML comments.
    rest = drop_comments(&rest);
    // Step 3: drop remaining tags.
    drop_remaining_tags(&rest)
}

fn drop_block(input: &str, block_tag: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let open = format!("<{block_tag}");
    let close = format!("</{block_tag}>");
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;
    let mut next_piece_start = 0usize;
    while let Some(open_rel) = lower[cursor..].find(&open) {
        let open_abs = cursor + open_rel;
        let after_tag = open_abs + open.len();
        let next_char = lower.as_bytes().get(after_tag).copied();
        let is_tag_start = matches!(next_char, Some(b' ') | Some(b'>') | Some(b'/'));
        if !is_tag_start {
            cursor = open_abs + 1;
            continue;
        }
        if let Some(close_rel) = lower[open_abs..].find(&close) {
            let close_abs = open_abs + close_rel + close.len();
            out.push_str(&input[next_piece_start..open_abs]);
            cursor = close_abs;
            next_piece_start = close_abs;
        } else {
            break;
        }
    }
    out.push_str(&input[next_piece_start..]);
    out
}

fn drop_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;
    while let Some(open_rel) = input[cursor..].find("<!--") {
        let open_abs = cursor + open_rel;
        out.push_str(&input[cursor..open_abs]);
        if let Some(close_rel) = input[open_abs..].find("-->") {
            cursor = open_abs + close_rel + 3;
        } else {
            cursor = input.len();
            break;
        }
    }
    out.push_str(&input[cursor..]);
    out
}

fn drop_remaining_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn is_jwt_like(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    // Each part should be base64url-ish (alphanumeric + - _ =)
    parts.iter().all(|p| {
        p.len() > 10
            && p.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '=')
    })
}

fn has_connection_credentials(s: &str) -> bool {
    let lower = s.to_lowercase();
    // URI format: scheme://user:password@host. We must scan every
    // URI in the input, not just the first one — clipboard payloads
    // can be a multi-line text/uri-list where the first `://` is an
    // innocent `file://` and a later line is the real
    // `postgres://user:pass@host/db`.
    for part in lower.split_whitespace() {
        if let Some(scheme_end) = part.find("://") {
            let after = &part[scheme_end + 3..];
            if let Some(at_pos) = after.find('@') {
                let userinfo = &after[..at_pos];
                if userinfo.contains(':') {
                    return true;
                }
            }
        }
    }
    // Key=value connection strings
    (lower.contains("password=") || lower.contains("pwd="))
        && (lower.contains("server=")
            || lower.contains("host=")
            || lower.contains("://")
            || lower.contains("data source="))
}

fn is_api_key(s: &str) -> bool {
    let prefixes = [
        "sk-",         // OpenAI
        "sk_",         // Stripe
        "pk_",         // Stripe public
        "ghp_",        // GitHub PAT
        "gho_",        // GitHub OAuth
        "ghs_",        // GitHub App
        "github_pat_", // GitHub fine-grained
        "xoxb-",       // Slack bot
        "xoxp-",       // Slack user
        "AKIA",        // AWS access key
        "Bearer ",     // Auth headers
        "token ",      // Generic tokens
        "glpat-",      // GitLab PAT
        "npm_",        // npm token
    ];
    prefixes.iter().any(|p| s.starts_with(p))
}

fn is_otp_code(s: &str) -> bool {
    // Exactly 6-8 digits, nothing else
    let len = s.len();
    (6..=8).contains(&len) && s.chars().all(|c| c.is_ascii_digit())
}

fn is_hex_token(s: &str) -> bool {
    // 32+ character hex string (MD5, SHA, tokens)
    s.len() >= 32 && s.len() <= 128 && !s.contains(' ') && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn looks_like_password(s: &str) -> bool {
    // Single line, reasonable length, mixed char classes (letters + digits + symbols)
    if s.contains('\n') || s.len() < 8 || s.len() > 128 {
        return false;
    }
    // Exclude URLs and paths
    if s.contains("://") || s.starts_with('/') || s.starts_with("http") {
        return false;
    }
    let has_upper = s.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = s.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    let has_special = s
        .chars()
        .any(|c| !c.is_alphanumeric() && !c.is_whitespace());
    let no_spaces = !s.contains(' ');

    // Needs at least 3 character classes and no spaces (password-like)
    let classes = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();
    no_spaces && classes >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_content() {
        assert!(!check_sensitivity("Hello world").is_sensitive);
        assert!(!check_sensitivity("Some normal text").is_sensitive);
        assert!(!check_sensitivity("12345").is_sensitive); // too short for OTP
    }

    #[test]
    fn test_otp_detection() {
        assert!(check_sensitivity("123456").is_sensitive);
        assert!(check_sensitivity("98765432").is_sensitive);
        assert!(!check_sensitivity("12345").is_sensitive);
        assert!(!check_sensitivity("123456789").is_sensitive);
    }

    #[test]
    fn test_jwt_detection() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        assert!(check_sensitivity(jwt).is_sensitive);
    }

    #[test]
    fn test_api_key_detection() {
        assert!(check_sensitivity("sk-abc123xyz").is_sensitive);
        assert!(check_sensitivity("ghp_1234567890abcdef").is_sensitive);
        assert!(check_sensitivity("AKIAIOSFODNN7EXAMPLE").is_sensitive);
        assert!(check_sensitivity("glpat-xxxxxxxxxxxxxxxxxxxx").is_sensitive);
    }

    #[test]
    fn test_private_key_detection() {
        let key = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAK...\n-----END RSA PRIVATE KEY-----";
        assert!(check_sensitivity(key).is_sensitive);
    }

    #[test]
    fn test_connection_string() {
        assert!(
            check_sensitivity("Server=localhost;Database=mydb;Password=secret123").is_sensitive
        );
        assert!(check_sensitivity("postgresql://user:pass@host:5432/db").is_sensitive);
        assert!(check_sensitivity("mysql://admin:secret@db.example.com/mydb").is_sensitive);
        // URL with password= param
        assert!(check_sensitivity("host=localhost password=secret").is_sensitive);
        // Plain URL without credentials should not trigger
        assert!(!check_sensitivity("https://example.com/page").is_sensitive);
    }

    #[test]
    fn test_connection_string_in_text_uri_list() {
        // text/uri-list with a leading innocent file:// line and a
        // later credentialed postgres:// line. The first
        // `://` is file://; the detector must still spot the
        // credentialed URI in the next line.
        let list = "file:///home/me/db.dump\npostgresql://admin:secret@db.example.com/x";
        assert!(check_sensitivity(list).is_sensitive);
    }

    #[test]
    fn test_hex_token() {
        assert!(check_sensitivity("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4").is_sensitive);
        assert!(!check_sensitivity("abcdef").is_sensitive);
    }

    #[test]
    fn test_password_like() {
        assert!(check_sensitivity("MyP@ssw0rd!").is_sensitive);
        assert!(check_sensitivity("Str0ng#Pass").is_sensitive);
        assert!(!check_sensitivity("hello world").is_sensitive);
        assert!(!check_sensitivity("alllowercase").is_sensitive);
    }

    // ── HTML helper ─────────────────────────────────────────────────────

    #[test]
    fn test_strip_html_tags_basic() {
        assert_eq!(strip_html_tags("<b>hi</b>"), "hi");
        assert_eq!(strip_html_tags("<p>hello <i>world</i></p>"), "hello world");
        assert_eq!(strip_html_tags("plain text"), "plain text");
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn test_strip_html_tags_drops_script_and_style() {
        let html = "<div>before<script>alert('x')</script>after</div>";
        let stripped = strip_html_tags(html);
        assert!(!stripped.contains("alert"));
        assert!(stripped.contains("before"));
        assert!(stripped.contains("after"));

        let html2 = "<p>ok<style>body{color:red}</style>done</p>";
        let stripped2 = strip_html_tags(html2);
        assert!(!stripped2.contains("color"));
        assert!(stripped2.contains("ok"));
        assert!(stripped2.contains("done"));
    }

    #[test]
    fn test_strip_html_tags_drops_comments() {
        let html = "before<!-- secret: hunter2 -->after";
        let stripped = strip_html_tags(html);
        assert!(!stripped.contains("secret"));
        assert!(stripped.contains("before"));
        assert!(stripped.contains("after"));
    }

    #[test]
    fn test_check_sensitive_html_safe() {
        let check = check_sensitive_html("<p>Hello world</p>", "Hello world");
        assert!(!check.is_sensitive);
    }

    #[test]
    fn test_check_sensitive_html_secret_in_plain_text() {
        let html = "<p>Copy me</p>";
        let plain = "ghp_1234567890abcdefghij";
        let check = check_sensitive_html(html, plain);
        assert!(check.is_sensitive, "plain text companion must trigger");
    }

    #[test]
    fn test_check_sensitive_html_secret_in_stripped_html() {
        // No plain-text companion, but the secret sits in a tag attribute.
        // The value uses a strong password-like pattern that the
        // existing detector classifies as sensitive.
        let html = r#"<form><input type="password" value="MyP@ssw0rd!" /></form>"#;
        let check = check_sensitive_html(html, "");
        assert!(check.is_sensitive, "secret in attribute value must trigger");
    }

    #[test]
    fn test_check_sensitive_html_secret_in_html_comment() {
        // The full comment body is a password-like string, so the
        // comment-content branch of the detector sees it as sensitive.
        let html = "<!-- MyP@ssw0rd! -->";
        let check = check_sensitive_html(html, "");
        assert!(check.is_sensitive, "secret in HTML comment must trigger");
    }

    #[test]
    fn test_check_sensitive_html_safe_attribute() {
        // HTML with no secrets anywhere.
        let html = r#"<a href="https://example.com">link</a>"#;
        let check = check_sensitive_html(html, "link");
        assert!(!check.is_sensitive);
    }

    #[test]
    fn test_check_sensitive_html_credential_uri() {
        // URI with embedded credentials in an attribute (e.g. an
        // <img src="postgres://user:pass@host/db"> payload).
        let html = r#"<img src="postgresql://admin:secret@db.example.com/x" alt="" />"#;
        let check = check_sensitive_html(html, "");
        assert!(check.is_sensitive);
    }

    #[test]
    fn test_extract_attribute_values_basic() {
        let html = r#"<a href="https://example.com" title='hi'>x</a>"#;
        let values = extract_attribute_values(html);
        assert!(values.contains(&"https://example.com".to_string()));
        assert!(values.contains(&"hi".to_string()));
    }

    #[test]
    fn test_extract_attribute_values_skips_empty() {
        let html = r#"<img src="" alt="hi" />"#;
        let values = extract_attribute_values(html);
        assert_eq!(values, vec!["hi".to_string()]);
    }

    #[test]
    fn test_extract_html_comments_basic() {
        let html = "before<!-- one -->mid<!-- two -->after";
        let comments = extract_html_comments(html);
        assert_eq!(comments, vec![" one ".to_string(), " two ".to_string()]);
    }

    #[test]
    fn test_extract_html_comments_no_close() {
        // Unterminated comment: nothing is extracted.
        let html = "before<!-- unterminated";
        let comments = extract_html_comments(html);
        assert!(comments.is_empty());
    }
}
