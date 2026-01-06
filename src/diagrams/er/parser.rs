//! Parser for ER diagrams.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};

use super::lexer::{tokenize, ErToken, Token};
use super::{Cardinality, IdentificationType};

/// Parser for ER diagrams.
pub struct ErParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> ErParser<'a> {
    /// Create a new parser.
    pub fn new(source: &'a str) -> Self {
        Self {
            tokens: tokenize(source),
            pos: 0,
            source,
            diagnostics: Vec::new(),
        }
    }

    /// Parse the ER diagram.
    pub fn parse(&mut self) -> Result<Ast, Vec<Diagnostic>> {
        let start_span = Span::new(0, self.source.len());
        let mut root = AstNode::new(NodeKind::Root, start_span);

        // Skip any leading newlines
        self.skip_newlines();

        // Parse the diagram declaration
        if let Some(decl) = self.parse_declaration() {
            root.add_child(decl);
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ExpectedToken,
                "Expected 'erDiagram'".to_string(),
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

    /// Parse the erDiagram declaration.
    fn parse_declaration(&mut self) -> Option<AstNode> {
        if !self.check(&ErToken::ErDiagram) {
            return None;
        }

        let start = self.current_span().start;
        self.advance(); // consume 'erDiagram'
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::DiagramDeclaration, Span::new(start, end));
        node.text = Some("erDiagram".to_string());

        Some(node)
    }

    /// Parse a statement.
    fn parse_statement(&mut self) -> Option<AstNode> {
        // Skip semicolons
        while self.check(&ErToken::Semicolon) {
            self.advance();
        }

        self.skip_newlines();

        if self.is_at_end() {
            return None;
        }

        // Check for direction
        if self.check(&ErToken::Direction) {
            return self.parse_direction();
        }

        // Check for style/classDef/class
        if self.check(&ErToken::Style) {
            return self.parse_style();
        }
        if self.check(&ErToken::ClassDef) {
            return self.parse_classdef();
        }
        if self.check(&ErToken::Class) {
            return self.parse_class_assignment();
        }

        // Check for accessibility
        if self.check(&ErToken::AccTitle) || self.check(&ErToken::AccDescr) {
            return self.parse_accessibility();
        }

        // Parse entity or relationship
        if self.check(&ErToken::Identifier) || self.check(&ErToken::QuotedString) {
            return self.parse_entity_or_relationship();
        }

        None
    }

    /// Parse direction statement.
    fn parse_direction(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'direction'

        if self.check(&ErToken::DirectionValue) || self.check(&ErToken::Identifier) {
            let dir = self.current_text();
            self.advance();
            let end = self.previous_span().end;

            let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
            node.add_property("type", "direction");
            node.add_property("value", dir);
            return Some(node);
        }

        None
    }

    /// Parse style statement.
    fn parse_style(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'style'

        // Parse entity names
        let mut entities = Vec::new();
        while self.check(&ErToken::Identifier) {
            entities.push(self.current_text());
            self.advance();
            if self.check(&ErToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        // Parse style properties
        let style_text = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "style");
        node.add_property("entities", entities.join(","));
        node.add_property("styles", style_text);
        Some(node)
    }

    /// Parse classDef statement.
    fn parse_classdef(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'classDef'

        // Parse class names
        let mut class_names = Vec::new();
        while self.check(&ErToken::Identifier) {
            class_names.push(self.current_text());
            self.advance();
            if self.check(&ErToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        // Parse style properties
        let style_text = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "classDef");
        node.add_property("classes", class_names.join(","));
        node.add_property("styles", style_text);
        Some(node)
    }

    /// Parse class assignment.
    fn parse_class_assignment(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'class'

        // Parse entity names
        let mut entities = Vec::new();
        while self.check(&ErToken::Identifier) {
            entities.push(self.current_text());
            self.advance();
            if self.check(&ErToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        // Parse class names
        let mut class_names = Vec::new();
        while self.check(&ErToken::Identifier) {
            class_names.push(self.current_text());
            self.advance();
            if self.check(&ErToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "class");
        node.add_property("entities", entities.join(","));
        node.add_property("classes", class_names.join(","));
        Some(node)
    }

    /// Parse accessibility statement.
    fn parse_accessibility(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        let acc_type = if self.check(&ErToken::AccTitle) {
            "accTitle"
        } else {
            "accDescr"
        };
        self.advance();

        // Skip colon if present
        if self.check(&ErToken::Colon) {
            self.advance();
        }

        // Check for multi-line description
        if self.check(&ErToken::OpenBrace) {
            self.advance();
            let mut content = String::new();
            while !self.check(&ErToken::CloseBrace) && !self.is_at_end() {
                content.push_str(&self.current_text());
                content.push(' ');
                self.advance();
            }
            if self.check(&ErToken::CloseBrace) {
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
        node.add_property("value", value);
        Some(node)
    }

    /// Parse entity definition or relationship.
    fn parse_entity_or_relationship(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Get first entity name
        let entity_a = self.parse_entity_name()?;

        // Check for ::: class assignment
        let class_a = if self.check(&ErToken::TripleColon) {
            self.advance();
            if self.check(&ErToken::Identifier) {
                let class = self.current_text();
                self.advance();
                Some(class)
            } else {
                None
            }
        } else {
            None
        };

        // Check for attributes block
        if self.check(&ErToken::OpenBrace) {
            return self.parse_entity_with_attributes(start, entity_a, class_a);
        }

        // Check for relationship
        if self.current_is_cardinality() {
            return self.parse_relationship(start, entity_a, class_a);
        }

        // Just an entity declaration
        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Other("Entity".to_string()), Span::new(start, end));
        node.text = Some(entity_a.clone());
        node.add_property("name", entity_a);
        if let Some(class) = class_a {
            node.add_property("class", class);
        }
        Some(node)
    }

    /// Parse entity name (identifier or quoted string).
    fn parse_entity_name(&mut self) -> Option<String> {
        if self.check(&ErToken::Identifier) {
            let name = self.current_text();
            self.advance();
            Some(name)
        } else if self.check(&ErToken::QuotedString) {
            let text = self.current_text();
            self.advance();
            // Remove quotes
            Some(text[1..text.len() - 1].to_string())
        } else {
            None
        }
    }

    /// Parse entity with attributes.
    fn parse_entity_with_attributes(
        &mut self,
        start: usize,
        name: String,
        class: Option<String>,
    ) -> Option<AstNode> {
        self.advance(); // consume '{'

        let mut entity = AstNode::new(NodeKind::Other("Entity".to_string()), Span::new(start, start));
        entity.text = Some(name.clone());
        entity.add_property("name", name);
        if let Some(c) = class {
            entity.add_property("class", c);
        }

        // Parse attributes
        while !self.check(&ErToken::CloseBrace) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&ErToken::CloseBrace) {
                break;
            }

            if let Some(attr) = self.parse_attribute() {
                entity.add_child(attr);
            } else {
                self.advance(); // Skip unknown token
            }
        }

        if self.check(&ErToken::CloseBrace) {
            self.advance();
        }

        let end = self.previous_span().end;
        entity.span = Span::new(start, end);
        Some(entity)
    }

    /// Parse an attribute.
    fn parse_attribute(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Get attribute type
        let attr_type = if self.check(&ErToken::Identifier) {
            let t = self.current_text();
            self.advance();
            t
        } else {
            return None;
        };

        // Check for generic type
        let attr_type = if self.check(&ErToken::GenericType) || self.check(&ErToken::Tilde) {
            let mut full_type = attr_type;
            if self.check(&ErToken::Tilde) {
                self.advance();
                full_type.push('~');
                while !self.check(&ErToken::Tilde)
                    && !self.check(&ErToken::Newline)
                    && !self.is_at_end()
                {
                    full_type.push_str(&self.current_text());
                    self.advance();
                }
                if self.check(&ErToken::Tilde) {
                    full_type.push('~');
                    self.advance();
                }
            } else {
                full_type.push_str(&self.current_text());
                self.advance();
            }
            full_type
        } else {
            attr_type
        };

        // Get attribute name
        let attr_name = if self.check(&ErToken::Identifier) {
            let n = self.current_text();
            self.advance();
            n
        } else {
            return None;
        };

        let mut attr = AstNode::new(NodeKind::Attribute, Span::new(start, start));
        attr.add_property("type", attr_type);
        attr.add_property("name", attr_name);

        // Parse keys (PK, FK, UK)
        let mut keys = Vec::new();
        loop {
            if self.check(&ErToken::PrimaryKey) {
                keys.push("PK");
                self.advance();
            } else if self.check(&ErToken::ForeignKey) {
                keys.push("FK");
                self.advance();
            } else if self.check(&ErToken::UniqueKey) {
                keys.push("UK");
                self.advance();
            } else if self.check(&ErToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        if !keys.is_empty() {
            attr.add_property("keys", keys.join(","));
        }

        // Parse comment
        if self.check(&ErToken::QuotedString) {
            let comment = self.current_text();
            self.advance();
            attr.add_property("comment", comment[1..comment.len() - 1].to_string());
        }

        let end = self.previous_span().end;
        attr.span = Span::new(start, end);
        Some(attr)
    }

    /// Parse a relationship.
    fn parse_relationship(
        &mut self,
        start: usize,
        entity_a: String,
        class_a: Option<String>,
    ) -> Option<AstNode> {
        // Parse left cardinality
        let card_a = self.parse_cardinality()?;

        // Parse identification type
        let id_type = if self.check(&ErToken::Identifying) {
            self.advance();
            IdentificationType::Identifying
        } else if self.check(&ErToken::NonIdentifying) {
            self.advance();
            IdentificationType::NonIdentifying
        } else {
            IdentificationType::Identifying
        };

        // Parse right cardinality
        let card_b = self.parse_cardinality()?;

        // Parse second entity
        let entity_b = self.parse_entity_name()?;

        // Check for ::: class assignment on second entity
        let class_b = if self.check(&ErToken::TripleColon) {
            self.advance();
            if self.check(&ErToken::Identifier) {
                let class = self.current_text();
                self.advance();
                Some(class)
            } else {
                None
            }
        } else {
            None
        };

        // Parse label (after colon)
        let label = if self.check(&ErToken::Colon) {
            self.advance();
            Some(self.consume_until_newline())
        } else {
            None
        };

        let end = self.previous_span().end;

        let mut rel = AstNode::new(NodeKind::Relationship, Span::new(start, end));
        rel.add_property("entityA", entity_a);
        rel.add_property("cardinalityA", card_a.as_str().to_string());
        rel.add_property("identification", id_type.as_str().to_string());
        rel.add_property("cardinalityB", card_b.as_str().to_string());
        rel.add_property("entityB", entity_b);

        if let Some(c) = class_a {
            rel.add_property("classA", c);
        }
        if let Some(c) = class_b {
            rel.add_property("classB", c);
        }
        if let Some(l) = label {
            rel.add_property("label", l.trim().to_string());
        }

        Some(rel)
    }

    /// Parse cardinality.
    fn parse_cardinality(&mut self) -> Option<Cardinality> {
        if self.check(&ErToken::OnlyOneLeft) {
            self.advance();
            Some(Cardinality::OnlyOne)
        } else if self.check(&ErToken::ZeroOrOneLeft) || self.check(&ErToken::ZeroOrOneRight) {
            self.advance();
            Some(Cardinality::ZeroOrOne)
        } else if self.check(&ErToken::OneOrMoreLeft) || self.check(&ErToken::OneOrMoreRight) {
            self.advance();
            Some(Cardinality::OneOrMore)
        } else if self.check(&ErToken::ZeroOrMoreLeft) || self.check(&ErToken::ZeroOrMoreRight) {
            self.advance();
            Some(Cardinality::ZeroOrMore)
        } else {
            None
        }
    }

    /// Check if current token is a cardinality marker.
    fn current_is_cardinality(&self) -> bool {
        if let Some(token) = self.current() {
            token.kind.is_cardinality()
        } else {
            false
        }
    }

    /// Consume tokens until newline.
    fn consume_until_newline(&mut self) -> String {
        let mut text = String::new();
        while !self.check(&ErToken::Newline) && !self.is_at_end() {
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

    fn check(&self, kind: &ErToken) -> bool {
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
        while self.check(&ErToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let code = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places"#;

        let mut parser = ErParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_attributes() {
        let code = r#"erDiagram
    CUSTOMER {
        string name
        string custNumber PK
    }"#;

        let mut parser = ErParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_multiple_relationships() {
        let code = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places
    ORDER ||--|{ LINE-ITEM : contains
    CUSTOMER }|..|{ DELIVERY-ADDRESS : uses"#;

        let mut parser = ErParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_full_example() {
        let code = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places
    CUSTOMER {
        string name
        string custNumber PK
        string sector
    }
    ORDER ||--|{ LINE-ITEM : contains
    ORDER {
        int orderNumber PK
        string deliveryAddress
    }
    LINE-ITEM {
        string productCode PK, FK
        int quantity
        float pricePerUnit
    }"#;

        let mut parser = ErParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_direction() {
        let code = r#"erDiagram
    direction LR
    CUSTOMER ||--o{ ORDER : places"#;

        let mut parser = ErParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_invalid() {
        let code = "not an er diagram";
        let mut parser = ErParser::new(code);
        let result = parser.parse();
        assert!(result.is_err());
    }
}
