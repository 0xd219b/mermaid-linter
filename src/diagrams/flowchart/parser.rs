//! Flowchart parser implementation.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::config::MermaidConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::traits::DiagramParser;

use super::lexer::{tokenize, FlowToken, PositionedToken};
use super::{Direction, LinkType, NodeShape};

/// Flowchart parser.
pub struct FlowchartParser;

impl FlowchartParser {
    /// Creates a new flowchart parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlowchartParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramParser for FlowchartParser {
    fn parse(&self, code: &str, _config: &MermaidConfig) -> Result<Ast, Vec<Diagnostic>> {
        let tokens = tokenize(code);
        let mut parser = FlowchartParserImpl::new(&tokens, code);
        parser.parse()
    }

    fn name(&self) -> &'static str {
        "flowchart"
    }
}

/// Internal parser implementation.
struct FlowchartParserImpl<'a> {
    tokens: &'a [PositionedToken],
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> FlowchartParserImpl<'a> {
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
        if let Some(decl) = self.parse_declaration() {
            root.add_child(decl);
        } else {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::ParserError,
                "Expected 'graph' or 'flowchart' declaration",
                Span::new(0, 0),
            ));
            return Err(std::mem::take(&mut self.diagnostics));
        }

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

    fn parse_declaration(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Check for 'graph' or 'flowchart'
        let is_graph = self.check(&FlowToken::Graph);
        let is_flowchart = self.check(&FlowToken::Flowchart);

        if !is_graph && !is_flowchart {
            return None;
        }

        let keyword = self.advance()?.text.clone();

        // Parse direction
        let direction = if self.check(&FlowToken::DirectionValue) {
            let dir_token = self.advance()?;
            Direction::from_str(&dir_token.text)
        } else {
            Some(Direction::TopToBottom) // Default direction
        };

        let end = self.previous_span().end;
        let mut node = AstNode::with_text(
            NodeKind::DiagramDeclaration,
            Span::new(start, end),
            keyword,
        );

        if let Some(dir) = direction {
            node.add_property("direction", format!("{:?}", dir));
        }

        Some(node)
    }

    fn parse_statement(&mut self) -> Option<AstNode> {
        // Skip leading whitespace and newlines
        self.skip_newlines();

        if self.is_at_end() {
            return None;
        }

        // Check for different statement types
        if self.check(&FlowToken::Subgraph) {
            return self.parse_subgraph();
        }

        if self.check(&FlowToken::End) {
            return self.parse_end();
        }

        if self.check(&FlowToken::Style) {
            return self.parse_style();
        }

        if self.check(&FlowToken::ClassDef) {
            return self.parse_classdef();
        }

        if self.check(&FlowToken::Class) {
            return self.parse_class_assignment();
        }

        if self.check(&FlowToken::Direction) {
            return self.parse_direction();
        }

        if self.check(&FlowToken::Click) {
            return self.parse_click();
        }

        if self.check(&FlowToken::LinkStyle) {
            return self.parse_linkstyle();
        }

        // Otherwise, try to parse a node/link statement
        self.parse_node_or_link()
    }

    fn parse_node_or_link(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Parse the first node
        let first_node = self.parse_node()?;

        // Check if there's a link following
        if self.is_link_start() {
            let mut stmt = AstNode::new(NodeKind::Edge, Span::new(start, start));
            stmt.add_child(first_node);

            // Parse chain of links
            while self.is_link_start() {
                if let Some((link_type, label)) = self.parse_link() {
                    // Parse the target node
                    if let Some(target_node) = self.parse_node() {
                        let mut edge = AstNode::new(NodeKind::Edge, Span::new(start, self.previous_span().end));
                        edge.add_property("link_type", format!("{:?}", link_type));
                        if let Some(lbl) = label {
                            edge.add_property("label", lbl);
                        }
                        edge.add_child(target_node);
                        stmt.add_child(edge);
                    }
                }
            }

            stmt.span = Span::new(start, self.previous_span().end);
            Some(stmt)
        } else {
            // Just a node definition
            Some(first_node)
        }
    }

    fn parse_node(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Parse node ID
        let id = if self.check(&FlowToken::Identifier) || self.check(&FlowToken::Number) {
            self.advance()?.text.clone()
        } else {
            return None;
        };

        // Check for shape/label
        let (shape, label) = self.parse_node_shape_and_label();

        let end = self.previous_span().end;
        let mut node = AstNode::with_text(NodeKind::Node, Span::new(start, end), &id);
        node.add_property("id", id);
        node.add_property("shape", format!("{:?}", shape));

        if let Some(lbl) = label {
            node.add_property("label", lbl);
        }

        Some(node)
    }

    fn parse_node_shape_and_label(&mut self) -> (NodeShape, Option<String>) {
        // Check for different shape delimiters
        if self.check(&FlowToken::LDoubleParen) {
            let start_span = self.current_span();
            self.advance();
            if self.check(&FlowToken::LParen) {
                // ((( ))) - double circle
                self.advance();
                let label = self.parse_label_content();
                if label.is_empty() {
                    self.diagnostics.push(Diagnostic::error(
                        DiagnosticCode::ParserError,
                        "Empty node label is not allowed",
                        start_span,
                    ));
                }
                self.expect(&FlowToken::RParen);
                self.expect(&FlowToken::RDoubleParen);
                return (NodeShape::DoubleCircle, Some(label));
            }
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RDoubleParen);
            return (NodeShape::Circle, Some(label));
        }

        if self.check(&FlowToken::LDoubleBracket) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RDoubleBracket);
            return (NodeShape::Subroutine, Some(label));
        }

        if self.check(&FlowToken::LDoubleBrace) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RDoubleBrace);
            return (NodeShape::Hexagon, Some(label));
        }

        if self.check(&FlowToken::LParenBracket) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RBracketParen);
            return (NodeShape::Stadium, Some(label));
        }

        if self.check(&FlowToken::LBracketParen) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RParenBracket);
            return (NodeShape::Cylindrical, Some(label));
        }

        if self.check(&FlowToken::LBracket) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RBracket);
            return (NodeShape::Rectangle, Some(label));
        }

        if self.check(&FlowToken::LParen) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RParen);
            return (NodeShape::RoundedRect, Some(label));
        }

        if self.check(&FlowToken::LBrace) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RBrace);
            return (NodeShape::Rhombus, Some(label));
        }

        if self.check(&FlowToken::GreaterThan) {
            let start_span = self.current_span();
            self.advance();
            let label = self.parse_label_content();
            if label.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ParserError,
                    "Empty node label is not allowed",
                    start_span,
                ));
            }
            self.expect(&FlowToken::RBracket);
            return (NodeShape::Asymmetric, Some(label));
        }

        (NodeShape::Rectangle, None)
    }

    fn parse_label_content(&mut self) -> String {
        let mut label = String::new();

        while !self.is_at_end() {
            if self.check(&FlowToken::RBracket)
                || self.check(&FlowToken::RParen)
                || self.check(&FlowToken::RBrace)
                || self.check(&FlowToken::RDoubleParen)
                || self.check(&FlowToken::RDoubleBracket)
                || self.check(&FlowToken::RDoubleBrace)
                || self.check(&FlowToken::RBracketParen)
                || self.check(&FlowToken::RParenBracket)
            {
                break;
            }

            if self.check(&FlowToken::DoubleQuotedString) || self.check(&FlowToken::SingleQuotedString) {
                let quoted = self.advance().map(|t| &t.text).unwrap();
                // Remove quotes
                label.push_str(&quoted[1..quoted.len() - 1]);
            } else if let Some(token) = self.advance() {
                label.push_str(&token.text);
            }
        }

        label.trim().to_string()
    }

    fn is_link_start(&self) -> bool {
        self.check(&FlowToken::Arrow)
            || self.check(&FlowToken::Line)
            || self.check(&FlowToken::DottedLine)
            || self.check(&FlowToken::DottedArrow)
            || self.check(&FlowToken::ThickArrow)
            || self.check(&FlowToken::ThickLine)
            || self.check(&FlowToken::Invisible)
            || self.check(&FlowToken::DoubleDash)
            || self.check(&FlowToken::DashDot)
            || self.check(&FlowToken::DoubleEqual)
    }

    fn parse_link(&mut self) -> Option<(LinkType, Option<String>)> {
        let link_type = match self.peek()?.kind {
            FlowToken::Arrow => {
                self.advance();
                LinkType::Arrow
            }
            FlowToken::Line => {
                self.advance();
                LinkType::Open
            }
            FlowToken::DottedLine => {
                self.advance();
                LinkType::Dotted
            }
            FlowToken::DottedArrow => {
                self.advance();
                LinkType::DottedArrow
            }
            FlowToken::ThickArrow => {
                self.advance();
                LinkType::ThickArrow
            }
            FlowToken::ThickLine => {
                self.advance();
                LinkType::Thick
            }
            FlowToken::Invisible => {
                self.advance();
                LinkType::Invisible
            }
            FlowToken::DoubleDash => {
                self.advance();
                // Check for label: -- label -->
                let label = self.parse_edge_label();
                // Expect closing arrow
                if self.check(&FlowToken::Arrow) {
                    self.advance();
                    return Some((LinkType::Arrow, label));
                }
                return Some((LinkType::Open, label));
            }
            _ => return None,
        };

        // Check for pipe-delimited label: -->|label|
        let label = if self.check(&FlowToken::Pipe) {
            self.advance();
            let lbl = self.parse_until_pipe();
            if self.check(&FlowToken::Pipe) {
                self.advance();
            }
            Some(lbl)
        } else {
            None
        };

        Some((link_type, label))
    }

    fn parse_edge_label(&mut self) -> Option<String> {
        let mut label = String::new();

        while !self.is_at_end()
            && !self.check(&FlowToken::Arrow)
            && !self.check(&FlowToken::Line)
            && !self.check(&FlowToken::Newline)
            && !self.check(&FlowToken::Identifier)
        {
            if let Some(token) = self.advance() {
                if !label.is_empty() {
                    label.push(' ');
                }
                label.push_str(&token.text);
            }
        }

        if label.is_empty() {
            None
        } else {
            Some(label.trim().to_string())
        }
    }

    fn parse_until_pipe(&mut self) -> String {
        let mut content = String::new();

        while !self.is_at_end() && !self.check(&FlowToken::Pipe) {
            if let Some(token) = self.advance() {
                content.push_str(&token.text);
            }
        }

        content.trim().to_string()
    }

    fn parse_subgraph(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'subgraph'

        // Parse subgraph ID/title
        let mut id = String::new();
        let mut label = None;

        // Skip whitespace but not newlines
        while self.check(&FlowToken::Text) || self.check(&FlowToken::Identifier) {
            let token = self.advance()?;
            if !id.is_empty() {
                id.push(' ');
            }
            id.push_str(&token.text);
        }

        // Check for bracketed title
        if self.check(&FlowToken::LBracket) {
            self.advance();
            label = Some(self.parse_label_content());
            self.expect(&FlowToken::RBracket);
        }

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Subgraph, Span::new(start, end));
        node.add_property("id", id.trim().to_string());
        if let Some(lbl) = label {
            node.add_property("label", lbl);
        }

        Some(node)
    }

    fn parse_end(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'end'
        let end = self.previous_span().end;

        Some(AstNode::new(NodeKind::Statement, Span::new(start, end)))
    }

    fn parse_style(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'style'

        // Parse node ID
        let id = if self.check(&FlowToken::Identifier) {
            self.advance()?.text.clone()
        } else {
            return None;
        };

        // Parse styles (rest of line)
        let mut styles = Vec::new();
        while !self.is_at_end() && !self.check(&FlowToken::Newline) {
            if let Some(token) = self.advance() {
                styles.push(token.text.clone());
            }
        }

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Style, Span::new(start, end));
        node.add_property("node_id", id);
        node.add_property("styles", styles.join(" "));

        Some(node)
    }

    fn parse_classdef(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'classDef'

        // Parse class name
        let name = if self.check(&FlowToken::Identifier) {
            self.advance()?.text.clone()
        } else {
            return None;
        };

        // Parse styles (rest of line)
        let mut styles = Vec::new();
        while !self.is_at_end() && !self.check(&FlowToken::Newline) {
            if let Some(token) = self.advance() {
                styles.push(token.text.clone());
            }
        }

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::ClassDef, Span::new(start, end));
        node.add_property("name", name);
        node.add_property("styles", styles.join(" "));

        Some(node)
    }

    fn parse_class_assignment(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'class'

        // Parse node IDs
        let mut node_ids = Vec::new();
        while self.check(&FlowToken::Identifier) {
            node_ids.push(self.advance()?.text.clone());
            if self.check(&FlowToken::Comma) {
                self.advance();
            }
        }

        // Parse class name
        let class_name = if self.check(&FlowToken::Identifier) {
            self.advance()?.text.clone()
        } else {
            String::new()
        };

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "class_assignment");
        node.add_property("node_ids", node_ids.join(","));
        node.add_property("class_name", class_name);

        Some(node)
    }

    fn parse_direction(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'direction'

        let direction = if self.check(&FlowToken::DirectionValue) {
            self.advance()?.text.clone()
        } else {
            String::new()
        };

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "direction");
        node.add_property("direction", direction);

        Some(node)
    }

    fn parse_click(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'click'

        // Parse node ID
        let node_id = if self.check(&FlowToken::Identifier) {
            self.advance()?.text.clone()
        } else {
            return None;
        };

        // Parse rest of click definition (URL, callback, tooltip)
        let mut rest = Vec::new();
        while !self.is_at_end() && !self.check(&FlowToken::Newline) {
            if let Some(token) = self.advance() {
                rest.push(token.text.clone());
            }
        }

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "click");
        node.add_property("node_id", node_id);
        node.add_property("definition", rest.join(" "));

        Some(node)
    }

    fn parse_linkstyle(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'linkStyle'

        // Parse link index(es) or 'default'
        let mut indices = Vec::new();
        while self.check(&FlowToken::Number) || self.check(&FlowToken::Identifier) {
            indices.push(self.advance()?.text.clone());
            if self.check(&FlowToken::Comma) {
                self.advance();
            }
        }

        // Parse styles
        let mut styles = Vec::new();
        while !self.is_at_end() && !self.check(&FlowToken::Newline) {
            if let Some(token) = self.advance() {
                styles.push(token.text.clone());
            }
        }

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "linkStyle");
        node.add_property("indices", indices.join(","));
        node.add_property("styles", styles.join(" "));

        Some(node)
    }

    // Helper methods

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&PositionedToken> {
        self.tokens.get(self.pos)
    }

    fn check(&self, kind: &FlowToken) -> bool {
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

    fn expect(&mut self, kind: &FlowToken) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            let span = self.current_span();
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::ExpectedToken,
                format!("Expected {:?}", kind),
                span,
            ));
            false
        }
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
        // Skip newlines and semicolons (both work as statement separators in Mermaid)
        while self.check(&FlowToken::Newline) || self.check(&FlowToken::Semicolon) {
            self.advance();
        }
    }

    fn skip_to_newline(&mut self) {
        while !self.is_at_end() && !self.check(&FlowToken::Newline) {
            self.advance();
        }
        if self.check(&FlowToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(code: &str) -> Result<Ast, Vec<Diagnostic>> {
        FlowchartParser::new().parse(code, &MermaidConfig::default())
    }

    #[test]
    fn test_parse_simple_flowchart() {
        let code = "graph TD\n    A --> B";
        let result = parse(code);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.root.kind, NodeKind::Root);
    }

    #[test]
    fn test_parse_flowchart_with_labels() {
        let code = r#"graph LR
    A[Start] --> B{Decision}
    B -->|Yes| C[End]
    B -->|No| D[Retry]
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_flowchart_v2() {
        let code = "flowchart TD\n    A --> B --> C";
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_subgraph() {
        let code = r#"graph TD
    subgraph one
        a1 --> a2
    end
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_style() {
        let code = r#"graph TD
    A --> B
    style A fill:#f9f
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_classdef() {
        let code = r#"graph TD
    A --> B
    classDef default fill:#f9f
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_different_shapes() {
        let shapes = vec![
            "A[Rectangle]",
            "B(Rounded)",
            "C{Rhombus}",
            "D((Circle))",
            "E[[Subroutine]]",
            "F[(Cylinder)]",
            "G([Stadium])",
            "H{{Hexagon}}",
        ];

        for shape in shapes {
            let code = format!("graph TD\n    {}", shape);
            let result = parse(&code);
            assert!(result.is_ok(), "Failed to parse shape: {}", shape);
        }
    }

    #[test]
    fn test_parse_invalid() {
        let code = "invalid diagram";
        let result = parse(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_paren_label() {
        // Empty parentheses should fail - this is invalid in Mermaid
        let code = "graph TD; A-->B()";
        let result = parse(code);
        assert!(result.is_err(), "Expected error for empty parentheses");
    }

    #[test]
    fn test_parse_empty_bracket_label() {
        // Empty brackets should also fail
        let code = "graph TD; A-->B[]";
        let result = parse(code);
        assert!(result.is_err(), "Expected error for empty brackets");
    }

    #[test]
    fn test_parse_empty_brace_label() {
        // Empty braces should also fail
        let code = "graph TD; A-->B{}";
        let result = parse(code);
        assert!(result.is_err(), "Expected error for empty braces");
    }
}
