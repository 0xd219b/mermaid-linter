//! Flowchart lexer.

use logos::Logos;

use crate::ast::Span;

/// Token types for flowchart parsing.
#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]  // Skip whitespace but not newlines
pub enum FlowToken {
    // Keywords
    #[token("graph", ignore(case))]
    Graph,

    #[token("flowchart", ignore(case))]
    Flowchart,

    #[token("subgraph", ignore(case))]
    Subgraph,

    #[token("end", ignore(case))]
    End,

    #[token("direction", ignore(case))]
    Direction,

    #[token("style", ignore(case))]
    Style,

    #[token("classDef", ignore(case))]
    ClassDef,

    #[token("class", ignore(case))]
    Class,

    #[token("click", ignore(case))]
    Click,

    #[token("linkStyle", ignore(case))]
    LinkStyle,

    // Direction keywords
    #[regex(r"(?i)TB|TD|BT|LR|RL")]
    DirectionValue,

    // Arrow types (order matters - longer patterns first)
    #[token("-->")]
    Arrow,

    #[token("---")]
    Line,

    #[token("-.-")]
    DottedLine,

    #[token("-.->")]
    DottedArrow,

    #[token("==>")]
    ThickArrow,

    #[token("===")]
    ThickLine,

    #[token("~~~")]
    Invisible,

    #[token("--")]
    DoubleDash,

    #[token("-.")]
    DashDot,

    #[token("==")]
    DoubleEqual,

    // Node shapes (delimiters)
    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("((")]
    LDoubleParen,

    #[token("))")]
    RDoubleParen,

    #[token("([")]
    LParenBracket,

    #[token("])")]
    RBracketParen,

    #[token("[[")]
    LDoubleBracket,

    #[token("]]")]
    RDoubleBracket,

    #[token("[(")]
    LBracketParen,

    #[token(")]")]
    RParenBracket,

    #[token("{{")]
    LDoubleBrace,

    #[token("}}")]
    RDoubleBrace,

    #[token("[/")]
    LSlashBracket,

    #[token("/]")]
    RSlashBracket,

    #[token("[\\")]
    LBackslashBracket,

    #[token("\\]")]
    RBackslashBracket,

    #[token(">")]
    GreaterThan,

    // Other tokens
    #[token("|")]
    Pipe,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token("&")]
    Ampersand,

    #[token("\n")]
    Newline,

    // Quoted strings
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleQuotedString,

    #[regex(r#"'([^'\\]|\\.)*'"#)]
    SingleQuotedString,

    // Backtick strings (for special characters)
    #[regex(r#"`([^`])*`"#)]
    BacktickString,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Numbers
    #[regex(r"[0-9]+(\.[0-9]+)?")]
    Number,

    // Text (for labels, etc.) - lower priority so other patterns match first
    // Note: Excludes spaces so identifiers can be matched separately
    #[regex(r#"[^\[\](){}<>|:;\n\-=~&,/\\"'` \t]+"#, priority = 1)]
    Text,
}

/// A positioned token.
#[derive(Debug, Clone)]
pub struct PositionedToken {
    pub kind: FlowToken,
    pub span: Span,
    pub text: String,
}

/// Tokenize flowchart source code.
pub fn tokenize(source: &str) -> Vec<PositionedToken> {
    let mut tokens = Vec::new();
    let mut lexer = FlowToken::lexer(source);

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
        // Skip invalid tokens
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_graph_declaration() {
        let tokens = tokenize("graph TD");

        assert!(tokens.iter().any(|t| t.kind == FlowToken::Graph));
        assert!(tokens.iter().any(|t| t.kind == FlowToken::DirectionValue));
    }

    #[test]
    fn test_tokenize_flowchart_declaration() {
        let tokens = tokenize("flowchart LR");

        assert!(tokens.iter().any(|t| t.kind == FlowToken::Flowchart));
        assert!(tokens.iter().any(|t| t.kind == FlowToken::DirectionValue));
    }

    #[test]
    fn test_tokenize_node() {
        let tokens = tokenize("A[Label]");

        assert!(tokens.iter().any(|t| t.kind == FlowToken::Identifier));
        assert!(tokens.iter().any(|t| t.kind == FlowToken::LBracket));
        assert!(tokens.iter().any(|t| t.kind == FlowToken::RBracket));
    }

    #[test]
    fn test_tokenize_arrow() {
        let tokens = tokenize("A --> B");

        assert_eq!(tokens.iter().filter(|t| t.kind == FlowToken::Identifier).count(), 2);
        assert!(tokens.iter().any(|t| t.kind == FlowToken::Arrow));
    }

    #[test]
    fn test_tokenize_subgraph() {
        let tokens = tokenize("subgraph title\n    A --> B\nend");

        assert!(tokens.iter().any(|t| t.kind == FlowToken::Subgraph));
        assert!(tokens.iter().any(|t| t.kind == FlowToken::End));
    }

    #[test]
    fn test_tokenize_quoted_string() {
        let tokens = tokenize(r#"A["Hello World"]"#);

        assert!(tokens.iter().any(|t| t.kind == FlowToken::DoubleQuotedString));
    }

    #[test]
    fn test_tokenize_edge_label() {
        let tokens = tokenize("A -->|label| B");

        assert!(tokens.iter().any(|t| t.kind == FlowToken::Arrow));
        assert!(tokens.iter().filter(|t| t.kind == FlowToken::Pipe).count() >= 1);
    }
}
