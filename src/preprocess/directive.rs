//! Directive parsing (%%{...}%%).

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value as JsonValue;

use crate::config::MermaidConfig;

/// Regex for matching directive content (type: value or just type).
static DIRECTIVE_CONTENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(\w+)\s*(?::\s*(.*))?$").unwrap()
});

/// Types of directives supported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveType {
    /// Initialize/init directive for configuration.
    Init,
    /// Wrap directive to enable text wrapping.
    Wrap,
    /// Unknown directive type.
    Unknown(String),
}

impl DirectiveType {
    /// Parses a directive type from a string.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "init" | "initialize" => DirectiveType::Init,
            "wrap" => DirectiveType::Wrap,
            _ => DirectiveType::Unknown(s.to_string()),
        }
    }
}

/// A parsed directive.
#[derive(Debug, Clone)]
pub struct Directive {
    /// The type of directive.
    pub directive_type: DirectiveType,
    /// The arguments (parsed JSON for init, or raw string).
    pub args: Option<JsonValue>,
}

/// Result of parsing all directives.
#[derive(Debug, Clone)]
pub struct DirectiveResult {
    /// Text with directives removed.
    pub text: String,
    /// Configuration extracted from init directives.
    pub config: MermaidConfig,
    /// Whether wrap was enabled.
    pub wrap: bool,
}

impl Default for DirectiveResult {
    fn default() -> Self {
        Self {
            text: String::new(),
            config: MermaidConfig::default(),
            wrap: false,
        }
    }
}

/// Find all directive spans in text (start, end positions).
fn find_directive_spans(text: &str) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut pos = 0;

    while let Some(start) = text[pos..].find("%%{") {
        let abs_start = pos + start;
        // Find the matching }%%
        if let Some(end_offset) = text[abs_start..].find("}%%") {
            let abs_end = abs_start + end_offset + 3; // Include }%%
            let content = &text[abs_start + 3..abs_start + end_offset];
            spans.push((abs_start, abs_end, content.to_string()));
            pos = abs_end;
        } else {
            // No closing }%%, skip this occurrence
            pos = abs_start + 3;
        }
    }

    spans
}

/// Parses a single directive from text.
///
/// # Example
///
/// ```
/// use mermaid_linter::preprocess::parse_directive;
///
/// let directive = parse_directive("%%{init: {\"theme\": \"dark\"}}%%");
/// assert!(directive.is_some());
/// ```
pub fn parse_directive(text: &str) -> Option<Directive> {
    // Extract content between %%{ and }%%
    let content = if text.starts_with("%%{") && text.ends_with("}%%") {
        &text[3..text.len() - 3]
    } else {
        return None;
    };

    parse_directive_content(content)
}

/// Parse directive content (without the %%{ and }%% markers).
fn parse_directive_content(content: &str) -> Option<Directive> {
    let caps = DIRECTIVE_CONTENT_REGEX.captures(content)?;

    let type_str = caps.get(1)?.as_str();
    let directive_type = DirectiveType::from_str(type_str);

    let args = if let Some(args_match) = caps.get(2) {
        let args_str = args_match.as_str().trim();
        if args_str.is_empty() {
            None
        } else {
            // Try to parse as JSON
            serde_json::from_str(args_str).ok()
        }
    } else {
        None
    };

    Some(Directive {
        directive_type,
        args,
    })
}

/// Extracts all directives from text and returns processed text.
///
/// # Example
///
/// ```ignore
/// use mermaid_linter::preprocess::directive::extract_directives;
///
/// let text = r#"%%{init: {"theme": "dark"}}%%
/// %%{wrap}%%
/// graph TD
///     A --> B
/// "#;
///
/// let result = extract_directives(text);
/// assert!(result.wrap);
/// assert!(result.text.contains("graph TD"));
/// ```
pub fn extract_directives(text: &str) -> DirectiveResult {
    let mut result = DirectiveResult::default();
    let mut init_configs: Vec<MermaidConfig> = Vec::new();

    let spans = find_directive_spans(text);

    // Process each directive
    for (_, _, content) in &spans {
        if let Some(directive) = parse_directive_content(content) {
            match directive.directive_type {
                DirectiveType::Init => {
                    if let Some(JsonValue::Object(obj)) = directive.args {
                        if let Ok(config) =
                            serde_json::from_value::<MermaidConfig>(JsonValue::Object(obj))
                        {
                            init_configs.push(config);
                        }
                    }
                }
                DirectiveType::Wrap => {
                    result.wrap = true;
                }
                DirectiveType::Unknown(_) => {
                    // Ignore unknown directives
                }
            }
        }
    }

    // Merge all init configs
    for config in init_configs {
        result.config.merge(&config);
    }

    // Remove all directives from text
    let mut processed = text.to_string();
    // Remove from end to start to preserve positions
    for (start, end, _) in spans.into_iter().rev() {
        processed.replace_range(start..end, "");
    }

    result.text = processed;

    result
}

/// Removes all directives from text.
#[allow(dead_code)]
pub fn remove_directives(text: &str) -> String {
    let spans = find_directive_spans(text);
    let mut processed = text.to_string();

    // Remove from end to start to preserve positions
    for (start, end, _) in spans.into_iter().rev() {
        processed.replace_range(start..end, "");
    }

    processed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_init_directive() {
        let text = r#"%%{init: {"theme": "dark"}}%%"#;
        let directive = parse_directive(text).unwrap();

        assert_eq!(directive.directive_type, DirectiveType::Init);
        assert!(directive.args.is_some());
    }

    #[test]
    fn test_parse_wrap_directive() {
        let text = "%%{wrap}%%";
        let directive = parse_directive(text).unwrap();

        assert_eq!(directive.directive_type, DirectiveType::Wrap);
    }

    #[test]
    fn test_parse_initialize_directive() {
        let text = r#"%%{initialize: {"logLevel": 1}}%%"#;
        let directive = parse_directive(text).unwrap();

        assert_eq!(directive.directive_type, DirectiveType::Init);
    }

    #[test]
    fn test_extract_directives() {
        let text = r#"%%{init: {"flowchart": {"defaultRenderer": "elk"}}}%%
%%{wrap}%%
graph TD
    A --> B
"#;
        let result = extract_directives(text);

        assert!(result.wrap);
        assert_eq!(
            result.config.flowchart.default_renderer,
            Some("elk".to_string())
        );
        assert!(result.text.contains("graph TD"));
        assert!(!result.text.contains("%%{"));
    }

    #[test]
    fn test_remove_directives() {
        let text = "%%{wrap}%%\ngraph TD\n    A --> B";
        let result = remove_directives(text);

        assert!(!result.contains("%%{"));
        assert!(result.contains("graph TD"));
    }

    #[test]
    fn test_multiple_init_directives() {
        let text = r#"%%{init: {"wrap": true}}%%
%%{init: {"flowchart": {"defaultRenderer": "dagre-wrapper"}}}%%
graph TD
"#;
        let result = extract_directives(text);

        assert!(result.config.wrap);
        assert_eq!(
            result.config.flowchart.default_renderer,
            Some("dagre-wrapper".to_string())
        );
    }
}
