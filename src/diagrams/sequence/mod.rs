//! Sequence diagram parser.
//!
//! Supports the Mermaid sequence diagram syntax.
//!
//! # Example
//!
//! ```text
//! sequenceDiagram
//!     participant Alice
//!     participant Bob
//!     Alice->>Bob: Hello Bob
//!     Bob-->>Alice: Hi Alice
//! ```

mod lexer;
mod parser;

pub use parser::SequenceParser;

use crate::ast::Span;

/// Type of participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParticipantType {
    #[default]
    Participant,
    Actor,
}

/// Type of arrow/message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowType {
    #[default]
    Solid,          // ->>
    Dotted,         // -->>
    SolidLine,      // ->
    DottedLine,     // -->
    SolidCross,     // -x
    DottedCross,    // --x
    SolidAsync,     // -)
    DottedAsync,    // --)
}

impl ArrowType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "->>" => Some(ArrowType::Solid),
            "-->>" => Some(ArrowType::Dotted),
            "->" => Some(ArrowType::SolidLine),
            "-->" => Some(ArrowType::DottedLine),
            "-x" | "-X" => Some(ArrowType::SolidCross),
            "--x" | "--X" => Some(ArrowType::DottedCross),
            "-)" => Some(ArrowType::SolidAsync),
            "--)" => Some(ArrowType::DottedAsync),
            _ => None,
        }
    }
}

/// Position of a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotePosition {
    LeftOf(String),
    RightOf(String),
    Over(Vec<String>),
}

/// A participant in the diagram.
#[derive(Debug, Clone)]
pub struct Participant {
    pub id: String,
    pub alias: Option<String>,
    pub participant_type: ParticipantType,
    pub span: Span,
}

/// A message between participants.
#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub arrow_type: ArrowType,
    pub text: String,
    pub span: Span,
}

/// A note in the diagram.
#[derive(Debug, Clone)]
pub struct Note {
    pub position: NotePosition,
    pub text: String,
    pub span: Span,
}

/// An activation marker.
#[derive(Debug, Clone)]
pub struct Activation {
    pub participant: String,
    pub is_activate: bool, // true = activate, false = deactivate
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_type_from_str() {
        assert_eq!(ArrowType::from_str("->>"), Some(ArrowType::Solid));
        assert_eq!(ArrowType::from_str("-->>"), Some(ArrowType::Dotted));
        assert_eq!(ArrowType::from_str("-x"), Some(ArrowType::SolidCross));
        assert_eq!(ArrowType::from_str("invalid"), None);
    }
}
