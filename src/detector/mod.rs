//! Diagram type detection.
//!
//! This module detects the type of Mermaid diagram from the source text.
//! The detection order matches Mermaid.js to ensure compatibility.

mod detectors;

pub use detectors::detect_type;

use serde::{Deserialize, Serialize};

/// Supported Mermaid diagram types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramType {
    // Special pseudo-diagrams
    /// Error diagram (text == "error")
    Error,
    /// Bad frontmatter (text starts with ---)
    BadFrontmatter,

    // Phase 1: Core diagrams
    /// Flowchart (graph keyword, legacy renderer)
    Flowchart,
    /// Flowchart v2 (flowchart keyword or graph with dagre-wrapper)
    FlowchartV2,
    /// Flowchart with ELK layout
    FlowchartElk,
    /// Sequence diagram
    Sequence,
    /// Class diagram (legacy)
    Class,
    /// Class diagram v2
    ClassDiagram,
    /// State diagram (legacy)
    State,
    /// State diagram v2
    StateDiagram,

    // Phase 2: Additional diagrams
    /// Entity-Relationship diagram
    Er,
    /// Gantt chart
    Gantt,
    /// User journey diagram
    Journey,
    /// Requirement diagram
    Requirement,
    /// Git graph
    GitGraph,
    /// XY chart
    XyChart,
    /// Quadrant chart
    QuadrantChart,

    // Phase 3: More diagrams
    /// C4 diagram (Context, Container, Component, Dynamic, Deployment)
    C4,
    /// Packet diagram
    Packet,
    /// Treemap
    Treemap,
    /// Sankey diagram
    Sankey,
    /// Kanban board
    Kanban,
    /// Block diagram
    Block,
    /// Radar chart
    Radar,
    /// Pie chart
    Pie,
    /// Info diagram
    Info,
    /// Timeline
    Timeline,
    /// Mindmap
    Mindmap,
    /// Architecture diagram
    Architecture,
}

impl DiagramType {
    /// Returns the string identifier for this diagram type.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagramType::Error => "error",
            DiagramType::BadFrontmatter => "---",
            DiagramType::Flowchart => "flowchart",
            DiagramType::FlowchartV2 => "flowchart-v2",
            DiagramType::FlowchartElk => "flowchart-elk",
            DiagramType::Sequence => "sequence",
            DiagramType::Class => "class",
            DiagramType::ClassDiagram => "classDiagram",
            DiagramType::State => "state",
            DiagramType::StateDiagram => "stateDiagram",
            DiagramType::Er => "er",
            DiagramType::Gantt => "gantt",
            DiagramType::Journey => "journey",
            DiagramType::Requirement => "requirement",
            DiagramType::GitGraph => "gitGraph",
            DiagramType::XyChart => "xychart",
            DiagramType::QuadrantChart => "quadrantChart",
            DiagramType::C4 => "c4",
            DiagramType::Packet => "packet",
            DiagramType::Treemap => "treemap",
            DiagramType::Sankey => "sankey",
            DiagramType::Kanban => "kanban",
            DiagramType::Block => "block",
            DiagramType::Radar => "radar",
            DiagramType::Pie => "pie",
            DiagramType::Info => "info",
            DiagramType::Timeline => "timeline",
            DiagramType::Mindmap => "mindmap",
            DiagramType::Architecture => "architecture",
        }
    }

    /// Returns true if this diagram type requires entity encoding.
    pub fn needs_entity_encoding(&self) -> bool {
        matches!(
            self,
            DiagramType::Flowchart | DiagramType::FlowchartV2 | DiagramType::FlowchartElk
        )
    }

    /// Returns true if this is a "large feature" diagram in Mermaid.
    pub fn is_large_feature(&self) -> bool {
        matches!(
            self,
            DiagramType::FlowchartElk | DiagramType::Mindmap | DiagramType::Architecture
        )
    }

    /// Returns true if this diagram uses Langium grammar (vs Jison).
    pub fn uses_langium(&self) -> bool {
        matches!(
            self,
            DiagramType::Pie
                | DiagramType::Info
                | DiagramType::Packet
                | DiagramType::GitGraph
                | DiagramType::Radar
                | DiagramType::Architecture
                | DiagramType::Treemap
        )
    }

    /// Returns all supported diagram types.
    pub fn all() -> &'static [DiagramType] {
        &[
            DiagramType::Error,
            DiagramType::BadFrontmatter,
            DiagramType::Flowchart,
            DiagramType::FlowchartV2,
            DiagramType::FlowchartElk,
            DiagramType::Sequence,
            DiagramType::Class,
            DiagramType::ClassDiagram,
            DiagramType::State,
            DiagramType::StateDiagram,
            DiagramType::Er,
            DiagramType::Gantt,
            DiagramType::Journey,
            DiagramType::Requirement,
            DiagramType::GitGraph,
            DiagramType::XyChart,
            DiagramType::QuadrantChart,
            DiagramType::C4,
            DiagramType::Packet,
            DiagramType::Treemap,
            DiagramType::Sankey,
            DiagramType::Kanban,
            DiagramType::Block,
            DiagramType::Radar,
            DiagramType::Pie,
            DiagramType::Info,
            DiagramType::Timeline,
            DiagramType::Mindmap,
            DiagramType::Architecture,
        ]
    }
}

impl std::fmt::Display for DiagramType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagram_type_str() {
        assert_eq!(DiagramType::Flowchart.as_str(), "flowchart");
        assert_eq!(DiagramType::Sequence.as_str(), "sequence");
        assert_eq!(DiagramType::ClassDiagram.as_str(), "classDiagram");
    }

    #[test]
    fn test_needs_entity_encoding() {
        assert!(DiagramType::Flowchart.needs_entity_encoding());
        assert!(DiagramType::FlowchartV2.needs_entity_encoding());
        assert!(!DiagramType::Sequence.needs_entity_encoding());
    }

    #[test]
    fn test_uses_langium() {
        assert!(DiagramType::Pie.uses_langium());
        assert!(DiagramType::Packet.uses_langium());
        assert!(!DiagramType::Flowchart.uses_langium());
    }
}
