//! Parser for GitGraph diagrams.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};

use super::lexer::{tokenize, GitGraphToken, Token};

/// Parser for GitGraph diagrams.
pub struct GitGraphParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> GitGraphParser<'a> {
    /// Create a new parser.
    pub fn new(source: &'a str) -> Self {
        Self {
            tokens: tokenize(source),
            pos: 0,
            source,
            diagnostics: Vec::new(),
        }
    }

    /// Parse the GitGraph diagram.
    pub fn parse(&mut self) -> Result<Ast, Vec<Diagnostic>> {
        let start_span = Span::new(0, self.source.len());
        let mut root = AstNode::new(NodeKind::Root, start_span);

        // Skip any leading whitespace/newlines
        self.skip_newlines();

        // Parse the gitGraph declaration
        if let Some(decl) = self.parse_declaration() {
            root.add_child(decl);
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ExpectedToken,
                "Expected 'gitGraph'".to_string(),
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
                self.advance();
            }
        }

        if self.diagnostics.iter().any(|d| d.severity == Severity::Error) {
            Err(self.diagnostics.clone())
        } else {
            Ok(Ast::new(root, self.source.to_string()))
        }
    }

    /// Parse the gitGraph declaration.
    fn parse_declaration(&mut self) -> Option<AstNode> {
        if !self.check(&GitGraphToken::GitGraph) {
            return None;
        }

        let start = self.current_span().start;
        self.advance();

        let mut node = AstNode::new(NodeKind::DiagramDeclaration, Span::new(start, start));
        node.text = Some("gitGraph".to_string());

        // Check for options (LR, TB, etc.)
        if self.check(&GitGraphToken::LR) {
            node.add_property("direction", "LR");
            self.advance();
        } else if self.check(&GitGraphToken::TB) {
            node.add_property("direction", "TB");
            self.advance();
        }

        // Check for colon (optional)
        if self.check(&GitGraphToken::Colon) {
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

        // Check for commit
        if self.check(&GitGraphToken::Commit) {
            return self.parse_commit();
        }

        // Check for branch
        if self.check(&GitGraphToken::Branch) {
            return self.parse_branch();
        }

        // Check for checkout
        if self.check(&GitGraphToken::Checkout) {
            return self.parse_checkout();
        }

        // Check for merge
        if self.check(&GitGraphToken::Merge) {
            return self.parse_merge();
        }

        // Check for cherry-pick
        if self.check(&GitGraphToken::CherryPick) {
            return self.parse_cherry_pick();
        }

        // Check for accessibility
        if self.check(&GitGraphToken::AccTitle) || self.check(&GitGraphToken::AccDescr) {
            return self.parse_accessibility();
        }

        None
    }

    /// Parse commit statement.
    fn parse_commit(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'commit'

        let mut node = AstNode::new(NodeKind::Node, Span::new(start, start));
        node.add_property("type", "commit");

        // Parse commit options
        while !self.check(&GitGraphToken::Newline) && !self.is_at_end() {
            if self.check(&GitGraphToken::Id) {
                self.advance();
                if self.check(&GitGraphToken::Colon) {
                    self.advance();
                }
                if self.check(&GitGraphToken::QuotedString) {
                    let id = self.current_text();
                    node.add_property("id", id[1..id.len() - 1].to_string());
                    self.advance();
                } else if self.check(&GitGraphToken::Identifier) {
                    node.add_property("id", self.current_text());
                    self.advance();
                }
            } else if self.check(&GitGraphToken::Msg) {
                self.advance();
                if self.check(&GitGraphToken::Colon) {
                    self.advance();
                }
                if self.check(&GitGraphToken::QuotedString) {
                    let msg = self.current_text();
                    node.add_property("message", msg[1..msg.len() - 1].to_string());
                    self.advance();
                }
            } else if self.check(&GitGraphToken::Tag) {
                self.advance();
                if self.check(&GitGraphToken::Colon) {
                    self.advance();
                }
                if self.check(&GitGraphToken::QuotedString) {
                    let tag = self.current_text();
                    node.add_property("tag", tag[1..tag.len() - 1].to_string());
                    self.advance();
                }
            } else if self.check(&GitGraphToken::Type) {
                self.advance();
                if self.check(&GitGraphToken::Colon) {
                    self.advance();
                }
                if self.check(&GitGraphToken::Normal) || self.check(&GitGraphToken::Reverse) || self.check(&GitGraphToken::Highlight) {
                    node.add_property("commitType", self.current_text().to_uppercase());
                    self.advance();
                }
            } else {
                self.advance();
            }
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);
        Some(node)
    }

    /// Parse branch statement.
    fn parse_branch(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'branch'

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, start));
        node.add_property("type", "branch");

        // Get branch name
        if self.check(&GitGraphToken::Identifier) {
            node.add_property("name", self.current_text());
            self.advance();
        }

        // Check for order
        if self.check(&GitGraphToken::Order) {
            self.advance();
            if self.check(&GitGraphToken::Colon) {
                self.advance();
            }
            if self.check(&GitGraphToken::Number) {
                node.add_property("order", self.current_text());
                self.advance();
            }
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);
        Some(node)
    }

    /// Parse checkout statement.
    fn parse_checkout(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'checkout'

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, start));
        node.add_property("type", "checkout");

        // Get branch name
        if self.check(&GitGraphToken::Identifier) {
            node.add_property("branch", self.current_text());
            self.advance();
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);
        Some(node)
    }

    /// Parse merge statement.
    fn parse_merge(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'merge'

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, start));
        node.add_property("type", "merge");

        // Get branch name
        if self.check(&GitGraphToken::Identifier) {
            node.add_property("branch", self.current_text());
            self.advance();
        }

        // Parse optional parameters (id, tag, type)
        while !self.check(&GitGraphToken::Newline) && !self.is_at_end() {
            if self.check(&GitGraphToken::Id) {
                self.advance();
                if self.check(&GitGraphToken::Colon) {
                    self.advance();
                }
                if self.check(&GitGraphToken::QuotedString) {
                    let id = self.current_text();
                    node.add_property("id", id[1..id.len() - 1].to_string());
                    self.advance();
                }
            } else if self.check(&GitGraphToken::Tag) {
                self.advance();
                if self.check(&GitGraphToken::Colon) {
                    self.advance();
                }
                if self.check(&GitGraphToken::QuotedString) {
                    let tag = self.current_text();
                    node.add_property("tag", tag[1..tag.len() - 1].to_string());
                    self.advance();
                }
            } else if self.check(&GitGraphToken::Type) {
                self.advance();
                if self.check(&GitGraphToken::Colon) {
                    self.advance();
                }
                if self.check(&GitGraphToken::Normal) || self.check(&GitGraphToken::Reverse) || self.check(&GitGraphToken::Highlight) {
                    node.add_property("commitType", self.current_text().to_uppercase());
                    self.advance();
                }
            } else {
                self.advance();
            }
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);
        Some(node)
    }

    /// Parse cherry-pick statement.
    fn parse_cherry_pick(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'cherry-pick'

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, start));
        node.add_property("type", "cherry-pick");

        // Parse id parameter
        if self.check(&GitGraphToken::Id) {
            self.advance();
            if self.check(&GitGraphToken::Colon) {
                self.advance();
            }
            if self.check(&GitGraphToken::QuotedString) {
                let id = self.current_text();
                node.add_property("id", id[1..id.len() - 1].to_string());
                self.advance();
            }
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);
        Some(node)
    }

    /// Parse accessibility statement.
    fn parse_accessibility(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        let acc_type = if self.check(&GitGraphToken::AccTitle) {
            "accTitle"
        } else {
            "accDescr"
        };
        self.advance();

        if self.check(&GitGraphToken::Colon) {
            self.advance();
        }

        if self.check(&GitGraphToken::OpenBrace) {
            self.advance();
            let mut content = String::new();
            while !self.check(&GitGraphToken::CloseBrace) && !self.is_at_end() {
                content.push_str(&self.current_text());
                content.push(' ');
                self.advance();
            }
            if self.check(&GitGraphToken::CloseBrace) {
                self.advance();
            }
            let end = self.previous_span().end;

            let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
            node.add_property("type", acc_type);
            node.add_property("value", content.trim().to_string());
            return Some(node);
        }

        let value = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", acc_type);
        node.add_property("value", value.trim().to_string());
        Some(node)
    }

    fn consume_until_newline(&mut self) -> String {
        let mut text = String::new();
        while !self.check(&GitGraphToken::Newline) && !self.is_at_end() {
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

    fn check(&self, kind: &GitGraphToken) -> bool {
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
        while self.check(&GitGraphToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let code = r#"gitGraph
    commit
    commit
    branch develop
    checkout develop
    commit"#;

        let mut parser = GitGraphParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_merge() {
        let code = r#"gitGraph
    commit
    branch feature
    checkout feature
    commit
    checkout main
    merge feature"#;

        let mut parser = GitGraphParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_options() {
        let code = r#"gitGraph
    commit id: "1" msg: "Initial commit" tag: "v1.0"
    commit id: "2" type: HIGHLIGHT"#;

        let mut parser = GitGraphParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_invalid() {
        let code = "not a git graph";
        let mut parser = GitGraphParser::new(code);
        let result = parser.parse();
        assert!(result.is_err());
    }
}
