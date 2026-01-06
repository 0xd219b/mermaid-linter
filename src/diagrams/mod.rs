//! Diagram-specific parsers.
//!
//! Each diagram type has its own submodule with lexer, parser, and AST definitions.

pub mod class;
pub mod er;
pub mod flowchart;
pub mod gantt;
pub mod gitgraph;
pub mod journey;
pub mod pie;
pub mod sequence;
pub mod state;
