//! Pure clipboard text transformations.
use serde::{Deserialize, Serialize};

/// A supported transformation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformKind {
    PlainText,
    MarkdownLink,
    FencedCode { language_hint: Option<String> },
    Quote,
    JsonPretty,
    JsonMinified,
    Redacted,
}

/// Safe failure that never embeds source content.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransformError {
    #[error("this transform requires valid JSON")]
    InvalidJson,
    #[error("Markdown link transform requires a non-empty URL")]
    MissingUrl,
    #[error("sensitive content requires explicit confirmation")]
    SensitiveConfirmationRequired,
}

/// Transform text without mutating the source.
pub fn apply(
    input: &str,
    kind: &TransformKind,
    sensitive: bool,
    confirmed: bool,
) -> Result<String, TransformError> {
    if sensitive && !confirmed && !matches!(kind, TransformKind::Redacted) {
        return Err(TransformError::SensitiveConfirmationRequired);
    }
    match kind {
        TransformKind::PlainText => Ok(strip_html(input)),
        TransformKind::MarkdownLink => {
            let url = input.trim();
            if url.is_empty() {
                Err(TransformError::MissingUrl)
            } else {
                Ok(format!("[{url}]({url})"))
            }
        }
        TransformKind::FencedCode { language_hint } => Ok(format!(
            "```{}\n{}\n```",
            language_hint.as_deref().unwrap_or("").trim(),
            input.trim_end()
        )),
        TransformKind::Quote => Ok(input
            .lines()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n")),
        TransformKind::JsonPretty => serde_json::from_str::<serde_json::Value>(input)
            .and_then(|v| serde_json::to_string_pretty(&v))
            .map_err(|_| TransformError::InvalidJson),
        TransformKind::JsonMinified => serde_json::from_str::<serde_json::Value>(input)
            .and_then(|v| serde_json::to_string(&v))
            .map_err(|_| TransformError::InvalidJson),
        TransformKind::Redacted => Ok("•••••••• Sensitive item".into()),
    }
}
fn strip_html(input: &str) -> String {
    let (mut out, mut tag) = (String::new(), false);
    for ch in input.chars() {
        match ch {
            '<' => tag = true,
            '>' => tag = false,
            _ if !tag => out.push(ch),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_and_privacy() {
        let p = apply(r#"{"a":1}"#, &TransformKind::JsonPretty, false, false).unwrap();
        assert_eq!(
            apply(&p, &TransformKind::JsonMinified, false, false).unwrap(),
            r#"{"a":1}"#
        );
        assert_eq!(
            apply("secret", &TransformKind::Quote, true, false),
            Err(TransformError::SensitiveConfirmationRequired)
        );
        assert!(!apply("secret", &TransformKind::Redacted, true, false)
            .unwrap()
            .contains("secret"));
    }
    #[test]
    fn errors_do_not_echo_input() {
        assert_eq!(
            apply("private bad json", &TransformKind::JsonPretty, false, false),
            Err(TransformError::InvalidJson)
        );
    }
}
