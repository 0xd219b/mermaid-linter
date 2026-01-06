//! Lexer for ER diagrams.

use logos::Logos;

/// Tokens for ER diagram lexing.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum ErToken {
    // Keywords
    #[token("erDiagram", ignore(case))]
    ErDiagram,

    #[token("direction", ignore(case))]
    Direction,

    #[token("TB", ignore(case))]
    #[token("BT", ignore(case))]
    #[token("LR", ignore(case))]
    #[token("RL", ignore(case))]
    DirectionValue,

    #[token("style", ignore(case))]
    Style,

    #[token("classDef", ignore(case))]
    ClassDef,

    #[token("class", ignore(case))]
    Class,

    #[token("accTitle", ignore(case))]
    AccTitle,

    #[token("accDescr", ignore(case))]
    AccDescr,

    // Attribute keys
    #[token("PK", ignore(case))]
    PrimaryKey,

    #[token("FK", ignore(case))]
    ForeignKey,

    #[token("UK", ignore(case))]
    UniqueKey,

    // Cardinality markers - left side
    #[token("||")]
    OnlyOneLeft,

    #[token("|o")]
    ZeroOrOneLeft,

    #[token("o|")]
    ZeroOrOneRight,

    #[token("}|")]
    OneOrMoreLeft,

    #[token("|{")]
    OneOrMoreRight,

    #[token("}o")]
    ZeroOrMoreLeft,

    #[token("o{")]
    ZeroOrMoreRight,

    // Relationship line types
    #[token("--")]
    Identifying,

    #[token("..")]
    NonIdentifying,

    // Alternative cardinality keywords
    #[token("to", ignore(case))]
    To,

    #[token("optionally", ignore(case))]
    Optionally,

    #[token("only", ignore(case))]
    Only,

    #[token("one", ignore(case))]
    One,

    #[token("zero", ignore(case))]
    Zero,

    #[token("or", ignore(case))]
    Or,

    #[token("more", ignore(case))]
    More,

    #[token("many", ignore(case))]
    Many,

    // Delimiters
    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("[")]
    OpenBracket,

    #[token("]")]
    CloseBracket,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token(":::")]
    TripleColon,

    #[token("~")]
    Tilde,

    // Quoted strings
    #[regex(r#""[^"]*""#)]
    QuotedString,

    // Entity/attribute names - alphanumeric with hyphens and underscores
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_\-]*", priority = 2)]
    Identifier,

    // Numbers
    #[regex(r"[0-9]+")]
    Number,

    // Generic type notation
    #[regex(r"~[^~]+~")]
    GenericType,

    // Comment text (after keys)
    #[regex(r#""[^"]*""#, priority = 2)]
    CommentText,

    // Newline
    #[regex(r"\n|\r\n")]
    Newline,

    // Semicolon (statement separator)
    #[token(";")]
    Semicolon,
}

impl ErToken {
    /// Check if this token is a cardinality marker.
    pub fn is_cardinality(&self) -> bool {
        matches!(
            self,
            ErToken::OnlyOneLeft
                | ErToken::ZeroOrOneLeft
                | ErToken::ZeroOrOneRight
                | ErToken::OneOrMoreLeft
                | ErToken::OneOrMoreRight
                | ErToken::ZeroOrMoreLeft
                | ErToken::ZeroOrMoreRight
        )
    }

    /// Check if this token is a relationship line type.
    pub fn is_relationship_line(&self) -> bool {
        matches!(self, ErToken::Identifying | ErToken::NonIdentifying)
    }
}

/// A token with its span information.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: ErToken,
    pub text: String,
    pub span: std::ops::Range<usize>,
}

/// Tokenize ER diagram source.
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = ErToken::lexer(source);

    while let Some(result) = lexer.next() {
        if let Ok(kind) = result {
            tokens.push(Token {
                kind,
                text: lexer.slice().to_string(),
                span: lexer.span(),
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
        let input = "erDiagram";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == ErToken::ErDiagram));
    }

    #[test]
    fn test_tokenize_entity() {
        let input = "CUSTOMER";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == ErToken::Identifier));
    }

    #[test]
    fn test_tokenize_relationship() {
        let input = "||--o{";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == ErToken::OnlyOneLeft));
        assert!(tokens.iter().any(|t| t.kind == ErToken::Identifying));
        assert!(tokens.iter().any(|t| t.kind == ErToken::ZeroOrMoreRight));
    }

    #[test]
    fn test_tokenize_attribute_keys() {
        let input = "PK FK UK";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == ErToken::PrimaryKey));
        assert!(tokens.iter().any(|t| t.kind == ErToken::ForeignKey));
        assert!(tokens.iter().any(|t| t.kind == ErToken::UniqueKey));
    }

    #[test]
    fn test_tokenize_quoted_string() {
        let input = r#""Customer Name""#;
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == ErToken::QuotedString));
    }

    #[test]
    fn test_tokenize_full_relationship() {
        let input = "CUSTOMER ||--o{ ORDER : places";
        let tokens = tokenize(input);
        assert!(tokens.len() >= 5);
    }

    #[test]
    fn test_tokenize_attributes() {
        let input = r#"{
            string name
            int id PK
        }"#;
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == ErToken::OpenBrace));
        assert!(tokens.iter().any(|t| t.kind == ErToken::CloseBrace));
        assert!(tokens.iter().any(|t| t.kind == ErToken::PrimaryKey));
    }
}
