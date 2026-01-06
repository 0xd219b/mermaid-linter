//! Flowchart diagram parser.
//!
//! Supports the following syntax:
//! - `graph TD` / `graph LR` etc. (legacy flowchart)
//! - `flowchart TD` / `flowchart LR` etc. (v2 flowchart)
//!
//! # Example
//!
//! ```text
//! graph TD
//!     A[Start] --> B{Decision}
//!     B -->|Yes| C[Action 1]
//!     B -->|No| D[Action 2]
//!     C --> E[End]
//!     D --> E
//! ```

mod lexer;
mod parser;

pub use parser::FlowchartParser;

use crate::ast::Span;

/// Direction of the flowchart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    TopToBottom, // TB, TD
    BottomToTop, // BT
    LeftToRight, // LR
    RightToLeft, // RL
}

impl Direction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TB" | "TD" => Some(Direction::TopToBottom),
            "BT" => Some(Direction::BottomToTop),
            "LR" => Some(Direction::LeftToRight),
            "RL" => Some(Direction::RightToLeft),
            _ => None,
        }
    }
}

/// Shape of a node in the flowchart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeShape {
    #[default]
    Rectangle,      // [text]
    RoundedRect,    // (text)
    Stadium,        // ([text])
    Subroutine,     // [[text]]
    Cylindrical,    // [(text)]
    Circle,         // ((text))
    Asymmetric,     // >text]
    Rhombus,        // {text}
    Hexagon,        // {{text}}
    Parallelogram,  // [/text/]
    ParallelogramAlt, // [\text\]
    Trapezoid,      // [/text\]
    TrapezoidAlt,   // [\text/]
    DoubleCircle,   // (((text)))
}

/// Type of edge/link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkType {
    #[default]
    Arrow,          // -->
    Open,           // ---
    Dotted,         // -.-
    DottedArrow,    // -.->
    Thick,          // ==>
    ThickArrow,     // ==>
    Invisible,      // ~~~
}

/// A node in the flowchart.
#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub label: Option<String>,
    pub shape: NodeShape,
    pub span: Span,
}

/// A link between nodes.
#[derive(Debug, Clone)]
pub struct FlowLink {
    pub from: String,
    pub to: String,
    pub link_type: LinkType,
    pub label: Option<String>,
    pub span: Span,
}

/// A subgraph in the flowchart.
#[derive(Debug, Clone)]
pub struct Subgraph {
    pub id: String,
    pub label: Option<String>,
    pub direction: Option<Direction>,
    pub span: Span,
}

/// A style definition.
#[derive(Debug, Clone)]
pub struct StyleDef {
    pub node_ids: Vec<String>,
    pub styles: Vec<String>,
    pub span: Span,
}

/// A class definition.
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub styles: Vec<String>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_from_str() {
        assert_eq!(Direction::from_str("TD"), Some(Direction::TopToBottom));
        assert_eq!(Direction::from_str("TB"), Some(Direction::TopToBottom));
        assert_eq!(Direction::from_str("LR"), Some(Direction::LeftToRight));
        assert_eq!(Direction::from_str("RL"), Some(Direction::RightToLeft));
        assert_eq!(Direction::from_str("BT"), Some(Direction::BottomToTop));
        assert_eq!(Direction::from_str("XX"), None);
    }
}
