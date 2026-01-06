//! Parser traits and common interfaces.

use crate::ast::Ast;
use crate::config::MermaidConfig;
use crate::diagnostic::Diagnostic;

/// Trait for diagram-specific parsers.
///
/// Each diagram type implements this trait to provide its parsing logic.
pub trait DiagramParser {
    /// Parses the given code and returns an AST or diagnostics.
    fn parse(&self, code: &str, config: &MermaidConfig) -> Result<Ast, Vec<Diagnostic>>;

    /// Returns the name of this parser.
    fn name(&self) -> &'static str;

    /// Returns true if this parser supports incremental parsing.
    fn supports_incremental(&self) -> bool {
        false
    }
}

/// A parser that can be validated.
pub trait ValidatingParser: DiagramParser {
    /// Performs semantic validation on the parsed AST.
    fn validate(&self, ast: &Ast, config: &MermaidConfig) -> Vec<Diagnostic>;
}

/// Context provided to parsers during parsing.
#[derive(Debug, Clone)]
pub struct ParseContext<'a> {
    /// The source code being parsed.
    pub source: &'a str,
    /// The configuration.
    pub config: &'a MermaidConfig,
    /// Whether to collect all errors or stop at first.
    pub collect_all_errors: bool,
}

impl<'a> ParseContext<'a> {
    /// Creates a new parse context.
    pub fn new(source: &'a str, config: &'a MermaidConfig) -> Self {
        Self {
            source,
            config,
            collect_all_errors: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestParser;

    impl DiagramParser for TestParser {
        fn parse(&self, code: &str, _config: &MermaidConfig) -> Result<Ast, Vec<Diagnostic>> {
            use crate::ast::{AstNode, NodeKind, Span};

            let root = AstNode::new(NodeKind::Root, Span::new(0, code.len()));
            Ok(Ast::new(root, code.to_string()))
        }

        fn name(&self) -> &'static str {
            "test"
        }
    }

    #[test]
    fn test_parser_trait() {
        let parser = TestParser;
        let result = parser.parse("test", &MermaidConfig::default());
        assert!(result.is_ok());
        assert_eq!(parser.name(), "test");
    }
}
