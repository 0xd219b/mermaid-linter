//! Parser for Pie charts.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};

use super::lexer::{tokenize, PieToken, Token};

/// Parser for Pie charts.
pub struct PieParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> PieParser<'a> {
    /// Create a new parser.
    pub fn new(source: &'a str) -> Self {
        Self {
            tokens: tokenize(source),
            pos: 0,
            source,
            diagnostics: Vec::new(),
        }
    }

    /// Parse the Pie chart.
    pub fn parse(&mut self) -> Result<Ast, Vec<Diagnostic>> {
        let start_span = Span::new(0, self.source.len());
        let mut root = AstNode::new(NodeKind::Root, start_span);

        // Skip any leading whitespace/newlines
        self.skip_newlines();

        // Parse the pie declaration
        if let Some(decl) = self.parse_declaration() {
            root.add_child(decl);
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ExpectedToken,
                "Expected 'pie'".to_string(),
                Severity::Error,
                self.current_span(),
            ));
            return Err(self.diagnostics.clone());
        }

        // Parse statements
        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            if let Some(stmt) = self.parse_statement() {
                root.add_child(stmt);
            } else {
                // Skip unknown token
                self.advance();
            }
        }

        if self.diagnostics.iter().any(|d| d.severity == Severity::Error) {
            Err(self.diagnostics.clone())
        } else {
            Ok(Ast::new(root, self.source.to_string()))
        }
    }

    /// Parse the pie declaration.
    fn parse_declaration(&mut self) -> Option<AstNode> {
        if !self.check(&PieToken::Pie) {
            return None;
        }

        let start = self.current_span().start;
        self.advance(); // consume 'pie'

        let mut node = AstNode::new(NodeKind::DiagramDeclaration, Span::new(start, start));
        node.text = Some("pie".to_string());

        // Check for showData option
        if self.check(&PieToken::ShowData) {
            node.add_property("showData", "true");
            self.advance();
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);

        Some(node)
    }

    /// Parse a statement.
    fn parse_statement(&mut self) -> Option<AstNode> {
        self.skip_newlines();

        if self.is_at_end() {
            return None;
        }

        // Check for title
        if self.check(&PieToken::Title) {
            return self.parse_title();
        }

        // Check for accessibility
        if self.check(&PieToken::AccTitle) || self.check(&PieToken::AccDescr) {
            return self.parse_accessibility();
        }

        // Otherwise, try to parse a slice
        self.parse_slice()
    }

    /// Parse title statement.
    fn parse_title(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'title'

        let title = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "title");
        node.add_property("value", title.trim().to_string());
        Some(node)
    }

    /// Parse accessibility statement.
    fn parse_accessibility(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        let acc_type = if self.check(&PieToken::AccTitle) {
            "accTitle"
        } else {
            "accDescr"
        };
        self.advance();

        // Skip colon if present
        if self.check(&PieToken::Colon) {
            self.advance();
        }

        // Check for multi-line description
        if self.check(&PieToken::OpenBrace) {
            self.advance();
            let mut content = String::new();
            while !self.check(&PieToken::CloseBrace) && !self.is_at_end() {
                content.push_str(&self.current_text());
                content.push(' ');
                self.advance();
            }
            if self.check(&PieToken::CloseBrace) {
                self.advance();
            }
            let end = self.previous_span().end;

            let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
            node.add_property("type", acc_type);
            node.add_property("value", content.trim().to_string());
            return Some(node);
        }

        // Single line
        let value = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", acc_type);
        node.add_property("value", value.trim().to_string());
        Some(node)
    }

    /// Parse a pie slice.
    /// Format: "Label" : value
    fn parse_slice(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Get slice label
        let label = if self.check(&PieToken::QuotedString) {
            let text = self.current_text();
            self.advance();
            // Remove quotes
            text[1..text.len() - 1].to_string()
        } else if self.check(&PieToken::Identifier) {
            let text = self.current_text();
            self.advance();
            text
        } else {
            return None;
        };

        // Expect colon
        if !self.check(&PieToken::Colon) {
            return None;
        }
        self.advance();

        // Get value
        let value = if self.check(&PieToken::Number) {
            let v = self.current_text();
            self.advance();
            v
        } else {
            "0".to_string()
        };

        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Node, Span::new(start, end));
        node.add_property("type", "slice");
        node.add_property("label", label);
        node.add_property("value", value);
        Some(node)
    }

    /// Consume tokens until newline.
    fn consume_until_newline(&mut self) -> String {
        let mut text = String::new();
        while !self.check(&PieToken::Newline) && !self.is_at_end() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&self.current_text());
            self.advance();
        }
        text
    }

    // Helper methods

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_text(&self) -> String {
        self.current().map(|t| t.text.clone()).unwrap_or_default()
    }

    fn current_span(&self) -> Span {
        self.current()
            .map(|t| Span::new(t.span.start, t.span.end))
            .unwrap_or(Span::new(self.source.len(), self.source.len()))
    }

    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens
                .get(self.pos - 1)
                .map(|t| Span::new(t.span.start, t.span.end))
                .unwrap_or(Span::new(0, 0))
        } else {
            Span::new(0, 0)
        }
    }

    fn check(&self, kind: &PieToken) -> bool {
        self.current().map(|t| &t.kind == kind).unwrap_or(false)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn skip_newlines(&mut self) {
        while self.check(&PieToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let code = r#"pie
    title Key elements
    "Calcium" : 42.96
    "Potassium" : 50.05"#;

        let mut parser = PieParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_showdata() {
        let code = r#"pie showData
    title Distribution
    "A" : 30
    "B" : 70"#;

        let mut parser = PieParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_multiple_slices() {
        let code = r#"pie
    "Dogs" : 386
    "Cats" : 85
    "Rats" : 15"#;

        let mut parser = PieParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_invalid() {
        let code = "not a pie chart";
        let mut parser = PieParser::new(code);
        let result = parser.parse();
        assert!(result.is_err());
    }
}
