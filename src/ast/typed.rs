//! Typed AST definitions for specific diagram types.
//!
//! These provide more specific type information than the generic AST
//! for diagrams that need semantic validation.

use serde::{Deserialize, Serialize};

use super::Span;

// ============================================================================
// Flowchart AST
// ============================================================================

/// Direction of a flowchart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

impl FlowDirection {
    /// Parses a direction from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TB" | "TD" => Some(FlowDirection::TopToBottom),
            "BT" => Some(FlowDirection::BottomToTop),
            "LR" => Some(FlowDirection::LeftToRight),
            "RL" => Some(FlowDirection::RightToLeft),
            _ => None,
        }
    }
}

/// Shape of a flowchart node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeShape {
    Rectangle,
    RoundedRect,
    Stadium,
    Subroutine,
    Cylindrical,
    Circle,
    Asymmetric,
    Rhombus,
    Hexagon,
    Parallelogram,
    ParallelogramAlt,
    Trapezoid,
    TrapezoidAlt,
    DoubleCircle,
}

/// A node in a flowchart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub label: Option<String>,
    pub shape: NodeShape,
    pub span: Span,
}

/// Type of edge in a flowchart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Arrow,
    Open,
    Dotted,
    Thick,
    DottedArrow,
    ThickArrow,
}

/// An edge in a flowchart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub label: Option<String>,
    pub span: Span,
}

/// A subgraph in a flowchart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSubgraph {
    pub id: String,
    pub label: Option<String>,
    pub direction: Option<FlowDirection>,
    pub span: Span,
}

// ============================================================================
// Sequence Diagram AST
// ============================================================================

/// Type of participant in a sequence diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantType {
    Participant,
    Actor,
}

/// A participant in a sequence diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeqParticipant {
    pub id: String,
    pub alias: Option<String>,
    pub participant_type: ParticipantType,
    pub span: Span,
}

/// Type of arrow in a sequence diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeqArrowType {
    Solid,
    Dotted,
    SolidCross,
    DottedCross,
    SolidAsync,
    DottedAsync,
}

/// A message in a sequence diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeqMessage {
    pub from: String,
    pub to: String,
    pub arrow_type: SeqArrowType,
    pub text: String,
    pub span: Span,
}

/// Position of a note in a sequence diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotePosition {
    LeftOf(String),
    RightOf(String),
    Over(Vec<String>),
}

/// A note in a sequence diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeqNote {
    pub position: NotePosition,
    pub text: String,
    pub span: Span,
}

// ============================================================================
// Class Diagram AST
// ============================================================================

/// Visibility of a class member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Package,
}

/// A member (attribute or method) of a class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMember {
    pub name: String,
    pub member_type: Option<String>,
    pub visibility: Option<Visibility>,
    pub is_static: bool,
    pub is_abstract: bool,
    pub span: Span,
}

/// Type of relationship between classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    Inheritance,
    Composition,
    Aggregation,
    Association,
    Dependency,
    Realization,
    Link,
}

/// Cardinality of a relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cardinality {
    pub min: Option<String>,
    pub max: Option<String>,
}

/// A class in a class diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDef {
    pub name: String,
    pub stereotype: Option<String>,
    pub attributes: Vec<ClassMember>,
    pub methods: Vec<ClassMember>,
    pub span: Span,
}

/// A relationship between classes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassRelation {
    pub from: String,
    pub to: String,
    pub relation_type: RelationType,
    pub label: Option<String>,
    pub from_cardinality: Option<Cardinality>,
    pub to_cardinality: Option<Cardinality>,
    pub span: Span,
}

// ============================================================================
// State Diagram AST
// ============================================================================

/// Type of state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    Normal,
    Start,
    End,
    Fork,
    Join,
    Choice,
    Note,
}

/// A state in a state diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDef {
    pub id: String,
    pub label: Option<String>,
    pub state_type: StateType,
    pub is_composite: bool,
    pub span: Span,
}

/// A transition between states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub span: Span,
}

// ============================================================================
// Packet Diagram AST (requires semantic validation)
// ============================================================================

/// A row in a packet diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketRow {
    pub fields: Vec<PacketField>,
    pub span: Span,
}

/// A field in a packet diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketField {
    pub start: u32,
    pub end: u32,
    pub label: String,
    pub span: Span,
}

impl PacketField {
    /// Validates the packet field.
    pub fn validate(&self) -> Result<(), String> {
        if self.end < self.start {
            return Err(format!(
                "Packet field end ({}) must be >= start ({})",
                self.end, self.start
            ));
        }
        if self.end == self.start && self.end == 0 {
            return Err("Packet field bits cannot be 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_direction() {
        assert_eq!(
            FlowDirection::from_str("TD"),
            Some(FlowDirection::TopToBottom)
        );
        assert_eq!(
            FlowDirection::from_str("LR"),
            Some(FlowDirection::LeftToRight)
        );
        assert_eq!(FlowDirection::from_str("invalid"), None);
    }

    #[test]
    fn test_packet_field_validation() {
        let valid = PacketField {
            start: 0,
            end: 7,
            label: "byte".to_string(),
            span: Span::default(),
        };
        assert!(valid.validate().is_ok());

        let invalid = PacketField {
            start: 8,
            end: 0,
            label: "invalid".to_string(),
            span: Span::default(),
        };
        assert!(invalid.validate().is_err());
    }
}
