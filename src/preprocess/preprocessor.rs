//! Main preprocessor that orchestrates all preprocessing steps.

use thiserror::Error;

use super::comments::remove_comments;
use super::directive::extract_directives;
use super::frontmatter::extract_frontmatter;
use super::normalize::normalize_text;
use crate::config::MermaidConfig;

/// Errors that can occur during preprocessing.
#[derive(Debug, Error)]
pub enum PreprocessError {
    /// Error parsing frontmatter.
    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterError(String),

    /// Error parsing directive.
    #[error("Failed to parse directive: {0}")]
    DirectiveError(String),
}

/// Result of preprocessing.
#[derive(Debug, Clone)]
pub struct PreprocessResult {
    /// The preprocessed code ready for parsing.
    pub code: String,
    /// Title extracted from frontmatter.
    pub title: Option<String>,
    /// Merged configuration from frontmatter and directives.
    pub config: MermaidConfig,
}

/// Preprocessor for Mermaid diagram text.
///
/// The preprocessor performs these steps in order:
/// 1. Normalize text (CRLF -> LF, HTML attribute quotes)
/// 2. Extract frontmatter (YAML at start of document)
/// 3. Extract directives (%%{...}%%)
/// 4. Remove comments (%% ...)
#[derive(Debug, Clone, Default)]
pub struct Preprocessor {
    // Future: options for preprocessing behavior
}

impl Preprocessor {
    /// Creates a new preprocessor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Preprocesses Mermaid diagram text.
    ///
    /// # Example
    ///
    /// ```
    /// use mermaid_linter::preprocess::Preprocessor;
    ///
    /// let preprocessor = Preprocessor::new();
    /// let result = preprocessor.preprocess(r#"---
    /// title: My Diagram
    /// ---
    /// %% This is a comment
    /// graph TD
    ///     A --> B
    /// "#).unwrap();
    ///
    /// assert_eq!(result.title, Some("My Diagram".to_string()));
    /// assert!(result.code.contains("graph TD"));
    /// ```
    pub fn preprocess(&self, text: &str) -> Result<PreprocessResult, PreprocessError> {
        // Step 1: Normalize text
        let normalized = normalize_text(text);

        // Step 2: Extract frontmatter
        let frontmatter_result = extract_frontmatter(&normalized);
        let mut config = frontmatter_result.config;

        // Handle displayMode -> gantt.displayMode
        if let Some(display_mode) = &frontmatter_result.display_mode {
            config.gantt.display_mode = Some(display_mode.clone());
        }

        // Step 3: Extract and process directives
        let directive_result = extract_directives(&frontmatter_result.text);

        // Merge directive config into frontmatter config
        config.merge(&directive_result.config);

        // Handle wrap directive
        if directive_result.wrap {
            config.wrap = true;
        }

        // Step 4: Remove comments
        let code = remove_comments(&directive_result.text);

        Ok(PreprocessResult {
            code,
            title: frontmatter_result.title,
            config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_simple() {
        let preprocessor = Preprocessor::new();
        let result = preprocessor.preprocess("graph TD\n    A --> B").unwrap();

        assert_eq!(result.code, "graph TD\n    A --> B");
        assert!(result.title.is_none());
    }

    #[test]
    fn test_preprocess_with_frontmatter() {
        let preprocessor = Preprocessor::new();
        let text = "---\ntitle: Test\n---\ngraph TD\n    A --> B";
        let result = preprocessor.preprocess(text).unwrap();

        assert_eq!(result.title, Some("Test".to_string()));
        assert!(result.code.starts_with("graph TD"));
    }

    #[test]
    fn test_preprocess_with_directives() {
        let preprocessor = Preprocessor::new();
        let text = r#"%%{init: {"flowchart": {"defaultRenderer": "elk"}}}%%
graph TD
    A --> B
"#;
        let result = preprocessor.preprocess(text).unwrap();

        assert_eq!(
            result.config.flowchart.default_renderer,
            Some("elk".to_string())
        );
        assert!(result.code.contains("graph TD"));
    }

    #[test]
    fn test_preprocess_with_comments() {
        let preprocessor = Preprocessor::new();
        let text = "%% Comment\ngraph TD\n    %% Another comment\n    A --> B";
        let result = preprocessor.preprocess(text).unwrap();

        assert!(!result.code.contains("Comment"));
        assert!(result.code.contains("A --> B"));
    }

    #[test]
    fn test_preprocess_full() {
        let preprocessor = Preprocessor::new();
        let text = r#"---
title: Full Test
config:
  flowchart:
    defaultRenderer: dagre-wrapper
---
%%{wrap}%%
%% This is a comment
graph TD
    A --> B
"#;
        let result = preprocessor.preprocess(text).unwrap();

        assert_eq!(result.title, Some("Full Test".to_string()));
        assert!(result.config.wrap);
        assert_eq!(
            result.config.flowchart.default_renderer,
            Some("dagre-wrapper".to_string())
        );
        assert!(!result.code.contains("comment"));
        assert!(result.code.contains("graph TD"));
    }

    #[test]
    fn test_preprocess_crlf_normalization() {
        let preprocessor = Preprocessor::new();
        let text = "graph TD\r\n    A --> B\r\n    B --> C";
        let result = preprocessor.preprocess(text).unwrap();

        assert!(!result.code.contains('\r'));
        assert!(result.code.contains('\n'));
    }

    #[test]
    fn test_preprocess_html_attribute_normalization() {
        let preprocessor = Preprocessor::new();
        let text = r#"graph TD
    A["<span class="foo">text</span>"] --> B
"#;
        let result = preprocessor.preprocess(text).unwrap();

        // Double quotes in HTML attributes should be converted to single quotes
        assert!(result.code.contains("class='foo'"));
    }

    #[test]
    fn test_preprocess_display_mode() {
        let preprocessor = Preprocessor::new();
        let text = "---\ndisplayMode: compact\n---\ngantt\n    title Test";
        let result = preprocessor.preprocess(text).unwrap();

        assert_eq!(result.config.gantt.display_mode, Some("compact".to_string()));
    }
}
