//! Parser error types.

use thiserror::Error;

use crate::ast::Span;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Lexer encountered an unexpected character.
    #[error("Unexpected character '{ch}' at position {position}")]
    UnexpectedChar { ch: char, position: usize },

    /// Lexer encountered an unterminated string.
    #[error("Unterminated string starting at position {position}")]
    UnterminatedString { position: usize },

    /// Parser encountered an unexpected token.
    #[error("Unexpected token '{found}' at position {position}, expected {expected}")]
    UnexpectedToken {
        found: String,
        expected: String,
        position: usize,
    },

    /// Parser reached end of input unexpectedly.
    #[error("Unexpected end of input, expected {expected}")]
    UnexpectedEof { expected: String },

    /// Invalid syntax.
    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String, span: Span },

    /// Semantic error during parsing.
    #[error("Semantic error: {message}")]
    SemanticError { message: String, span: Span },

    /// Generic parse error.
    #[error("{message}")]
    Generic { message: String, span: Span },
}

impl ParseError {
    /// Creates an unexpected character error.
    pub fn unexpected_char(ch: char, position: usize) -> Self {
        Self::UnexpectedChar { ch, position }
    }

    /// Creates an unterminated string error.
    pub fn unterminated_string(position: usize) -> Self {
        Self::UnterminatedString { position }
    }

    /// Creates an unexpected token error.
    pub fn unexpected_token(
        found: impl Into<String>,
        expected: impl Into<String>,
        position: usize,
    ) -> Self {
        Self::UnexpectedToken {
            found: found.into(),
            expected: expected.into(),
            position,
        }
    }

    /// Creates an unexpected end of input error.
    pub fn unexpected_eof(expected: impl Into<String>) -> Self {
        Self::UnexpectedEof {
            expected: expected.into(),
        }
    }

    /// Creates an invalid syntax error.
    pub fn invalid_syntax(message: impl Into<String>, span: Span) -> Self {
        Self::InvalidSyntax {
            message: message.into(),
            span,
        }
    }

    /// Creates a semantic error.
    pub fn semantic_error(message: impl Into<String>, span: Span) -> Self {
        Self::SemanticError {
            message: message.into(),
            span,
        }
    }

    /// Creates a generic error.
    pub fn generic(message: impl Into<String>, span: Span) -> Self {
        Self::Generic {
            message: message.into(),
            span,
        }
    }

    /// Gets the span for this error.
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedChar { position, .. } => Span::empty(*position),
            Self::UnterminatedString { position } => Span::empty(*position),
            Self::UnexpectedToken { position, .. } => Span::empty(*position),
            Self::UnexpectedEof { .. } => Span::default(),
            Self::InvalidSyntax { span, .. } => *span,
            Self::SemanticError { span, .. } => *span,
            Self::Generic { span, .. } => *span,
        }
    }

    /// Converts this error to a diagnostic.
    pub fn to_diagnostic(&self) -> Diagnostic {
        let (code, message) = match self {
            Self::UnexpectedChar { ch, .. } => {
                (DiagnosticCode::LexerError, format!("Unexpected character '{}'", ch))
            }
            Self::UnterminatedString { .. } => {
                (DiagnosticCode::UnterminatedString, "Unterminated string".to_string())
            }
            Self::UnexpectedToken { found, expected, .. } => (
                DiagnosticCode::UnexpectedToken,
                format!("Unexpected token '{}', expected {}", found, expected),
            ),
            Self::UnexpectedEof { expected } => (
                DiagnosticCode::UnexpectedEof,
                format!("Unexpected end of input, expected {}", expected),
            ),
            Self::InvalidSyntax { message, .. } => {
                (DiagnosticCode::InvalidSyntax, message.clone())
            }
            Self::SemanticError { message, .. } => {
                (DiagnosticCode::SemanticError, message.clone())
            }
            Self::Generic { message, .. } => (DiagnosticCode::ParserError, message.clone()),
        };

        Diagnostic::new(code, message, Severity::Error, self.span())
    }
}

impl From<ParseError> for Diagnostic {
    fn from(error: ParseError) -> Self {
        error.to_diagnostic()
    }
}

impl From<ParseError> for Vec<Diagnostic> {
    fn from(error: ParseError) -> Self {
        vec![error.to_diagnostic()]
    }
}

/// A collection of parse errors.
#[derive(Debug, Default)]
pub struct ParseErrors {
    errors: Vec<ParseError>,
}

impl ParseErrors {
    /// Creates a new empty error collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an error to the collection.
    pub fn push(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    /// Returns true if there are no errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the number of errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Converts to a vector of diagnostics.
    pub fn to_diagnostics(&self) -> Vec<Diagnostic> {
        self.errors.iter().map(|e| e.to_diagnostic()).collect()
    }

    /// Consumes and converts to a vector of diagnostics.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.errors.into_iter().map(|e| e.to_diagnostic()).collect()
    }
}

impl IntoIterator for ParseErrors {
    type Item = ParseError;
    type IntoIter = std::vec::IntoIter<ParseError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unexpected_char() {
        let error = ParseError::unexpected_char('@', 10);
        let diag = error.to_diagnostic();

        assert_eq!(diag.code, DiagnosticCode::LexerError);
        assert!(diag.message.contains('@'));
    }

    #[test]
    fn test_unexpected_token() {
        let error = ParseError::unexpected_token("foo", "identifier", 5);
        let diag = error.to_diagnostic();

        assert_eq!(diag.code, DiagnosticCode::UnexpectedToken);
        assert!(diag.message.contains("foo"));
    }

    #[test]
    fn test_parse_errors_collection() {
        let mut errors = ParseErrors::new();
        errors.push(ParseError::unexpected_char('!', 0));
        errors.push(ParseError::unexpected_eof("end of statement"));

        assert_eq!(errors.len(), 2);

        let diagnostics = errors.to_diagnostics();
        assert_eq!(diagnostics.len(), 2);
    }
}
