//! Diagnostic error codes.

use serde::{Deserialize, Serialize};

/// Error codes for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticCode {
    // ========================================================================
    // General errors (E0xx)
    // ========================================================================
    /// Unknown or unrecognized diagram type.
    UnknownDiagram,
    /// Error during preprocessing.
    PreprocessError,

    // ========================================================================
    // Frontmatter/Directive errors (E1xx)
    // ========================================================================
    /// Error parsing YAML frontmatter.
    FrontmatterParseError,
    /// Error parsing directive (%%{...}%%).
    DirectiveParseError,
    /// Invalid directive type.
    InvalidDirective,
    /// Invalid JSON in directive.
    DirectiveJsonError,

    // ========================================================================
    // Lexer errors (E2xx)
    // ========================================================================
    /// Unknown or unexpected character.
    LexerError,
    /// Unterminated string literal.
    UnterminatedString,
    /// Invalid escape sequence.
    InvalidEscape,

    // ========================================================================
    // Parser errors (E3xx)
    // ========================================================================
    /// General parser error.
    ParserError,
    /// Unexpected token.
    UnexpectedToken,
    /// Expected a specific token.
    ExpectedToken,
    /// Unexpected end of input.
    UnexpectedEof,
    /// Invalid syntax.
    InvalidSyntax,
    /// Missing required element.
    MissingElement,
    /// Duplicate definition.
    DuplicateDefinition,

    // ========================================================================
    // Semantic errors (E4xx)
    // ========================================================================
    /// General semantic error.
    SemanticError,
    /// Reference to undefined node/participant.
    UndefinedReference,
    /// Invalid value for a field.
    InvalidValue,
    /// Constraint violation.
    ConstraintViolation,

    // ========================================================================
    // Flowchart-specific errors (E5xx)
    // ========================================================================
    /// Invalid flowchart direction.
    InvalidDirection,
    /// Invalid node shape.
    InvalidNodeShape,
    /// Invalid edge type.
    InvalidEdgeType,
    /// Subgraph error.
    SubgraphError,

    // ========================================================================
    // Sequence diagram-specific errors (E6xx)
    // ========================================================================
    /// Invalid arrow type in sequence diagram.
    InvalidArrowType,
    /// Invalid participant reference.
    InvalidParticipant,
    /// Invalid activation.
    InvalidActivation,

    // ========================================================================
    // Class diagram-specific errors (E7xx)
    // ========================================================================
    /// Invalid relationship type.
    InvalidRelationType,
    /// Invalid visibility modifier.
    InvalidVisibility,
    /// Invalid class member.
    InvalidMember,

    // ========================================================================
    // State diagram-specific errors (E8xx)
    // ========================================================================
    /// Invalid state type.
    InvalidStateType,
    /// Invalid transition.
    InvalidTransition,

    // ========================================================================
    // Other diagram-specific errors (E9xx)
    // ========================================================================
    /// Packet diagram: invalid bit range.
    PacketInvalidBitRange,
    /// Packet diagram: non-contiguous bits.
    PacketNonContiguous,
    /// Treemap: invalid node structure.
    TreemapInvalidStructure,
    /// Gantt: invalid date format.
    GanttInvalidDate,
}

