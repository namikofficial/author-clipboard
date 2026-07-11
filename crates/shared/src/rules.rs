//! Ordered, pure capture-rule evaluation.
use serde::{Deserialize, Serialize};

/// Action applied to the first matching rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureRuleAction {
    Ignore,
    ForceSensitive,
    Tag { tag: String },
}

/// One ordered capture rule. Empty match fields are wildcards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureRule {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub source_app: Option<String>,
    #[serde(default)]
    pub mime_prefix: Option<String>,
    #[serde(default)]
    pub content_contains: Option<String>,
    pub action: CaptureRuleAction,
}
fn default_true() -> bool {
    true
}

/// Rule validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuleError {
    #[error("capture rule name must not be blank")]
    BlankName,
    #[error("capture rule matcher must not be blank")]
    BlankMatcher,
    #[error("capture rule tag must not be blank")]
    BlankTag,
}

/// Validate a rule before saving configuration.
pub fn validate(rule: &CaptureRule) -> Result<(), RuleError> {
    if rule.name.trim().is_empty() {
        return Err(RuleError::BlankName);
    }
    if rule
        .source_app
        .as_ref()
        .is_some_and(|v| v.trim().is_empty())
        || rule
            .mime_prefix
            .as_ref()
            .is_some_and(|v| v.trim().is_empty())
        || rule.content_contains.as_ref().is_some_and(String::is_empty)
    {
        return Err(RuleError::BlankMatcher);
    }
    if matches!(&rule.action,CaptureRuleAction::Tag{tag} if tag.trim().is_empty()) {
        return Err(RuleError::BlankTag);
    }
    Ok(())
}

/// Return the first enabled matching action; configuration order is precedence.
pub fn evaluate<'a>(
    rules: &'a [CaptureRule],
    content: &str,
    mime: &str,
    source_app: Option<&str>,
) -> Option<&'a CaptureRuleAction> {
    rules
        .iter()
        .filter(|rule| rule.enabled && validate(rule).is_ok())
        .find(|rule| {
            rule.source_app.as_ref().is_none_or(|expected| {
                source_app.is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
            }) && rule
                .mime_prefix
                .as_ref()
                .is_none_or(|prefix| mime.starts_with(prefix))
                && rule
                    .content_contains
                    .as_ref()
                    .is_none_or(|needle| content.contains(needle))
        })
        .map(|rule| &rule.action)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(name: &str, needle: &str, action: CaptureRuleAction) -> CaptureRule {
        CaptureRule {
            name: name.into(),
            enabled: true,
            source_app: None,
            mime_prefix: None,
            content_contains: Some(needle.into()),
            action,
        }
    }
    #[test]
    fn first_match_wins_and_disabled_is_skipped() {
        let mut first = rule("first", "token", CaptureRuleAction::Ignore);
        first.enabled = false;
        let rules = [
            first,
            rule("second", "token", CaptureRuleAction::ForceSensitive),
            rule("third", "token", CaptureRuleAction::Ignore),
        ];
        assert_eq!(
            evaluate(&rules, "a token", "text/plain", None),
            Some(&CaptureRuleAction::ForceSensitive)
        );
    }
    #[test]
    fn matches_app_mime_and_content() {
        let rule = CaptureRule {
            name: "browser secret".into(),
            enabled: true,
            source_app: Some("Firefox".into()),
            mime_prefix: Some("text/".into()),
            content_contains: Some("secret".into()),
            action: CaptureRuleAction::Ignore,
        };
        assert!(evaluate(&[rule], "secret", "text/plain", Some("firefox")).is_some());
    }
    #[test]
    fn validates_blank_fields() {
        let r = rule("", "x", CaptureRuleAction::Ignore);
        assert_eq!(validate(&r), Err(RuleError::BlankName));
    }
}
