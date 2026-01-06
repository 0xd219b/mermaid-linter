//! Sequence diagram lexer.

use logos::Logos;

use crate::ast::Span;

/// Token types for sequence diagram parsing.
#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
pub enum SeqToken {
    // Keywords
    #[token("sequenceDiagram", ignore(case))]
    SequenceDiagram,

    #[token("participant", ignore(case))]
    Participant,

    #[token("actor", ignore(case))]
    Actor,

    #[token("as", ignore(case))]
    As,

    #[token("note", ignore(case))]
    Note,

    #[token("left of", ignore(case))]
    LeftOf,

    #[token("right of", ignore(case))]
    RightOf,

    #[token("over", ignore(case))]
    Over,

    #[token("activate", ignore(case))]
    Activate,

    #[token("deactivate", ignore(case))]
    Deactivate,

    #[token("loop", ignore(case))]
    Loop,

    #[token("end", ignore(case))]
    End,

    #[token("alt", ignore(case))]
    Alt,

    #[token("else", ignore(case))]
    Else,

    #[token("opt", ignore(case))]
    Opt,

    #[token("par", ignore(case))]
    Par,

    #[token("and", ignore(case))]
    And,

    #[token("critical", ignore(case))]
    Critical,

    #[token("option", ignore(case))]
    Option,

    #[token("break", ignore(case))]
    Break,

    #[token("rect", ignore(case))]
    Rect,

    #[token("autonumber", ignore(case))]
    Autonumber,

    #[token("title", ignore(case))]
    Title,

    #[token("box", ignore(case))]
    Box,

    #[token("create", ignore(case))]
    Create,

    #[token("destroy", ignore(case))]
    Destroy,

    #[token("links", ignore(case))]
    Links,

    #[token("link", ignore(case))]
    Link,

    // Arrow types (order matters - longer patterns first)
    #[token("->>")]
    SolidArrow,

    #[token("-->>")]
    DottedArrow,

    #[token("->")]
    SolidLine,

    #[token("-->")]
    DottedLine,

    #[token("-x")]
    SolidCross,

    #[token("-X")]
    SolidCrossUpper,

    #[token("--x")]
    DottedCross,

    #[token("--X")]
    DottedCrossUpper,

    #[token("-)")]
    SolidAsync,

    #[token("--)")]
    DottedAsync,

    // Other tokens
    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("\n")]
    Newline,

    // Quoted strings
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleQuotedString,

    #[regex(r#"'([^'\\]|\\.)*'"#)]
    SingleQuotedString,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Numbers
    #[regex(r"[0-9]+")]
    Number,

    // Text (for messages, notes) - lower priority so other patterns match first
    // Note: Excludes spaces so identifiers can be matched separately
    #[regex(r#"[^\n:,\-+"' \t]+"#, priority = 1)]
    Text,
}

/// A positioned token.
#[derive(Debug, Clone)]
pub struct PositionedToken {
    pub kind: SeqToken,
    pub span: Span,
    pub text: String,
}

/// Tokenize sequence diagram source code.
pub fn tokenize(source: &str) -> Vec<PositionedToken> {
    let mut tokens = Vec::new();
    let mut lexer = SeqToken::lexer(source);

    while let Some(result) = lexer.next() {
        if let Ok(kind) = result {
            let span = lexer.span();
            let text = lexer.slice().to_string();
            tokens.push(PositionedToken {
                kind,
                span: Span::new(span.start, span.end),
                text,
            });
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_declaration() {
        let tokens = tokenize("sequenceDiagram");
        assert!(tokens.iter().any(|t| t.kind == SeqToken::SequenceDiagram));
    }

    #[test]
    fn test_tokenize_participant() {
        let tokens = tokenize("participant Alice");
        assert!(tokens.iter().any(|t| t.kind == SeqToken::Participant));
        assert!(tokens.iter().any(|t| t.kind == SeqToken::Identifier));
    }

    #[test]
    fn test_tokenize_message() {
        let tokens = tokenize("Alice->>Bob: Hello");
        assert!(tokens.iter().any(|t| t.kind == SeqToken::SolidArrow));
        assert!(tokens.iter().any(|t| t.kind == SeqToken::Colon));
    }

    #[test]
    fn test_tokenize_note() {
        let tokens = tokenize("Note right of Alice: Text");
        assert!(tokens.iter().any(|t| t.kind == SeqToken::Note));
        assert!(tokens.iter().any(|t| t.kind == SeqToken::RightOf));
    }

    #[test]
    fn test_tokenize_loop() {
        let tokens = tokenize("loop Every minute\n    Alice->>Bob: Hi\nend");
        assert!(tokens.iter().any(|t| t.kind == SeqToken::Loop));
        assert!(tokens.iter().any(|t| t.kind == SeqToken::End));
    }
}
