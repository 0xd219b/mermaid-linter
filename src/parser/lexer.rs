//! Common lexer utilities.
//!
//! This module provides common utilities for building diagram-specific lexers.

use crate::ast::Span;

/// A token from the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T> {
    /// The type of token.
    pub kind: T,
    /// The span in the source.
    pub span: Span,
    /// The text of the token.
    pub text: String,
}

impl<T> Token<T> {
    /// Creates a new token.
    pub fn new(kind: T, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
        }
    }

    /// Returns true if this token is of the given kind.
    pub fn is(&self, kind: &T) -> bool
    where
        T: PartialEq,
    {
        &self.kind == kind
    }
}

/// A position in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    /// Byte offset.
    pub offset: usize,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
}

impl Position {
    /// Creates a new position.
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    /// Creates a position at the start of the source.
    pub fn start() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

/// Base lexer that tracks position.
#[derive(Debug, Clone)]
pub struct BaseLexer<'a> {
    /// The source text.
    source: &'a str,
    /// Current position.
    pos: Position,
    /// Iterator over characters.
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
}

impl<'a> BaseLexer<'a> {
    /// Creates a new base lexer.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: Position::start(),
            chars: source.char_indices().peekable(),
        }
    }

    /// Returns the source text.
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// Returns the current position.
    pub fn position(&self) -> Position {
        self.pos
    }

    /// Returns true if at end of input.
    pub fn is_eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    /// Peeks at the next character without consuming it.
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    /// Peeks at the character n positions ahead.
    pub fn peek_n(&self, n: usize) -> Option<char> {
        self.source[self.pos.offset..].chars().nth(n)
    }

    /// Consumes and returns the next character.
    pub fn advance(&mut self) -> Option<char> {
        let (_, ch) = self.chars.next()?;

        self.pos.offset += ch.len_utf8();

        if ch == '\n' {
            self.pos.line += 1;
            self.pos.column = 1;
        } else {
            self.pos.column += 1;
        }

        Some(ch)
    }

    /// Consumes characters while the predicate is true.
    pub fn advance_while<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start = self.pos.offset;
        while let Some(ch) = self.peek() {
            if !predicate(ch) {
                break;
            }
            self.advance();
        }
        &self.source[start..self.pos.offset]
    }

    /// Skips whitespace characters.
    pub fn skip_whitespace(&mut self) {
        self.advance_while(|c| c.is_whitespace());
    }

    /// Skips whitespace except newlines.
    pub fn skip_horizontal_whitespace(&mut self) {
        self.advance_while(|c| c == ' ' || c == '\t');
    }

    /// Consumes a string if it matches.
    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.source[self.pos.offset..].starts_with(s) {
            for _ in s.chars() {
                self.advance();
            }
            true
        } else {
            false
        }

    }

    /// Returns the remaining source text.
    pub fn remaining(&self) -> &'a str {
        &self.source[self.pos.offset..]
    }

    /// Creates a span from start to current position.
    pub fn span_from(&self, start: usize) -> Span {
        Span::new(start, self.pos.offset)
    }

    /// Gets text for a span.
    pub fn text_for_span(&self, span: &Span) -> &'a str {
        &self.source[span.start..span.end]
    }
}

/// Utilities for common lexing patterns.
pub mod patterns {
    use super::*;

    /// Checks if a character can start an identifier.
    pub fn is_ident_start(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    /// Checks if a character can continue an identifier.
    pub fn is_ident_continue(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    /// Checks if a character is a digit.
    pub fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    /// Checks if a character is a hex digit.
    pub fn is_hex_digit(ch: char) -> bool {
        ch.is_ascii_hexdigit()
    }

    /// Checks if a string is a valid identifier.
    pub fn is_identifier(s: &str) -> bool {
        let mut chars = s.chars();
        chars
            .next()
            .map(is_ident_start)
            .unwrap_or(false)
            && chars.all(is_ident_continue)
    }

    /// Reads an identifier from the lexer.
    pub fn read_identifier<'a>(lexer: &mut BaseLexer<'a>) -> Option<&'a str> {
        if !lexer.peek().map(is_ident_start).unwrap_or(false) {
            return None;
        }

        let start = lexer.position().offset;
        lexer.advance_while(is_ident_continue);
        Some(&lexer.source()[start..lexer.position().offset])
    }

    /// Reads a number from the lexer.
    pub fn read_number<'a>(lexer: &mut BaseLexer<'a>) -> Option<&'a str> {
        if !lexer.peek().map(is_digit).unwrap_or(false) {
            return None;
        }

        let start = lexer.position().offset;
        lexer.advance_while(is_digit);

        // Handle decimal point
        if lexer.peek() == Some('.') && lexer.peek_n(1).map(is_digit).unwrap_or(false) {
            lexer.advance(); // consume '.'
            lexer.advance_while(is_digit);
        }

        Some(&lexer.source()[start..lexer.position().offset])
    }

    /// Reads a quoted string from the lexer.
    pub fn read_quoted_string<'a>(
        lexer: &mut BaseLexer<'a>,
        quote: char,
    ) -> Result<&'a str, &'static str> {
        if lexer.peek() != Some(quote) {
            return Err("Expected opening quote");
        }

        let start = lexer.position().offset;
        lexer.advance(); // consume opening quote

        while let Some(ch) = lexer.peek() {
            if ch == quote {
                lexer.advance(); // consume closing quote
                return Ok(&lexer.source()[start..lexer.position().offset]);
            }
            if ch == '\\' {
                lexer.advance(); // consume backslash
                lexer.advance(); // consume escaped char
            } else if ch == '\n' {
                return Err("Unexpected newline in string");
            } else {
                lexer.advance();
            }
        }

        Err("Unterminated string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_lexer_advance() {
        let mut lexer = BaseLexer::new("hello");

        assert_eq!(lexer.advance(), Some('h'));
        assert_eq!(lexer.advance(), Some('e'));
        assert_eq!(lexer.position().offset, 2);
        assert_eq!(lexer.position().column, 3);
    }

    #[test]
    fn test_base_lexer_newline() {
        let mut lexer = BaseLexer::new("a\nb");

        lexer.advance(); // 'a'
        lexer.advance(); // '\n'

        assert_eq!(lexer.position().line, 2);
        assert_eq!(lexer.position().column, 1);
    }

    #[test]
    fn test_base_lexer_peek() {
        let mut lexer = BaseLexer::new("abc");

        assert_eq!(lexer.peek(), Some('a'));
        assert_eq!(lexer.peek(), Some('a')); // peek doesn't consume

        lexer.advance();
        assert_eq!(lexer.peek(), Some('b'));
    }

    #[test]
    fn test_read_identifier() {
        let mut lexer = BaseLexer::new("hello123 world");

        let ident = patterns::read_identifier(&mut lexer);
        assert_eq!(ident, Some("hello123"));
    }

    #[test]
    fn test_read_number() {
        let mut lexer = BaseLexer::new("123.45 abc");

        let num = patterns::read_number(&mut lexer);
        assert_eq!(num, Some("123.45"));
    }

    #[test]
    fn test_read_quoted_string() {
        let mut lexer = BaseLexer::new("\"hello world\" rest");

        let s = patterns::read_quoted_string(&mut lexer, '"');
        assert_eq!(s, Ok("\"hello world\""));
    }

    #[test]
    fn test_advance_while() {
        let mut lexer = BaseLexer::new("aaabbb");

        let result = lexer.advance_while(|c| c == 'a');
        assert_eq!(result, "aaa");
        assert_eq!(lexer.peek(), Some('b'));
    }
}
