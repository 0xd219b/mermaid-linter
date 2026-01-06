//! Parser infrastructure and diagram-specific parsers.
//!
//! This module provides the common infrastructure for parsing Mermaid diagrams,
//! as well as the specific parsers for each diagram type.

pub mod error;
pub mod lexer;
pub mod traits;

use crate::ast::Ast;
use crate::config::MermaidConfig;
use crate::detector::DiagramType;
use crate::diagnostic::Diagnostic;

pub use error::ParseError;
pub use traits::DiagramParser;

/// Parses a diagram of the given type.
///
/// This is the main entry point for diagram-specific parsing.
/// It delegates to the appropriate parser based on the diagram type.
pub fn parse_diagram(
    diagram_type: DiagramType,
    code: &str,
    config: &MermaidConfig,
) -> Result<Ast, Vec<Diagnostic>> {
    match diagram_type {
        // Special cases that always fail
        DiagramType::Error | DiagramType::BadFrontmatter => {
            // These should be handled before calling parse_diagram
            unreachable!("Error and BadFrontmatter should be handled earlier");
        }

        // Phase 1 diagrams
        DiagramType::Flowchart | DiagramType::FlowchartV2 | DiagramType::FlowchartElk => {
            crate::diagrams::flowchart::FlowchartParser::new().parse(code, config)
        }
        DiagramType::Sequence => {
            crate::diagrams::sequence::SequenceParser::new().parse(code, config)
        }
        DiagramType::Class | DiagramType::ClassDiagram => {
            crate::diagrams::class::ClassParser::new().parse(code, config)
        }
        DiagramType::State | DiagramType::StateDiagram => {
            crate::diagrams::state::StateParser::new().parse(code, config)
        }

        // Phase 3 diagrams
        DiagramType::Er => {
            crate::diagrams::er::ErParser::new(code).parse()
        }
        DiagramType::Gantt => {
            crate::diagrams::gantt::GanttParser::new(code).parse()
        }
        DiagramType::Journey => {
            crate::diagrams::journey::JourneyParser::new(code).parse()
        }
        DiagramType::Pie => {
            crate::diagrams::pie::PieParser::new(code).parse()
        }
        DiagramType::GitGraph => {
            crate::diagrams::gitgraph::GitGraphParser::new(code).parse()
        }

        // Phase 3+ diagrams - stub implementations for now
        _ => {
            // Return a minimal AST for unsupported diagram types
            use crate::ast::{AstNode, NodeKind, Span};

            let mut root = AstNode::new(NodeKind::Root, Span::new(0, code.len()));
            root.add_property("diagram_type", diagram_type.as_str());
            root.add_property("status", "stub");

            Ok(Ast::new(root, code.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flowchart() {
        let code = "graph TD\n    A --> B";
        let result = parse_diagram(DiagramType::Flowchart, code, &MermaidConfig::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sequence() {
        let code = "sequenceDiagram\n    Alice->>Bob: Hello";
        let result = parse_diagram(DiagramType::Sequence, code, &MermaidConfig::default());
        assert!(result.is_ok());
    }
}
