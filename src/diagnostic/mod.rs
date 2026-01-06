//! Diagnostic types for reporting errors and warnings.

mod codes;

pub use codes::DiagnosticCode;

use crate::ast::Span;
use crate::detector::DiagramType;
use serde::{Deserialize, Serialize};

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// An error that prevents successful parsing.
    Error,
    /// A warning that doesn't prevent parsing but indicates a potential issue.
    Warning,
    /// An informational message.
    Info,
    /// A hint for improvement.
    Hint,
}

impl Severity {
    /// Returns true if this is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Severity::Error)
    }

    /// Returns the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        }
    }
}

/// A diagnostic message from parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Error code identifying the type of error.
    pub code: DiagnosticCode,
    /// Human-readable message describing the error.
    pub message: String,
    /// Severity of the diagnostic.
    pub severity: Severity,
    /// Location in the source code.
    pub span: Span,
    /// The diagram type, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagram_type: Option<DiagramType>,
    /// Additional notes or hints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Related diagnostics (e.g., "defined here").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedDiagnostic>,
}

impl Diagnostic {
    /// Creates a new diagnostic.
    pub fn new(code: DiagnosticCode, message: String, severity: Severity, span: Span) -> Self {
        Self {
            code,
            message,
            severity,
            span,
            diagram_type: None,
            notes: Vec::new(),
            related: Vec::new(),
        }
    }

    /// Creates an error diagnostic.
    pub fn error(code: DiagnosticCode, message: impl Into<String>, span: Span) -> Self {
        Self::new(code, message.into(), Severity::Error, span)
    }

    /// Creates a warning diagnostic.
    pub fn warning(code: DiagnosticCode, message: impl Into<String>, span: Span) -> Self {
        Self::new(code, message.into(), Severity::Warning, span)
    }

    /// Sets the diagram type.
    pub fn with_diagram_type(mut self, diagram_type: DiagramType) -> Self {
        self.diagram_type = Some(diagram_type);
        self
    }

    /// Adds a note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Adds a related diagnostic.
    pub fn with_related(mut self, related: RelatedDiagnostic) -> Self {
        self.related.push(related);
        self
    }

    /// Formats the diagnostic for display.
    pub fn format(&self, source: &str) -> String {
        let location = self.format_location(source);
        let mut result = format!(
            "{}: [{}] {}\n  --> {}",
            self.severity.as_str(),
            self.code.as_str(),
            self.message,
            location
        );

        // Add source context if available
        if !self.span.is_empty() {
            if let Some(context) = self.get_source_context(source) {
                result.push_str(&format!("\n{}", context));
            }
        }

        // Add notes
        for note in &self.notes {
            result.push_str(&format!("\n  = note: {}", note));
        }

        result
    }

    /// Formats the location for display.
    fn format_location(&self, source: &str) -> String {
        let (line, col) = self.offset_to_line_col(source, self.span.start);
        format!("{}:{}", line, col)
    }

    /// Gets source context around the error.
    fn get_source_context(&self, source: &str) -> Option<String> {
        let (line_num, col) = self.offset_to_line_col(source, self.span.start);
        let lines: Vec<&str> = source.lines().collect();

        if line_num == 0 || line_num > lines.len() {
            return None;
        }

        let line = lines[line_num - 1];
        let line_num_str = format!("{}", line_num);
        let padding = " ".repeat(line_num_str.len());

        let mut result = format!("{} |\n", padding);
        result.push_str(&format!("{} | {}\n", line_num_str, line));

        // Add caret pointing to the error
        let caret_padding = " ".repeat(col.saturating_sub(1));
        let caret_len = (self.span.end - self.span.start).min(line.len() - col + 1).max(1);
        let carets = "^".repeat(caret_len);
        result.push_str(&format!("{} | {}{}", padding, caret_padding, carets));

        Some(result)
    }

    /// Converts a byte offset to line and column numbers.
    fn offset_to_line_col(&self, source: &str, offset: usize) -> (usize, usize) {
        let offset = offset.min(source.len());
        let mut line = 1;
        let mut col = 1;

        for (idx, ch) in source.char_indices() {
            if idx >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }
}

/// A related diagnostic providing additional context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDiagnostic {
    /// Message for this related location.
    pub message: String,
    /// Location in the source.
    pub span: Span,
}

impl RelatedDiagnostic {
    /// Creates a new related diagnostic.
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

/// A collection of diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Creates a new empty diagnostics collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a diagnostic.
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Adds an error.
    pub fn error(&mut self, code: DiagnosticCode, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::error(code, message, span));
    }

    /// Adds a warning.
    pub fn warning(&mut self, code: DiagnosticCode, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::warning(code, message, span));
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity.is_error())
    }

    /// Returns the number of errors.
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity.is_error())
            .count()
    }

    /// Returns the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .count()
    }

    /// Returns all diagnostics.
    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns only errors.
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity.is_error())
            .collect()
    }

    /// Returns only warnings.
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .collect()
    }

    /// Consumes the collection and returns the diagnostics.
    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns the number of diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error(
            DiagnosticCode::ParserError,
            "unexpected token",
            Span::new(10, 15),
        );

        assert_eq!(diag.code, DiagnosticCode::ParserError);
        assert!(diag.severity.is_error());
    }

    #[test]
    fn test_diagnostic_format() {
        let source = "graph TD\n    A --> B\n    invalid";
        let diag = Diagnostic::error(
            DiagnosticCode::ParserError,
            "unexpected token 'invalid'",
            Span::new(24, 31),
        );

        let formatted = diag.format(source);
        assert!(formatted.contains("error"));
        assert!(formatted.contains("unexpected token"));
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diagnostics = Diagnostics::new();

        diagnostics.error(DiagnosticCode::ParserError, "error 1", Span::new(0, 5));
        diagnostics.warning(DiagnosticCode::SemanticError, "warning 1", Span::new(10, 15));

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics.error_count(), 1);
        assert_eq!(diagnostics.warning_count(), 1);
        assert!(diagnostics.has_errors());
    }
}