impl DiagnosticCode {
    /// Returns the string code for this diagnostic.
    pub fn as_str(&self) -> &'static str {
        match self {
            // General errors
            DiagnosticCode::UnknownDiagram => "E001",
            DiagnosticCode::PreprocessError => "E002",

            // Frontmatter/Directive errors
            DiagnosticCode::FrontmatterParseError => "E101",
            DiagnosticCode::DirectiveParseError => "E102",
            DiagnosticCode::InvalidDirective => "E103",
            DiagnosticCode::DirectiveJsonError => "E104",

            // Lexer errors
            DiagnosticCode::LexerError => "E201",
            DiagnosticCode::UnterminatedString => "E202",
            DiagnosticCode::InvalidEscape => "E203",

            // Parser errors
            DiagnosticCode::ParserError => "E301",
            DiagnosticCode::UnexpectedToken => "E302",
            DiagnosticCode::ExpectedToken => "E303",
            DiagnosticCode::UnexpectedEof => "E304",
            DiagnosticCode::InvalidSyntax => "E305",
            DiagnosticCode::MissingElement => "E306",
            DiagnosticCode::DuplicateDefinition => "E307",

            // Semantic errors
            DiagnosticCode::SemanticError => "E401",
            DiagnosticCode::UndefinedReference => "E402",
            DiagnosticCode::InvalidValue => "E403",
            DiagnosticCode::ConstraintViolation => "E404",

            // Flowchart errors
            DiagnosticCode::InvalidDirection => "E501",
            DiagnosticCode::InvalidNodeShape => "E502",
            DiagnosticCode::InvalidEdgeType => "E503",
            DiagnosticCode::SubgraphError => "E504",

            // Sequence diagram errors
            DiagnosticCode::InvalidArrowType => "E601",
            DiagnosticCode::InvalidParticipant => "E602",
            DiagnosticCode::InvalidActivation => "E603",

            // Class diagram errors
            DiagnosticCode::InvalidRelationType => "E701",
            DiagnosticCode::InvalidVisibility => "E702",
            DiagnosticCode::InvalidMember => "E703",

            // State diagram errors
            DiagnosticCode::InvalidStateType => "E801",
            DiagnosticCode::InvalidTransition => "E802",

            // Other diagram errors
            DiagnosticCode::PacketInvalidBitRange => "E901",
            DiagnosticCode::PacketNonContiguous => "E902",
            DiagnosticCode::TreemapInvalidStructure => "E903",
            DiagnosticCode::GanttInvalidDate => "E904",
        }
    }

    /// Returns a human-readable category for this code.
    pub fn category(&self) -> &'static str {
        match self {
            DiagnosticCode::UnknownDiagram | DiagnosticCode::PreprocessError => "general",
            DiagnosticCode::FrontmatterParseError
            | DiagnosticCode::DirectiveParseError
            | DiagnosticCode::InvalidDirective
            | DiagnosticCode::DirectiveJsonError => "frontmatter/directive",
            DiagnosticCode::LexerError
            | DiagnosticCode::UnterminatedString
            | DiagnosticCode::InvalidEscape => "lexer",
            DiagnosticCode::ParserError
            | DiagnosticCode::UnexpectedToken
            | DiagnosticCode::ExpectedToken
            | DiagnosticCode::UnexpectedEof
            | DiagnosticCode::InvalidSyntax
            | DiagnosticCode::MissingElement
            | DiagnosticCode::DuplicateDefinition => "parser",
            DiagnosticCode::SemanticError
            | DiagnosticCode::UndefinedReference
            | DiagnosticCode::InvalidValue
            | DiagnosticCode::ConstraintViolation => "semantic",
            DiagnosticCode::InvalidDirection
            | DiagnosticCode::InvalidNodeShape
            | DiagnosticCode::InvalidEdgeType
            | DiagnosticCode::SubgraphError => "flowchart",
            DiagnosticCode::InvalidArrowType
            | DiagnosticCode::InvalidParticipant
            | DiagnosticCode::InvalidActivation => "sequence",
            DiagnosticCode::InvalidRelationType
            | DiagnosticCode::InvalidVisibility
            | DiagnosticCode::InvalidMember => "class",
            DiagnosticCode::InvalidStateType | DiagnosticCode::InvalidTransition => "state",
            DiagnosticCode::PacketInvalidBitRange
            | DiagnosticCode::PacketNonContiguous
            | DiagnosticCode::TreemapInvalidStructure
            | DiagnosticCode::GanttInvalidDate => "diagram-specific",
        }
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_codes() {
        assert_eq!(DiagnosticCode::UnknownDiagram.as_str(), "E001");
        assert_eq!(DiagnosticCode::ParserError.as_str(), "E301");
        assert_eq!(DiagnosticCode::SemanticError.category(), "semantic");
    }

    #[test]
    fn test_diagnostic_code_display() {
        let code = DiagnosticCode::ParserError;
        assert_eq!(format!("{}", code), "E301");
    }
}
