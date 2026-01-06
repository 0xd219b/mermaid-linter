//! Class diagram parser implementation.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::config::MermaidConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::traits::DiagramParser;

use super::lexer::{tokenize, ClassToken, PositionedToken};
use super::{RelationType, Visibility};

/// Class diagram parser.
pub struct ClassParser;

impl ClassParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClassParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramParser for ClassParser {
    fn parse(&self, code: &str, _config: &MermaidConfig) -> Result<Ast, Vec<Diagnostic>> {
        let tokens = tokenize(code);
        let mut parser = ClassParserImpl::new(&tokens, code);
        parser.parse()
    }

    fn name(&self) -> &'static str {
        "class"
    }
}

struct ClassParserImpl<'a> {
    tokens: &'a [PositionedToken],
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> ClassParserImpl<'a> {
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
        if !self.check(&ClassToken::ClassDiagram) && !self.check(&ClassToken::ClassDiagramV2) {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::ParserError,
                "Expected 'classDiagram' declaration",
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

        if self.check(&ClassToken::Class) {
            return self.parse_class();
        }

        if self.check(&ClassToken::Namespace) {
            return self.parse_namespace();
        }

        if self.check(&ClassToken::Note) {
            return self.parse_note();
        }

        if self.check(&ClassToken::Direction) {
            return self.parse_direction();
        }

        if self.check(&ClassToken::Click) {
            return self.parse_click();
        }

        if self.check(&ClassToken::Link) || self.check(&ClassToken::Callback) {
            return self.parse_link_or_callback();
        }

        if self.check(&ClassToken::CssClass) {
            return self.parse_css_class();
        }

        // Try to parse a relationship or class member
        self.parse_relationship_or_member()
    }

    fn parse_class(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'class'

        let name = self.expect_identifier()?;

        // Check for stereotype
        let stereotype = if self.check(&ClassToken::Stereotype) {
            let s = self.advance()?.text.clone();
            Some(s.trim_start_matches("<<").trim_end_matches(">>").to_string())
        } else {
            None
        };

        // Check for generic parameter
        let generic = if self.check(&ClassToken::Text) && self.peek()?.text.starts_with('~') {
            let text = self.advance()?.text.clone();
            Some(text.trim_start_matches('~').trim_end_matches('~').to_string())
        } else {
            None
        };

        let mut node = AstNode::with_text(NodeKind::Class, Span::new(start, self.previous_span().end), &name);
        node.add_property("name", name);

        if let Some(st) = stereotype {
            node.add_property("stereotype", st);
        }
        if let Some(g) = generic {
            node.add_property("generic", g);
        }

        // Check for class body
        if self.check(&ClassToken::LBrace) {
            self.advance();
            self.skip_newlines();

            while !self.is_at_end() && !self.check(&ClassToken::RBrace) {
                self.skip_newlines();

                if self.check(&ClassToken::RBrace) {
                    break;
                }

                if let Some(member) = self.parse_class_member() {
                    node.add_child(member);
                } else {
                    self.skip_to_newline();
                }
            }

            if self.check(&ClassToken::RBrace) {
                self.advance();
            }
        }

        node.span = Span::new(start, self.previous_span().end);
        Some(node)
    }

    fn parse_class_member(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Check for stereotype
        if self.check(&ClassToken::Stereotype) {
            let stereotype = self.advance()?.text.clone();
            let mut node = AstNode::new(NodeKind::Statement, Span::new(start, self.previous_span().end));
            node.add_property("type", "stereotype");
            node.add_property("value", stereotype);
            return Some(node);
        }

        // Check for visibility
        let visibility = self.parse_visibility();

        // Check for static marker ($)
        let is_static = self.check(&ClassToken::Dollar);
        if is_static {
            self.advance();
        }

        // Parse type and name
        let first_part = if self.check(&ClassToken::Identifier) {
            self.advance()?.text.clone()
        } else if self.check(&ClassToken::Text) {
            self.advance()?.text.trim().to_string()
        } else {
            return None;
        };

        // Check if this is a method (has parentheses)
        let (name, member_type, is_method, params, return_type) = if self.check(&ClassToken::LParen) {
            // Method without explicit type
            self.advance();
            let params = self.parse_until(&ClassToken::RParen);
            if self.check(&ClassToken::RParen) {
                self.advance();
            }

            let ret_type = if self.check(&ClassToken::Colon) || self.check(&ClassToken::Identifier) {
                if self.check(&ClassToken::Colon) {
                    self.advance();
                }
                if self.check(&ClassToken::Identifier) {
                    Some(self.advance()?.text.clone())
                } else {
                    None
                }
            } else {
                None
            };

            (first_part, None::<String>, true, Some(params), ret_type)
        } else if self.check(&ClassToken::Identifier) {
            // We have type and name
            let name = self.advance()?.text.clone();

            if self.check(&ClassToken::LParen) {
                // Method with type
                self.advance();
                let params = self.parse_until(&ClassToken::RParen);
                if self.check(&ClassToken::RParen) {
                    self.advance();
                }
                (name, Some(first_part), true, Some(params), None)
            } else {
                // Attribute with type
                (name, Some(first_part), false, None, None)
            }
        } else {
            // Just a name (attribute without type)
            (first_part, None, false, None, None)
        };

        // Check for abstract marker (*)
        let is_abstract = self.check(&ClassToken::Star);
        if is_abstract {
            self.advance();
        }

        let end = self.previous_span().end;
        let kind = if is_method { NodeKind::Method } else { NodeKind::Attribute };

        let mut node = AstNode::with_text(kind, Span::new(start, end), &name);
        node.add_property("name", name);

        if let Some(v) = visibility {
            node.add_property("visibility", format!("{:?}", v));
        }
        if let Some(t) = member_type {
            node.add_property("type", t);
        }
        if is_static {
            node.add_property("static", "true");
        }
        if is_abstract {
            node.add_property("abstract", "true");
        }
        if let Some(p) = params {
            node.add_property("parameters", p);
        }
        if let Some(r) = return_type {
            node.add_property("return_type", r);
        }

        Some(node)
    }

    fn parse_visibility(&mut self) -> Option<Visibility> {
        let vis = match self.peek()?.kind {
            ClassToken::Public => Some(Visibility::Public),
            ClassToken::Private => Some(Visibility::Private),
            ClassToken::Protected => Some(Visibility::Protected),
            ClassToken::Package => Some(Visibility::Package),
            _ => None,
        };

        if vis.is_some() {
            self.advance();
        }

        vis
    }

    fn parse_relationship_or_member(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Parse first identifier
        let first_id = self.expect_identifier()?;

        // Check for relationship
        if let Some(rel_type) = self.try_parse_relation_type() {
            // This is a relationship
            let second_id = self.expect_identifier()?;

            // Check for label
            let label = if self.check(&ClassToken::Colon) {
                self.advance();
                Some(self.parse_text_until_newline())
            } else {
                None
            };

            let end = self.previous_span().end;
            let mut node = AstNode::new(NodeKind::Relationship, Span::new(start, end));
            node.add_property("from", first_id);
            node.add_property("to", second_id);
            node.add_property("relation_type", format!("{:?}", rel_type));

            if let Some(l) = label {
                node.add_property("label", l);
            }

            return Some(node);
        }

        // Check for member definition on class (ClassName : member)
        if self.check(&ClassToken::Colon) {
            self.advance();

            // Parse visibility
            let visibility = self.parse_visibility();

            // Parse rest of member definition
            let member_def = self.parse_text_until_newline();

            let end = self.previous_span().end;
            let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
            node.add_property("type", "class_member");
            node.add_property("class", first_id);
            node.add_property("member", member_def);

            if let Some(v) = visibility {
                node.add_property("visibility", format!("{:?}", v));
            }

            return Some(node);
        }

        None
    }

    fn try_parse_relation_type(&mut self) -> Option<RelationType> {
        let rel = match self.peek()?.kind {
            ClassToken::InheritanceLeft | ClassToken::InheritanceRight => Some(RelationType::Inheritance),
            ClassToken::CompositionLeft | ClassToken::CompositionRight => Some(RelationType::Composition),
            ClassToken::AggregationLeft | ClassToken::AggregationRight => Some(RelationType::Aggregation),
            ClassToken::DependencyLeft | ClassToken::DependencyRight => Some(RelationType::Dependency),
            ClassToken::RealizationLeft | ClassToken::RealizationRight => Some(RelationType::Realization),
            ClassToken::Association => Some(RelationType::Association),
            ClassToken::DashedLine => Some(RelationType::DashedLink),
            _ => None,
        };

        if rel.is_some() {
            self.advance();
        }

        rel
    }

    fn parse_namespace(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'namespace'

        let name = self.expect_identifier()?;

        let mut node = AstNode::new(NodeKind::Subgraph, Span::new(start, self.previous_span().end));
        node.add_property("type", "namespace");
        node.add_property("name", name);

        // Parse body if present
        if self.check(&ClassToken::LBrace) {
            self.advance();
            self.skip_newlines();

            while !self.is_at_end() && !self.check(&ClassToken::RBrace) {
                self.skip_newlines();

                if let Some(stmt) = self.parse_statement() {
                    node.add_child(stmt);
                } else {
                    self.skip_to_newline();
                }
            }

            if self.check(&ClassToken::RBrace) {
                self.advance();
            }
        }

        node.span = Span::new(start, self.previous_span().end);
        Some(node)
    }

    fn parse_note(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'note'

        // Parse position and target
        let mut position = String::new();
        let mut target = String::new();

        while !self.is_at_end() && !self.check(&ClassToken::Colon) && !self.check(&ClassToken::Newline) {
            if self.check(&ClassToken::For) {
                self.advance();
                target = self.expect_identifier().unwrap_or_default();
            } else if let Some(token) = self.advance() {
                if !position.is_empty() {
                    position.push(' ');
                }
                position.push_str(&token.text);
            }
        }

        let text = if self.check(&ClassToken::Colon) {
            self.advance();
            self.parse_text_until_newline()
        } else {
            String::new()
        };

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Note, Span::new(start, end));
        node.add_property("position", position.trim().to_string());
        node.add_property("target", target);
        node.add_property("text", text);

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

    fn parse_click(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let target = self.expect_identifier()?;
        let rest = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "click");
        node.add_property("target", target);
        node.add_property("definition", rest);

        Some(node)
    }

    fn parse_link_or_callback(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        let keyword = self.advance()?.text.clone();

        let target = self.expect_identifier()?;
        let rest = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", keyword.to_lowercase());
        node.add_property("target", target);
        node.add_property("definition", rest);

        Some(node)
    }

    fn parse_css_class(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance();

        let rest = self.parse_text_until_newline();

        let end = self.previous_span().end;
        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "cssClass");
        node.add_property("definition", rest);

        Some(node)
    }

    fn parse_until(&mut self, end_token: &ClassToken) -> String {
        let mut content = String::new();

        while !self.is_at_end() && !self.check(end_token) && !self.check(&ClassToken::Newline) {
            if let Some(token) = self.advance() {
                content.push_str(&token.text);
            }
        }

        content.trim().to_string()
    }

    // Helper methods

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&PositionedToken> {
        self.tokens.get(self.pos)
    }

    fn check(&self, kind: &ClassToken) -> bool {
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
        if self.check(&ClassToken::Identifier) {
            Some(self.advance()?.text.clone())
        } else if self.check(&ClassToken::DoubleQuotedString) {
            let quoted = self.advance()?.text.clone();
            Some(quoted[1..quoted.len() - 1].to_string())
        } else if self.check(&ClassToken::Text) {
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

        while !self.is_at_end() && !self.check(&ClassToken::Newline) {
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
        while self.check(&ClassToken::Newline) {
            self.advance();
        }
    }

    fn skip_to_newline(&mut self) {
        while !self.is_at_end() && !self.check(&ClassToken::Newline) {
            self.advance();
        }
        if self.check(&ClassToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(code: &str) -> Result<Ast, Vec<Diagnostic>> {
        ClassParser::new().parse(code, &MermaidConfig::default())
    }

    #[test]
    fn test_parse_simple() {
        let code = "classDiagram\n    class Animal";
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_inheritance() {
        let code = r#"classDiagram
    Animal <|-- Dog
    Animal <|-- Cat
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_class_with_members() {
        let code = r#"classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_stereotype() {
        let code = r#"classDiagram
    class Animal {
        <<interface>>
        +makeSound()
    }
"#;
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_relationships() {
        let code = r#"classDiagram
    classA <|-- classB
    classC *-- classD
    classE o-- classF
    classG ..> classH
    classI -- classJ
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
