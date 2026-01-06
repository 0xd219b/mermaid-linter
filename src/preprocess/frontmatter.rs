//! YAML frontmatter extraction and parsing.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::config::MermaidConfig;

/// Regex for matching Jekyll-style frontmatter blocks.
/// Matches: ---\n<yaml content>\n---
static FRONTMATTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Note: JS uses /^-{3}\s*[\n\r](.*?)[\n\r]-{3}\s*[\n\r]+/s
    // The 's' flag makes '.' match newlines
    Regex::new(r"^-{3}\s*[\n\r]([\s\S]*?)[\n\r]-{3}\s*[\n\r]+").unwrap()
});

/// Result of frontmatter extraction.
#[derive(Debug, Clone)]
pub struct FrontmatterResult {
    /// Text with frontmatter removed.
    pub text: String,
    /// Extracted title, if present.
    pub title: Option<String>,
    /// Extracted display mode (for Gantt charts).
    pub display_mode: Option<String>,
    /// Extracted configuration.
    pub config: MermaidConfig,
}

impl Default for FrontmatterResult {
    fn default() -> Self {
        Self {
            text: String::new(),
            title: None,
            display_mode: None,
            config: MermaidConfig::default(),
        }
    }
}

/// Extracts and parses YAML frontmatter from text.
///
/// Frontmatter is YAML bounded by `---` blocks at the start of the text.
/// Only `title`, `displayMode`, and `config` fields are supported.
///
/// # Example
///
/// ```
/// use mermaid_linter::preprocess::extract_frontmatter;
///
/// let text = r#"---
/// title: My Diagram
/// config:
///   flowchart:
///     defaultRenderer: elk
/// ---
/// graph TD
///     A --> B
/// "#;
///
/// let result = extract_frontmatter(text);
/// assert_eq!(result.title, Some("My Diagram".to_string()));
/// assert!(result.text.starts_with("graph TD"));
/// ```
pub fn extract_frontmatter(text: &str) -> FrontmatterResult {
    let Some(captures) = FRONTMATTER_REGEX.captures(text) else {
        return FrontmatterResult {
            text: text.to_string(),
            ..Default::default()
        };
    };

    let full_match = captures.get(0).unwrap();
    let yaml_content = captures.get(1).map(|m| m.as_str()).unwrap_or("");

    // Parse YAML
    let parsed: serde_yaml::Value = match serde_yaml::from_str(yaml_content) {
        Ok(v) => v,
        Err(_) => {
            // If YAML parsing fails, return original text
            return FrontmatterResult {
                text: text.to_string(),
                ..Default::default()
            };
        }
    };

    // Ensure it's an object
    let parsed = match parsed {
        serde_yaml::Value::Mapping(m) => m,
        _ => {
            return FrontmatterResult {
                text: text[full_match.end()..].to_string(),
                ..Default::default()
            };
        }
    };

    let mut result = FrontmatterResult {
        text: text[full_match.end()..].to_string(),
        ..Default::default()
    };

    // Extract title
    if let Some(serde_yaml::Value::String(title)) = parsed.get("title") {
        result.title = Some(title.clone());
    }

    // Extract displayMode
    if let Some(display_mode) = parsed.get("displayMode") {
        result.display_mode = Some(display_mode.as_str().unwrap_or("").to_string());
    }

    // Extract config
    if let Some(config_value) = parsed.get("config") {
        if let Ok(config) = serde_yaml::from_value(config_value.clone()) {
            result.config = config;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_frontmatter() {
        let text = "graph TD\n    A --> B";
        let result = extract_frontmatter(text);

        assert_eq!(result.text, text);
        assert!(result.title.is_none());
    }

    #[test]
    fn test_simple_frontmatter() {
        let text = "---\ntitle: Test Diagram\n---\ngraph TD\n    A --> B";
        let result = extract_frontmatter(text);

        assert_eq!(result.title, Some("Test Diagram".to_string()));
        assert!(result.text.starts_with("graph TD"));
    }

    #[test]
    fn test_frontmatter_with_config() {
        let text = r#"---
title: My Diagram
config:
  flowchart:
    defaultRenderer: elk
---
graph TD
    A --> B
"#;
        let result = extract_frontmatter(text);

        assert_eq!(result.title, Some("My Diagram".to_string()));
        assert_eq!(
            result.config.flowchart.default_renderer,
            Some("elk".to_string())
        );
    }

    #[test]
    fn test_frontmatter_with_display_mode() {
        let text = "---\ndisplayMode: compact\n---\ngantt\n    title Test";
        let result = extract_frontmatter(text);

        assert_eq!(result.display_mode, Some("compact".to_string()));
    }

    #[test]
    fn test_invalid_yaml_frontmatter() {
        let text = "---\n: invalid yaml [\n---\ngraph TD\n    A --> B";
        let result = extract_frontmatter(text);

        // Should return original text on invalid YAML
        assert_eq!(result.text, text);
    }

    #[test]
    fn test_frontmatter_not_at_start() {
        let text = "some text\n---\ntitle: Test\n---\ngraph TD";
        let result = extract_frontmatter(text);

        // Frontmatter must be at the start
        assert_eq!(result.text, text);
        assert!(result.title.is_none());
    }
}
