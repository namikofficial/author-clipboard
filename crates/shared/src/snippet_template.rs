//! Strict `{name}` compatibility for command-center snippets.
use crate::template::{self, RenderContext};
use std::fmt::Write as _;

/// Validation or privacy failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SnippetTemplateError {
    #[error("unknown snippet variable: {0}")]
    UnknownVariable(String),
    #[error("snippet source contains sensitive content; confirmation required")]
    SensitiveConfirmationRequired,
    #[error("unclosed snippet variable")]
    UnclosedVariable,
}

/// Expand strict variables; sensitive clipboard/selection sources require confirmation.
pub fn expand(
    input: &str,
    ctx: &RenderContext,
    selection: Option<&str>,
    sensitive: bool,
    confirmed: bool,
) -> Result<(String, Option<usize>), SnippetTemplateError> {
    if sensitive && !confirmed && (input.contains("{clipboard}") || input.contains("{selection}")) {
        return Err(SnippetTemplateError::SensitiveConfirmationRequired);
    }
    Ok(template::render(&canonicalize(input, selection)?, ctx))
}
fn canonicalize(input: &str, selection: Option<&str>) -> Result<String, SnippetTemplateError> {
    let chars: Vec<char> = input.chars().collect();
    let (mut out, mut i) = (String::new(), 0);
    while i < chars.len() {
        if chars[i] == '$' && chars.get(i + 1) == Some(&'{') {
            let end = chars[i + 2..]
                .iter()
                .position(|ch| *ch == '}')
                .ok_or(SnippetTemplateError::UnclosedVariable)?
                + i
                + 2;
            out.extend(&chars[i..=end]);
            i = end + 1;
            continue;
        }
        if chars[i] != '{' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        if chars.get(i + 1) == Some(&'{') {
            let e = chars[i + 2..]
                .windows(2)
                .position(|p| p == ['}', '}'])
                .ok_or(SnippetTemplateError::UnclosedVariable)?
                + i
                + 2;
            out.push('{');
            out.extend(&chars[i + 2..e]);
            out.push('}');
            i = e + 2;
            continue;
        }
        let e = chars[i + 1..]
            .iter()
            .position(|c| *c == '}')
            .ok_or(SnippetTemplateError::UnclosedVariable)?
            + i
            + 1;
        let n: String = chars[i + 1..e].iter().collect();
        match n.as_str() {
            "date" | "time" | "clipboard" => {
                let _ = write!(out, "${{{n}}}");
            }
            "selection" => out.push_str(selection.unwrap_or("")),
            _ => return Err(SnippetTemplateError::UnknownVariable(n)),
        }
        i = e + 1;
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expansion_and_escape() {
        let (v, _) = expand(
            "{{date}} {selection}",
            &RenderContext::default(),
            Some("chosen"),
            false,
            false,
        )
        .unwrap();
        assert_eq!(v, "{date} chosen");
    }
    #[test]
    fn validation_and_privacy() {
        assert!(matches!(
            expand("{wat}", &RenderContext::default(), None, false, false),
            Err(SnippetTemplateError::UnknownVariable(_))
        ));
        assert_eq!(
            expand("{clipboard}", &RenderContext::default(), None, true, false),
            Err(SnippetTemplateError::SensitiveConfirmationRequired)
        );
    }
}
