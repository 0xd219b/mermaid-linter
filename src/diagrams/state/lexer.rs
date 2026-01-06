//! State diagram lexer.

use logos::Logos;

use crate::ast::Span;

/// Token types for state diagram parsing.
#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
pub enum StateToken {
    // Keywords
    #[token("stateDiagram", ignore(case))]
    StateDiagram,

    #[token("stateDiagram-v2", ignore(case))]
    StateDiagramV2,

    #[token("state", ignore(case))]
    State,

    #[token("note", ignore(case))]
    Note,

    #[token("left of", ignore(case))]
    LeftOf,

    #[token("right of", ignore(case))]
    RightOf,

    #[token("end note", ignore(case))]
    EndNote,

    #[token("direction", ignore(case))]
    Direction,

    // Special states
    #[token("[*]")]
    StartEnd,

    #[token("<<fork>>", ignore(case))]
    Fork,

    #[token("<<join>>", ignore(case))]
    Join,

    #[token("<<choice>>", ignore(case))]
    Choice,

    // Transition arrow
    #[token("-->")]
    Arrow,

    // Other tokens
    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(":")]
    Colon,

    #[token("::")]
    DoubleColon,

    #[token("\n")]
    Newline,

    // Quoted strings
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleQuotedString,

    // Stereotypes
    #[regex(r"<<[^>]+>>")]
    Stereotype,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Numbers
    #[regex(r"[0-9]+")]
    Number,

    // Text - lower priority so other patterns match first
    // Note: Excludes spaces so identifiers can be matched separately
    #[regex(r#"[^\n{}:\[\]"<> \t]+"#, priority = 1)]
    Text,
}

/// A positioned token.
#[derive(Debug, Clone)]
pub struct PositionedToken {
    pub kind: StateToken,
    pub span: Span,
    pub text: String,
}

/// Tokenize state diagram source code.
pub fn tokenize(source: &str) -> Vec<PositionedToken> {
    let mut tokens = Vec::new();
    let mut lexer = StateToken::lexer(source);

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
        let input = "stateDiagram-v2";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == StateToken::StateDiagramV2));
    }

    #[test]
    fn test_tokenize_transition() {
        let input = ["[*]", "-->", "State1"].join(" ");
        let tokens = tokenize(&input);
        assert!(tokens.iter().any(|t| t.kind == StateToken::StartEnd));
        assert!(tokens.iter().any(|t| t.kind == StateToken::Arrow));
        assert!(tokens.iter().any(|t| t.kind == StateToken::Identifier));
    }

    #[test]
    fn test_tokenize_composite_state() {
        let input = r#"state Composite {
    [*] --> Inner
}"#;
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == StateToken::State));
        assert!(tokens.iter().any(|t| t.kind == StateToken::LBrace));
        assert!(tokens.iter().any(|t| t.kind == StateToken::RBrace));
    }

    #[test]
    fn test_tokenize_note() {
        let input = r#"note right of State1
    Text
end note"#;
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == StateToken::Note));
        assert!(tokens.iter().any(|t| t.kind == StateToken::RightOf));
        assert!(tokens.iter().any(|t| t.kind == StateToken::EndNote));
    }
}
