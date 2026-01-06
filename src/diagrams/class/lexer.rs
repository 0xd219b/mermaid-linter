//! Class diagram lexer.

use logos::Logos;

use crate::ast::Span;

/// Token types for class diagram parsing.
#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
pub enum ClassToken {
    // Keywords
    #[token("classDiagram", ignore(case))]
    ClassDiagram,

    #[token("classDiagram-v2", ignore(case))]
    ClassDiagramV2,

    #[token("class", ignore(case))]
    Class,

    #[token("namespace", ignore(case))]
    Namespace,

    #[token("note", ignore(case))]
    Note,

    #[token("for", ignore(case))]
    For,

    #[token("link", ignore(case))]
    Link,

    #[token("callback", ignore(case))]
    Callback,

    #[token("click", ignore(case))]
    Click,

    #[token("cssClass", ignore(case))]
    CssClass,

    #[token("direction", ignore(case))]
    Direction,

    // Stereotypes
    #[regex(r"<<[^>]+>>")]
    Stereotype,

    // Relationship types (order matters - longer patterns first)
    #[token("<|--")]
    InheritanceLeft,

    #[token("--|>")]
    InheritanceRight,

    #[token("*--")]
    CompositionLeft,

    #[token("--*")]
    CompositionRight,

    #[token("o--")]
    AggregationLeft,

    #[token("--o")]
    AggregationRight,

    #[token("<..")]
    DependencyLeft,

    #[token("..>")]
    DependencyRight,

    #[token("<|..")]
    RealizationLeft,

    #[token("..|>")]
    RealizationRight,

    #[token("--")]
    Association,

    #[token("..")]
    DashedLine,

    // Visibility markers
    #[token("+")]
    Public,

    #[token("-")]
    Private,

    #[token("#")]
    Protected,

    #[token("~")]
    Package,

    // Other tokens
    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("*")]
    Star,

    #[token("$")]
    Dollar,

    #[token("\n")]
    Newline,

    // Quoted strings
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleQuotedString,

    // Identifiers (including generic types)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*(<[^>]+>)?")]
    Identifier,

    // Numbers
    #[regex(r"[0-9]+")]
    Number,

    // Cardinality patterns
    #[regex(r#""[0-9*n]+(\.\.[0-9*n]+)?""#)]
    Cardinality,

    // Text - lower priority so other patterns match first
    // Note: Excludes spaces so identifiers can be matched separately
    #[regex(r#"[^\n{}()\[\]:,+\-#~*$" \t]+"#, priority = 1)]
    Text,
}

/// A positioned token.
#[derive(Debug, Clone)]
pub struct PositionedToken {
    pub kind: ClassToken,
    pub span: Span,
    pub text: String,
}

/// Tokenize class diagram source code.
pub fn tokenize(source: &str) -> Vec<PositionedToken> {
    let mut tokens = Vec::new();
    let mut lexer = ClassToken::lexer(source);

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
        let tokens = tokenize("classDiagram");
        assert!(tokens.iter().any(|t| t.kind == ClassToken::ClassDiagram));
    }

    #[test]
    fn test_tokenize_class() {
        let input = ["class", "Animal"].join(" ");
        let tokens = tokenize(&input);
        assert!(tokens.iter().any(|t| t.kind == ClassToken::Class));
        assert!(tokens.iter().any(|t| t.kind == ClassToken::Identifier));
    }

    #[test]
    fn test_tokenize_inheritance() {
        let input = ["Animal", "<|--", "Dog"].join(" ");
        let tokens = tokenize(&input);
        assert!(tokens.iter().any(|t| t.kind == ClassToken::InheritanceLeft));
    }

    #[test]
    fn test_tokenize_member() {
        let input = ["+", "String", "name"].join(" ");
        let tokens = tokenize(&input);
        assert!(tokens.iter().any(|t| t.kind == ClassToken::Public));
        assert!(tokens.iter().filter(|t| t.kind == ClassToken::Identifier).count() >= 1);
    }

    #[test]
    fn test_tokenize_stereotype() {
        let input = r#"class Animal {
    <<interface>>
}"#;
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == ClassToken::Stereotype));
    }
}
