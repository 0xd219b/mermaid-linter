//! # Mermaid Linter
//!
//! A Rust-based syntax linter for Mermaid diagrams.
//!
//! This library parses Mermaid text without rendering, detects diagram types,
//! validates syntax, produces AST output, and emits diagnostics.
//!
//! ## Features
//!
//! - Parse Mermaid text without rendering
//! - Detect diagram type and validate syntax
//! - Produce AST output for successful parses
//! - Emit diagnostics with location and error category on failure
//! - Support for all major Mermaid diagram types
//!
//! ## Example
//!
//! ```rust
//! use mermaid_linter::{parse, ParseOptions};
//!
//! let code = r#"
//! graph TD
//!     A --> B
//!     B --> C
//! "#;
//!
//! let result = parse(code, None);
//! assert!(result.ok);
//! assert_eq!(result.diagram_type, Some(mermaid_linter::DiagramType::Flowchart));
//! ```

pub mod ast;
pub mod config;
pub mod detector;
pub mod diagnostic;
pub mod diagrams;
pub mod parser;
pub mod preprocess;

// Re-export main types for convenience
pub use ast::{Ast, AstNode, Span};
pub use config::{MermaidConfig, ParseOptions};
pub use detector::DiagramType;
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};

use preprocess::preprocessor::Preprocessor;

/// The result of parsing a Mermaid diagram.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Whether the parse was successful.
    pub ok: bool,
    /// The detected diagram type, if any.
    pub diagram_type: Option<DiagramType>,
    /// The merged configuration from base config, frontmatter, and directives.
    pub config: MermaidConfig,
    /// The AST, if parsing was successful.
    pub ast: Option<Ast>,
    /// Diagnostics (errors and warnings) from parsing.
    pub diagnostics: Vec<Diagnostic>,
    /// The title extracted from frontmatter, if any.
    pub title: Option<String>,
}

impl ParseResult {
    /// Creates a new successful parse result.
    pub fn success(diagram_type: DiagramType, config: MermaidConfig, ast: Ast) -> Self {
        Self {
            ok: true,
            diagram_type: Some(diagram_type),
            config,
            ast: Some(ast),
            diagnostics: Vec::new(),
            title: None,
        }
    }

    /// Creates a new failed parse result.
    pub fn failure(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            ok: false,
            diagram_type: None,
            config: MermaidConfig::default(),
            ast: None,
            diagnostics,
            title: None,
        }
    }

    /// Creates a failed result with a single diagnostic.
    pub fn failure_single(diagnostic: Diagnostic) -> Self {
        Self::failure(vec![diagnostic])
    }

    /// Adds a diagnostic to the result.
    pub fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    /// Sets the title.
    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }
}

