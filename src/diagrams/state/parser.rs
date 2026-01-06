//! State diagram parser implementation.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::config::MermaidConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::traits::DiagramParser;

use super::lexer::{tokenize, PositionedToken, StateToken};
use super::StateType;

/// State diagram parser.
pub struct StateParser;

impl StateParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StateParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramParser for StateParser {
    fn parse(&self, code: &str, _config: &MermaidConfig) -> Result<Ast, Vec<Diagnostic>> {
        let tokens = tokenize(code);
        let mut parser = StateParserImpl::new(&tokens, code);
        parser.parse()
    }

    fn name(&self) -> &'static str {
        "state"
    }
}

struct StateParserImpl<'a> {
    tokens: &'a [PositionedToken],
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> StateParserImpl<'a> {
    fn new(tokens: &'a [PositionedToken], source: &'a str) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
            diagnostics: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<Ast, Vec<Diagnostic>> {
        let start_span = Span::new(0, self.source.len());
        let mut root = AstNode::new(NodeKind::Root, start_span);

        // Parse declaration
        if !self.check(&StateToken::StateDiagram) && !self.check(&StateToken::StateDiagramV2) {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::ParserError,
                "Expected 'stateDiagram' or 'stateDiagram-v2' declaration",
                Span::new(0, 0),
            ));
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let decl_span = self.current_span();
        let decl_text = self.advance().map(|t| t.text.clone()).unwrap_or_default();

        let decl = AstNode::with_text(NodeKind::DiagramDeclaration, decl_span, decl_text);
        root.add_child(decl);

        self.skip_newlines();

        // Parse statements
        while !self.is_at_end() {
            self.skip_newlines();

            if self.is_at_end() {
                break;
            }

            if let Some(stmt) = self.parse_statement() {
                root.add_child(stmt);
            } else {
                self.skip_to_newline();
            }
        }

        if self.diagnostics.iter().any(|d| d.severity.is_error()) {
            Err(std::mem::take(&mut self.diagnostics))
        } else {
            Ok(Ast::new(root, self.source.to_string()))
        }
    }

    fn parse_statement(&mut self) -> Option<AstNode> {
        self.skip_newlines();

        if self.is_at_end() {
            return None;
        }

        // Check for state definition
        if self.check(&StateToken::State) {
            return self.parse_state_definition();
        }

        // Check for note
        if self.check(&StateToken::Note) {
            return self.parse_note();
        }

        // Check for direction
        if self.check(&StateToken::Direction) {
            return self.parse_direction();
        }

        // Try to parse a transition
        self.parse_transition()
    }

    fn parse_state_definition(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'state'

        // Parse state name or quoted description
        let (id, label) = if self.check(&StateToken::DoubleQuotedString) {
            let quoted = self.advance()?.text.clone();
            let label = quoted[1..quoted.len() - 1].to_string();

            // Check for "as" identifier
            if self.check(&StateToken::Identifier) && self.peek()?.text.to_lowercase() == "as" {
                self.advance();
                let id = self.expect_identifier()?;
                (id, Some(label))
            } else {
                (label.clone(), Some(label))
            }
        } else {
            let id = self.expect_identifier()?;
            (id, None)
        };

        // Check for stereotype
        let state_type = if self.check(&StateToken::Fork) {
            self.advance();
            StateType::Fork
        } else if self.check(&StateToken::Join) {
            self.advance();
            StateType::Join
        } else if self.check(&StateToken::Choice) {
            self.advance();
            StateType::Choice
        } else if self.check(&StateToken::Stereotype) {
            let stereotype = self.advance()?.text.clone();
            match stereotype.to_lowercase().as_str() {
                "<<fork>>" => StateType::Fork,
                "<<join>>" => StateType::Join,
                "<<choice>>" => StateType::Choice,
                _ => StateType::Normal,
            }
        } else {
            StateType::Normal
        };

        let mut node = AstNode::with_text(NodeKind::State, Span::new(start, self.previous_span().end), &id);
        node.add_property("id", id.clone());
        node.add_property("state_type", format!("{:?}", state_type));

        if let Some(lbl) = label {
            node.add_property("label", lbl);
        }

        // Check for composite state body
        if self.check(&StateToken::LBrace) {
            self.advance();
            node.add_property("is_composite", "true");

            self.skip_newlines();

            while !self.is_at_end() && !self.check(&StateToken::RBrace) {
                self.skip_newlines();

                if self.check(&StateToken::RBrace) {
                    break;
                }

                if let Some(stmt) = self.parse_statement() {
                    node.add_child(stmt);
                } else {
                    self.skip_to_newline();
                }
            }

            if self.check(&StateToken::RBrace) {
                self.advance();
            }
        }

        // Check for colon (state description)
        if self.check(&StateToken::Colon) {
            self.advance();
            let description = self.parse_text_until_newline();
            node.add_property("description", description);
        }

        node.span = Span::new(start, self.previous_span().end);
        Some(node)
    }

    fn parse_transition(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Parse source state
        let from = self.parse_state_ref()?;

        // Expect arrow
        if !self.check(&StateToken::Arrow) {
            // Not a transition, might be just a state reference
            let mut node = AstNode::with_text(NodeKind::State, Span::new(start, self.previous_span().end), &from);
            node.add_property("id", from);
            return Some(node);
        }

        self.advance(); // consume -->

        // Parse target state
        let to = self.parse_state_ref()?;

        // Check for transition label
        let label = if self.check(&StateToken::Colon) {
            self.advance();
            Some(self.parse_text_until_newline())
        } else {
            None
        };

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Transition, Span::new(start, end));
        node.add_property("from", from);
        node.add_property("to", to);

        if let Some(lbl) = label {
            node.add_property("label", lbl);
        }

        Some(node)
    }

    fn parse_state_ref(&mut self) -> Option<String> {
        if self.check(&StateToken::StartEnd) {
            self.advance();
            return Some("[*]".to_string());
        }

        if self.check(&StateToken::Identifier) {
            return Some(self.advance()?.text.clone());
        }

        if self.check(&StateToken::Text) {
            let text = self.advance()?.text.trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }

        None
    }

    fn parse_note(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'note'

        // Parse position
        let position = if self.check(&StateToken::LeftOf) {
            self.advance();
            "left of"
        } else if self.check(&StateToken::RightOf) {
            self.advance();
            "right of"
        } else {
            ""
        };

        // Parse target state
        let target = self.expect_identifier().unwrap_or_default();

        // Parse note content
        let mut content = String::new();

        if self.check(&StateToken::Colon) {
            // Single line note
            self.advance();
            content = self.parse_text_until_newline();
        } else {
            // Multi-line note (until "end note")
            self.skip_newlines();

            while !self.is_at_end() && !self.check(&StateToken::EndNote) {
                if let Some(token) = self.advance() {
                    if !content.is_empty() && token.kind != StateToken::Newline {
                        content.push(' ');
                    }
                    if token.kind == StateToken::Newline {
                        content.push('\n');
                    } else {
                        content.push_str(&token.text);
                    }
                }
            }

            if self.check(&StateToken::EndNote) {
                self.advance();
            }
        }

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Note, Span::new(start, end));
        node.add_property("position", position.to_string());
        node.add_property("target", target);
        node.add_property("text", content.trim().to_string());

        Some(node)
    }

    fn parse_direction(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'direction'

        let direction = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "direction");
        node.add_property("direction", direction);

        Some(node)
    }

    // Helper methods

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&PositionedToken> {
        self.tokens.get(self.pos)
    }

    fn check(&self, kind: &StateToken) -> bool {
        self.peek().map(|t| &t.kind == kind).unwrap_or(false)
    }

    fn advance(&mut self) -> Option<&PositionedToken> {
        if !self.is_at_end() {
            self.pos += 1;
            self.tokens.get(self.pos - 1)
        } else {
            None
        }
    }

    fn expect_identifier(&mut self) -> Option<String> {
        if self.check(&StateToken::Identifier) {
            Some(self.advance()?.text.clone())
        } else if self.check(&StateToken::DoubleQuotedString) {
            let quoted = self.advance()?.text.clone();
            Some(quoted[1..quoted.len() - 1].to_string())
        } else if self.check(&StateToken::Text) {
            let text = self.advance()?.text.trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
            None
        } else {
            let span = self.current_span();
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::ExpectedToken,
                "Expected identifier",
                span,
            ));
            None
        }
    }

    fn parse_text_until_newline(&mut self) -> String {
        let mut text = String::new();

        while !self.is_at_end() && !self.check(&StateToken::Newline) {
            if let Some(token) = self.advance() {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(&token.text);
            }
        }

        text.trim().to_string()
    }

    fn current_span(&self) -> Span {
        self.peek()
            .map(|t| t.span)
            .unwrap_or_else(|| Span::new(self.source.len(), self.source.len()))
    }

    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&StateToken::Newline) {
            self.advance();
        }
    }

    fn skip_to_newline(&mut self) {
        while !self.is_at_end() && !self.check(&StateToken::Newline) {
            self.advance();
        }
        if self.check(&StateToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(code: &str) -> Result<Ast, Vec<Diagnostic>> {
        StateParser::new().parse(code, &MermaidConfig::default())
    }

    #[test]
    fn test_parse_simple() {
        let code = "stateDiagram-v2\n    [*] --> State1";
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_transitions() {
        let code = r#"stateDiagram-v2
    [*] --> Still
    Still --> [*]
    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_labels() {
        let code = r#"stateDiagram-v2
    [*] --> State1 : Start
    State1 --> State2 : Event1
    State2 --> [*] : Done
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_composite_state() {
        let code = r#"stateDiagram-v2
    state Composite {
        [*] --> Inner1
        Inner1 --> Inner2
        Inner2 --> [*]
    }
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fork_join() {
        let code = r#"stateDiagram-v2
    state fork_state <<fork>>
    [*] --> fork_state
    fork_state --> State1
    fork_state --> State2

    state join_state <<join>>
    State1 --> join_state
    State2 --> join_state
    join_state --> [*]
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_choice() {
        let code = r#"stateDiagram-v2
    state choice_state <<choice>>
    [*] --> choice_state
    choice_state --> State1 : Yes
    choice_state --> State2 : No
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_note() {
        let code = r#"stateDiagram-v2
    [*] --> State1
    note right of State1 : This is a note
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid() {
        let code = "invalid diagram";
        let result = parse(code);
        assert!(result.is_err());
    }
}
