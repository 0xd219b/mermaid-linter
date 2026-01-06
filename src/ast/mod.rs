//! Abstract Syntax Tree (AST) definitions for Mermaid diagrams.

mod common;
mod typed;

pub use common::{Ast, AstNode, NodeKind, Span};
pub use typed::*;

use serde::{Deserialize, Serialize};

/// Position in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
    /// Byte offset from start of source.
    pub offset: usize,
}

impl Position {
    /// Creates a new position.
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Creates a position at the start of the source.
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

/// A range in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Range {
    /// Start position.
    pub start: Position,
    /// End position.
    pub end: Position,
}

impl Range {
    /// Creates a new range.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Creates a range spanning a single position.
    pub fn point(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Creates a range from byte offsets and source text.
    pub fn from_offsets(source: &str, start_offset: usize, end_offset: usize) -> Self {
        let start = offset_to_position(source, start_offset);
        let end = offset_to_position(source, end_offset);
        Self { start, end }
    }
}

/// Converts a byte offset to a position (line, column).
fn offset_to_position(source: &str, offset: usize) -> Position {
    let offset = offset.min(source.len());
    let mut line = 1;
    let mut column = 1;
    let mut current_offset = 0;

    for (idx, ch) in source.char_indices() {
        if idx >= offset {
            break;
        }
        current_offset = idx;
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    Position {
        line,
        column,
        offset: current_offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_start() {
        let pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);
    }

    #[test]
    fn test_offset_to_position() {
        let source = "line1\nline2\nline3";

        // Start of source
        let pos = offset_to_position(source, 0);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);

        // Middle of first line
        let pos = offset_to_position(source, 3);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 4);

        // Start of second line
        let pos = offset_to_position(source, 6);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);

        // Middle of third line
        let pos = offset_to_position(source, 14);
        assert_eq!(pos.line, 3);
        assert_eq!(pos.column, 3);
    }

    #[test]
    fn test_range_from_offsets() {
        let source = "graph TD\n    A --> B";
        let range = Range::from_offsets(source, 9, 19);

        assert_eq!(range.start.line, 2);
        assert_eq!(range.end.line, 2);
    }
}
