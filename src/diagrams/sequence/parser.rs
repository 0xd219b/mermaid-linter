//! Sequence diagram parser implementation.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::config::MermaidConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::traits::DiagramParser;

use super::lexer::{tokenize, PositionedToken, SeqToken};
use super::ArrowType;

/// Sequence diagram parser.
pub struct SequenceParser;

impl SequenceParser {
    /// Creates a new sequence diagram parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SequenceParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramParser for SequenceParser {
    fn parse(&self, code: &str, _config: &MermaidConfig) -> Result<Ast, Vec<Diagnostic>> {
        let tokens = tokenize(code);
        let mut parser = SequenceParserImpl::new(&tokens, code);
        parser.parse()
    }

    fn name(&self) -> &'static str {
        "sequence"
    }
}

/// Internal parser implementation.
struct SequenceParserImpl<'a> {
    tokens: &'a [PositionedToken],
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> SequenceParserImpl<'a> {
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

        // Skip any leading newlines
        self.skip_newlines();

        // Parse the diagram declaration
        if !self.check(&SeqToken::SequenceDiagram) {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::ParserError,
                "Expected 'sequenceDiagram' declaration",
                Span::new(0, 0),
            ));
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let decl_span = self.current_span();
        self.advance();

        let decl = AstNode::with_text(NodeKind::DiagramDeclaration, decl_span, "sequenceDiagram");
        root.add_child(decl);

        // Skip newlines
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
                // Skip to next line on error
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

        // Check for different statement types
        if self.check(&SeqToken::Participant) {
            return self.parse_participant();
        }

        if self.check(&SeqToken::Actor) {
            return self.parse_actor();
        }

        if self.check(&SeqToken::Note) {
            return self.parse_note();
        }

        if self.check(&SeqToken::Activate) {
            return self.parse_activation(true);
        }

        if self.check(&SeqToken::Deactivate) {
            return self.parse_activation(false);
        }

        if self.check(&SeqToken::Loop) {
            return self.parse_loop();
        }

        if self.check(&SeqToken::Alt) {
            return self.parse_alt();
        }

        if self.check(&SeqToken::Opt) {
            return self.parse_opt();
        }

        if self.check(&SeqToken::Par) {
            return self.parse_par();
        }

        if self.check(&SeqToken::Critical) {
            return self.parse_critical();
        }

        if self.check(&SeqToken::Break) {
            return self.parse_break();
        }

        if self.check(&SeqToken::Rect) {
            return self.parse_rect();
        }

        if self.check(&SeqToken::End) {
            return self.parse_end();
        }

        if self.check(&SeqToken::Else) {
            return self.parse_else();
        }

        if self.check(&SeqToken::Autonumber) {
            return self.parse_autonumber();
        }

        if self.check(&SeqToken::Title) {
            return self.parse_title();
        }

        if self.check(&SeqToken::Box) {
            return self.parse_box();
        }

        if self.check(&SeqToken::Create) {
            return self.parse_create();
        }

        if self.check(&SeqToken::Destroy) {
            return self.parse_destroy();
        }

        // Otherwise, try to parse a message
        self.parse_message()
    }

    fn parse_participant(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'participant'

        // Parse participant ID
        let id = self.expect_identifier()?;

        // Check for alias
        let alias = if self.check(&SeqToken::As) {
            self.advance();
            Some(self.parse_text_until_newline())
        } else {
            None
        };

        let end = self.previous_span().end;
        let mut node = AstNode::with_text(NodeKind::Participant, Span::new(start, end), &id);
        node.add_property("id", id);
        node.add_property("type", "participant");
        if let Some(a) = alias {
            node.add_property("alias", a);
        }

        Some(node)
    }

    fn parse_actor(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'actor'

        let id = self.expect_identifier()?;

        let alias = if self.check(&SeqToken::As) {
            self.advance();
            Some(self.parse_text_until_newline())
        } else {
            None
        };

        let end = self.previous_span().end;
        let mut node = AstNode::with_text(NodeKind::Participant, Span::new(start, end), &id);
        node.add_property("id", id);
        node.add_property("type", "actor");
        if let Some(a) = alias {
            node.add_property("alias", a);
        }

        Some(node)
    }

    fn parse_message(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Parse sender
        let from = self.expect_identifier()?;

        // Parse arrow type
        let arrow_type = self.parse_arrow_type()?;

        // Check for activation marker
        let has_activation = self.check(&SeqToken::Plus);
        let has_deactivation = self.check(&SeqToken::Minus);
        if has_activation || has_deactivation {
            self.advance();
        }

        // Parse receiver
        let to = self.expect_identifier()?;

        // Parse message text (after colon)
        let text = if self.check(&SeqToken::Colon) {
            self.advance();
            self.parse_text_until_newline()
        } else {
            String::new()
        };

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Message, Span::new(start, end));
        node.add_property("from", from);
        node.add_property("to", to);
        node.add_property("arrow_type", format!("{:?}", arrow_type));
        node.add_property("text", text);

        if has_activation {
            node.add_property("activation", "activate");
        } else if has_deactivation {
            node.add_property("activation", "deactivate");
        }

        Some(node)
    }

    fn parse_arrow_type(&mut self) -> Option<ArrowType> {
        let arrow = match self.peek()?.kind {
            SeqToken::SolidArrow => ArrowType::Solid,
            SeqToken::DottedArrow => ArrowType::Dotted,
            SeqToken::SolidLine => ArrowType::SolidLine,
            SeqToken::DottedLine => ArrowType::DottedLine,
            SeqToken::SolidCross | SeqToken::SolidCrossUpper => ArrowType::SolidCross,
            SeqToken::DottedCross | SeqToken::DottedCrossUpper => ArrowType::DottedCross,
            SeqToken::SolidAsync => ArrowType::SolidAsync,
            SeqToken::DottedAsync => ArrowType::DottedAsync,
            _ => return None,
        };

        self.advance();
        Some(arrow)
    }

    fn parse_note(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'Note'

        // Parse position
        let position = if self.check(&SeqToken::LeftOf) {
            self.advance();
            let participant = self.expect_identifier()?;
            format!("left of {}", participant)
        } else if self.check(&SeqToken::RightOf) {
            self.advance();
            let participant = self.expect_identifier()?;
            format!("right of {}", participant)
        } else if self.check(&SeqToken::Over) {
            self.advance();
            let mut participants = vec![self.expect_identifier()?];
            while self.check(&SeqToken::Comma) {
                self.advance();
                participants.push(self.expect_identifier()?);
            }
            format!("over {}", participants.join(","))
        } else {
            String::new()
        };

        // Parse note text
        let text = if self.check(&SeqToken::Colon) {
            self.advance();
            self.parse_text_until_newline()
        } else {
            String::new()
        };

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Note, Span::new(start, end));
        node.add_property("position", position);
        node.add_property("text", text);

        Some(node)
    }

    fn parse_activation(&mut self, is_activate: bool) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'activate' or 'deactivate'

        let participant = self.expect_identifier()?;

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Activation, Span::new(start, end));
        node.add_property("participant", participant);
        node.add_property("action", if is_activate { "activate" } else { "deactivate" });

        Some(node)
    }

    fn parse_loop(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'loop'

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Loop, Span::new(start, end));
        node.add_property("label", label);

        Some(node)
    }

    fn parse_alt(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'alt'

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Alt, Span::new(start, end));
        node.add_property("type", "alt");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_opt(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "opt");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_par(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "par");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_critical(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "critical");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_break(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "break");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_rect(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "rect");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_end(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "end");

        Some(node)
    }

    fn parse_else(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'else'

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "else");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_autonumber(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let rest = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "autonumber");
        node.add_property("value", rest);

        Some(node)
    }

    fn parse_title(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let title = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "title");
        node.add_property("value", title);

        Some(node)
    }

    fn parse_box(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let label = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "box");
        node.add_property("label", label);

        Some(node)
    }

    fn parse_create(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        // Next should be participant
        if self.check(&SeqToken::Participant) || self.check(&SeqToken::Actor) {
            return self.parse_statement();
        }

        let id = self.expect_identifier()?;

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "create");
        node.add_property("participant", id);

        Some(node)
    }

    fn parse_destroy(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let id = self.expect_identifier()?;

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "destroy");
        node.add_property("participant", id);

        Some(node)
    }

    // Helper methods

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&PositionedToken> {
        self.tokens.get(self.pos)
    }

    fn check(&self, kind: &SeqToken) -> bool {
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
        if self.check(&SeqToken::Identifier) {
            Some(self.advance()?.text.clone())
        } else if self.check(&SeqToken::DoubleQuotedString) || self.check(&SeqToken::SingleQuotedString) {
            let quoted = self.advance()?.text.clone();
            Some(quoted[1..quoted.len() - 1].to_string())
        } else {
            // Try text token as identifier
            if self.check(&SeqToken::Text) {
                let text = self.advance()?.text.trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }

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

        while !self.is_at_end() && !self.check(&SeqToken::Newline) {
            if let Some(token) = self.advance() {
                if !text.is_empty() && !matches!(token.kind, SeqToken::Colon | SeqToken::Comma) {
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
        while self.check(&SeqToken::Newline) {
            self.advance();
        }
    }

    fn skip_to_newline(&mut self) {
        while !self.is_at_end() && !self.check(&SeqToken::Newline) {
            self.advance();
        }
        if self.check(&SeqToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(code: &str) -> Result<Ast, Vec<Diagnostic>> {
        SequenceParser::new().parse(code, &MermaidConfig::default())
    }

    #[test]
    fn test_parse_simple() {
        let code = "sequenceDiagram\n    Alice->>Bob: Hello";
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_participants() {
        let code = r#"sequenceDiagram
    participant Alice
    participant Bob
    Alice->>Bob: Hello
    Bob-->>Alice: Hi
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_actors() {
        let code = r#"sequenceDiagram
    actor User
    participant System
    User->>System: Request
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_note() {
        let code = r#"sequenceDiagram
    Alice->>Bob: Hello
    Note right of Bob: Bob thinks
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_loop() {
        let code = r#"sequenceDiagram
    loop Every minute
        Alice->>Bob: Ping
    end
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_alt() {
        let code = r#"sequenceDiagram
    Alice->>Bob: Request
    alt success
        Bob-->>Alice: OK
    else failure
        Bob-->>Alice: Error
    end
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_activation() {
        let code = r#"sequenceDiagram
    Alice->>+Bob: Hello
    Bob-->>-Alice: Hi
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
