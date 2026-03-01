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
    // Database connection strings
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
        assert!(!check_sensitivity("postgresql://user:pass@host:5432/db").is_sensitive);
        // URL with password= param
        assert!(check_sensitivity("host=localhost password=secret").is_sensitive);
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
}
