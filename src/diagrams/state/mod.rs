//! State diagram parser.
//!
//! Supports the Mermaid state diagram syntax.
//!
//! # Example
//!
//! ```text
//! stateDiagram-v2
//!     [*] --> Still
//!     Still --> [*]
//!     Still --> Moving
//!     Moving --> Still
//!     Moving --> Crash
//!     Crash --> [*]
//! ```

mod lexer;
mod parser;

pub use parser::StateParser;

use crate::ast::Span;

/// Type of state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StateType {
    #[default]
    Normal,
    Start,      // [*]
    End,        // [*]
    Fork,       // <<fork>>
    Join,       // <<join>>
    Choice,     // <<choice>>
    Note,
}

/// A state in the diagram.
#[derive(Debug, Clone)]
pub struct StateDef {
    pub id: String,
    pub label: Option<String>,
    pub state_type: StateType,
    pub is_composite: bool,
    pub span: Span,
}

/// A transition between states.
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_type_default() {
        assert_eq!(StateType::default(), StateType::Normal);
    }
}