/// Parse a Mermaid diagram string.
///
/// This is the main entry point for the linter. It performs:
/// 1. Text preprocessing (normalize line endings, extract frontmatter/directives, remove comments)
/// 2. Diagram type detection
/// 3. Diagram-specific parsing
/// 4. Semantic validation
///
/// # Arguments
///
/// * `code` - The Mermaid diagram source code
/// * `options` - Optional parse options including base configuration
///
/// # Returns
///
/// A `ParseResult` containing the parse status, AST (if successful), and any diagnostics.
pub fn parse(code: &str, options: Option<ParseOptions>) -> ParseResult {
    let options = options.unwrap_or_default();

    // Step 1: Preprocess the text
    let preprocessor = Preprocessor::new();
    let preprocess_result = match preprocessor.preprocess(code) {
        Ok(result) => result,
        Err(e) => {
            return ParseResult::failure_single(Diagnostic::new(
                DiagnosticCode::PreprocessError,
                e.to_string(),
                Severity::Error,
                Span::default(),
            ));
        }
    };

    // Merge config: base_config <- frontmatter config <- directive config
    let mut config = options.base_config.unwrap_or_default();
    config.merge(&preprocess_result.config);

    // Step 2: Detect diagram type
    let diagram_type = match detector::detect_type(&preprocess_result.code, &config) {
        Some(dt) => dt,
        None => {
            return ParseResult::failure_single(Diagnostic::new(
                DiagnosticCode::UnknownDiagram,
                "Could not detect diagram type".to_string(),
                Severity::Error,
                Span::default(),
            ))
            .with_title(preprocess_result.title);
        }
    };

    // Handle special diagram types
    match diagram_type {
        DiagramType::Error => {
            return ParseResult::failure_single(Diagnostic::new(
                DiagnosticCode::ParserError,
                "Error diagram type".to_string(),
                Severity::Error,
                Span::default(),
            ))
            .with_title(preprocess_result.title);
        }
        DiagramType::BadFrontmatter => {
            return ParseResult::failure_single(Diagnostic::new(
                DiagnosticCode::FrontmatterParseError,
                "Diagrams beginning with --- are not valid. If you were trying to use a YAML front-matter, please ensure that you've correctly opened and closed the YAML front-matter with un-indented `---` blocks".to_string(),
                Severity::Error,
                Span::default(),
            ))
            .with_title(preprocess_result.title);
        }
        _ => {}
    }

    // Step 3: Encode entities for flowchart-related diagrams
    let code_to_parse = if diagram_type.needs_entity_encoding() {
        preprocess::encode_entities(&preprocess_result.code)
    } else {
        preprocess_result.code.clone()
    };

    // Step 4: Parse with diagram-specific parser
    let parse_result = parser::parse_diagram(diagram_type, &code_to_parse, &config);

    match parse_result {
        Ok(ast) => {
            let mut result = ParseResult::success(diagram_type, config, ast);
            result.title = preprocess_result.title;
            result
        }
        Err(diagnostics) => {
            let mut result = ParseResult::failure(diagnostics);
            result.diagram_type = Some(diagram_type);
            result.config = config;
            result.title = preprocess_result.title;
            result
        }
    }
}

/// Validate a Mermaid diagram string without producing an AST.
///
/// This is a convenience function that only checks if the diagram is valid.
///
/// # Arguments
///
/// * `code` - The Mermaid diagram source code
/// * `options` - Optional parse options
///
/// # Returns
///
/// `true` if the diagram is valid, `false` otherwise.
pub fn validate(code: &str, options: Option<ParseOptions>) -> bool {
    parse(code, options).ok
}

/// Detect the diagram type from a Mermaid diagram string.
///
/// # Arguments
///
/// * `code` - The Mermaid diagram source code
///
/// # Returns
///
/// The detected diagram type, or `None` if the type could not be determined.
pub fn detect_type(code: &str) -> Option<DiagramType> {
    let preprocessor = Preprocessor::new();
    let preprocess_result = preprocessor.preprocess(code).ok()?;
    detector::detect_type(&preprocess_result.code, &MermaidConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flowchart() {
        let code = r#"
graph TD
    A --> B
    B --> C
"#;
        let result = parse(code, None);
        assert!(result.ok);
        assert_eq!(result.diagram_type, Some(DiagramType::Flowchart));
    }

    #[test]
    fn test_parse_sequence() {
        let code = r#"
sequenceDiagram
    Alice->>Bob: Hello
    Bob-->>Alice: Hi
"#;
        let result = parse(code, None);
        assert!(result.ok);
        assert_eq!(result.diagram_type, Some(DiagramType::Sequence));
    }

    #[test]
    fn test_detect_type() {
        assert_eq!(detect_type("graph TD\nA-->B"), Some(DiagramType::Flowchart));
        assert_eq!(
            detect_type("sequenceDiagram\nAlice->>Bob: Hi"),
            Some(DiagramType::Sequence)
        );
        assert_eq!(
            detect_type("classDiagram\nClass01 <|-- Class02"),
            Some(DiagramType::Class)
        );
    }

    #[test]
    fn test_invalid_diagram() {
        let result = parse("this is not a valid diagram", None);
        assert!(!result.ok);
        assert!(!result.diagnostics.is_empty());
    }
}
