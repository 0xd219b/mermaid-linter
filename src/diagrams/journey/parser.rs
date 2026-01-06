//! Parser for User Journey diagrams.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};

use super::lexer::{tokenize, JourneyToken, Token};

/// Parser for User Journey diagrams.
pub struct JourneyParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> JourneyParser<'a> {
    /// Create a new parser.
    pub fn new(source: &'a str) -> Self {
        Self {
            tokens: tokenize(source),
            pos: 0,
            source,
            diagnostics: Vec::new(),
        }
    }

    /// Parse the Journey diagram.
    pub fn parse(&mut self) -> Result<Ast, Vec<Diagnostic>> {
        let start_span = Span::new(0, self.source.len());
        let mut root = AstNode::new(NodeKind::Root, start_span);

        // Skip any leading whitespace/newlines
        self.skip_newlines();

        // Parse the journey declaration
        if let Some(decl) = self.parse_declaration() {
            root.add_child(decl);
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ExpectedToken,
                "Expected 'journey'".to_string(),
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

    /// Parse the journey declaration.
    fn parse_declaration(&mut self) -> Option<AstNode> {
        if !self.check(&JourneyToken::Journey) {
            return None;
        }

        let start = self.current_span().start;
        self.advance(); // consume 'journey'
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::DiagramDeclaration, Span::new(start, end));
        node.text = Some("journey".to_string());

        Some(node)
    }

    /// Parse a statement.
    fn parse_statement(&mut self) -> Option<AstNode> {
        self.skip_newlines();

        if self.is_at_end() {
            return None;
        }

        // Check for title
        if self.check(&JourneyToken::Title) {
            return self.parse_title();
        }

        // Check for section
        if self.check(&JourneyToken::Section) {
            return self.parse_section();
        }

        // Check for accessibility
        if self.check(&JourneyToken::AccTitle) || self.check(&JourneyToken::AccDescr) {
            return self.parse_accessibility();
        }

        // Otherwise, try to parse a task
        self.parse_task()
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

    /// Parse section statement.
    fn parse_section(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'section'

        let name = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Subgraph, Span::new(start, end));
        node.add_property("type", "section");
        node.add_property("name", name.trim().to_string());
        Some(node)
    }

    /// Parse accessibility statement.
    fn parse_accessibility(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        let acc_type = if self.check(&JourneyToken::AccTitle) {
            "accTitle"
        } else {
            "accDescr"
        };
        self.advance();

        // Skip colon if present
        if self.check(&JourneyToken::Colon) {
            self.advance();
        }

        // Check for multi-line description
        if self.check(&JourneyToken::OpenBrace) {
            self.advance();
            let mut content = String::new();
            while !self.check(&JourneyToken::CloseBrace) && !self.is_at_end() {
                content.push_str(&self.current_text());
                content.push(' ');
                self.advance();
            }
            if self.check(&JourneyToken::CloseBrace) {
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

    /// Parse a task.
    /// Format: TaskName: score: actors
    fn parse_task(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Collect task name (everything before the first colon)
        let mut task_name = String::new();
        while !self.check(&JourneyToken::Colon) && !self.check(&JourneyToken::Newline) && !self.is_at_end() {
            if !task_name.is_empty() {
                task_name.push(' ');
            }
            task_name.push_str(&self.current_text());
            self.advance();
        }

        if task_name.trim().is_empty() {
            return None;
        }

        let mut node = AstNode::new(NodeKind::Node, Span::new(start, start));
        node.add_property("type", "task");
        node.add_property("name", task_name.trim().to_string());

        // Parse score and actors after colon
        if self.check(&JourneyToken::Colon) {
            self.advance();

            // Get score
            if self.check(&JourneyToken::Number) {
                node.add_property("score", self.current_text());
                self.advance();
            }

            // Check for actors after second colon
            if self.check(&JourneyToken::Colon) {
                self.advance();

                // Collect actors
                let mut actors = Vec::new();
                while !self.check(&JourneyToken::Newline) && !self.is_at_end() {
                    if self.check(&JourneyToken::Identifier) {
                        actors.push(self.current_text());
                        self.advance();
                    } else if self.check(&JourneyToken::Comma) {
                        self.advance();
                    } else {
                        self.advance();
                    }
                }

                if !actors.is_empty() {
                    node.add_property("actors", actors.join(","));
                }
            }
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);
        Some(node)
    }

    /// Consume tokens until newline.
    fn consume_until_newline(&mut self) -> String {
        let mut text = String::new();
        while !self.check(&JourneyToken::Newline) && !self.is_at_end() {
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

    fn check(&self, kind: &JourneyToken) -> bool {
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
        while self.check(&JourneyToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let code = r#"journey
    title My working day
    section Go to work
    Make tea: 5: Me
    Go upstairs: 3: Me"#;

        let mut parser = JourneyParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_multiple_actors() {
        let code = r#"journey
    title My Journey
    section Home
    Wake up: 5: Me
    Do work: 3: Me, Cat"#;

        let mut parser = JourneyParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_multiple_sections() {
        let code = r#"journey
    title Customer Journey
    section Awareness
    See ad: 5: Customer
    section Consideration
    Research product: 4: Customer
    section Purchase
    Buy product: 5: Customer"#;

        let mut parser = JourneyParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_title() {
        let code = r#"journey
    title My User Journey"#;

        let mut parser = JourneyParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_invalid() {
        let code = "not a journey diagram";
        let mut parser = JourneyParser::new(code);
        let result = parser.parse();
        assert!(result.is_err());
    }
}
