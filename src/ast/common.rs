//! Common AST types used across all diagram types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A span in the source code (byte offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset.
    pub start: usize,
    /// End byte offset.
    pub end: usize,
}

impl Span {
    /// Creates a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Creates a span from a start position and length.
    pub fn from_len(start: usize, len: usize) -> Self {
        Self {
            start,
            end: start + len,
        }
    }

    /// Creates an empty span at the given position.
    pub fn empty(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Returns the length of the span.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the span is empty.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Merges this span with another, creating a span that encompasses both.
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Returns the text this span covers in the given source.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end.min(source.len())]
    }
}

/// Kind of AST node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// Root node of a diagram.
    Root,
    /// A diagram declaration (e.g., "graph TD", "sequenceDiagram").
    DiagramDeclaration,
    /// A node/vertex in a graph.
    Node,
    /// An edge/link between nodes.
    Edge,
    /// A subgraph/container.
    Subgraph,
    /// A style definition.
    Style,
    /// A class definition.
    ClassDef,
    /// A directive.
    Directive,
    /// A comment.
    Comment,
    /// A label/text.
    Label,
    /// An identifier.
    Identifier,
    /// A message (in sequence diagrams).
    Message,
    /// A participant (in sequence diagrams).
    Participant,
    /// An activation (in sequence diagrams).
    Activation,
    /// A note (in sequence diagrams).
    Note,
    /// A loop block.
    Loop,
    /// An alt/else block.
    Alt,
    /// A state (in state diagrams).
    State,
    /// A transition (in state diagrams).
    Transition,
    /// A class (in class diagrams).
    Class,
    /// A method (in class diagrams).
    Method,
    /// An attribute (in class diagrams).
    Attribute,
    /// A relationship (in class diagrams).
    Relationship,
    /// Generic statement.
    Statement,
    /// Unknown/other node type.
    Other(String),
}

impl NodeKind {
    /// Returns true if this is a container node that can have children.
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            NodeKind::Root
                | NodeKind::Subgraph
                | NodeKind::Loop
                | NodeKind::Alt
                | NodeKind::State
                | NodeKind::Class
        )
    }
}

/// A node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    /// The kind of node.
    pub kind: NodeKind,
    /// The span in the source code.
    pub span: Span,
    /// The raw text of this node (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Child nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<AstNode>,
    /// Named fields (for structured data).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub fields: HashMap<String, AstNode>,
    /// Additional properties.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
}

impl AstNode {
    /// Creates a new AST node.
    pub fn new(kind: NodeKind, span: Span) -> Self {
        Self {
            kind,
            span,
            text: None,
            children: Vec::new(),
            fields: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    /// Creates a new AST node with text.
    pub fn with_text(kind: NodeKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: Some(text.into()),
            children: Vec::new(),
            fields: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    /// Adds a child node.
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    /// Adds a named field.
    pub fn add_field(&mut self, name: impl Into<String>, node: AstNode) {
        self.fields.insert(name.into(), node);
    }

    /// Adds a property.
    pub fn add_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(name.into(), value.into());
    }

    /// Returns children of a specific kind.
    pub fn children_of_kind(&self, kind: &NodeKind) -> Vec<&AstNode> {
        self.children.iter().filter(|c| &c.kind == kind).collect()
    }

    /// Finds the first child of a specific kind.
    pub fn find_child(&self, kind: &NodeKind) -> Option<&AstNode> {
        self.children.iter().find(|c| &c.kind == kind)
    }

    /// Gets a field by name.
    pub fn get_field(&self, name: &str) -> Option<&AstNode> {
        self.fields.get(name)
    }

    /// Gets a property by name.
    pub fn get_property(&self, name: &str) -> Option<&str> {
        self.properties.get(name).map(|s| s.as_str())
    }
}

/// The complete AST for a Mermaid diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ast {
    /// The root node of the AST.
    pub root: AstNode,
    /// The source text (for reference).
    #[serde(skip)]
    pub source: String,
}

impl Ast {
    /// Creates a new AST with the given root node.
    pub fn new(root: AstNode, source: impl Into<String>) -> Self {
        Self {
            root,
            source: source.into(),
        }
    }

    /// Gets the text for a span.
    pub fn text_for_span(&self, span: &Span) -> &str {
        span.text(&self.source)
    }

    /// Walks the AST, calling the visitor for each node.
    pub fn walk<F>(&self, mut visitor: F)
    where
        F: FnMut(&AstNode, usize),
    {
        self.walk_node(&self.root, 0, &mut visitor);
    }

    fn walk_node<F>(&self, node: &AstNode, depth: usize, visitor: &mut F)
    where
        F: FnMut(&AstNode, usize),
    {
        visitor(node, depth);
        for child in &node.children {
            self.walk_node(child, depth + 1, visitor);
        }
        for (_, field) in &node.fields {
            self.walk_node(field, depth + 1, visitor);
        }
    }

    /// Counts the total number of nodes in the AST.
    pub fn node_count(&self) -> usize {
        let mut count = 0;
        self.walk(|_, _| count += 1);
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span() {
        let span = Span::new(0, 10);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());

        let empty = Span::empty(5);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 5);
        let span2 = Span::new(10, 15);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_ast_node() {
        let mut root = AstNode::new(NodeKind::Root, Span::new(0, 100));
        let child = AstNode::with_text(NodeKind::Node, Span::new(0, 10), "A");
        root.add_child(child);

        assert_eq!(root.children.len(), 1);
        assert!(root.find_child(&NodeKind::Node).is_some());
    }

    #[test]
    fn test_ast_walk() {
        let mut root = AstNode::new(NodeKind::Root, Span::new(0, 100));
        root.add_child(AstNode::new(NodeKind::Node, Span::new(0, 10)));
        root.add_child(AstNode::new(NodeKind::Node, Span::new(10, 20)));

        let ast = Ast::new(root, "");
        assert_eq!(ast.node_count(), 3);
    }
}
