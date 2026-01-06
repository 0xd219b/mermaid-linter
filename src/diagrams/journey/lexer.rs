//! Lexer for User Journey diagrams.

use logos::Logos;

/// Tokens for Journey diagram lexing.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum JourneyToken {
    // Keywords
    #[token("journey", ignore(case))]
    Journey,

    #[token("title", ignore(case))]
    Title,

    #[token("section", ignore(case))]
    Section,

    #[token("accTitle", ignore(case))]
    AccTitle,

    #[token("accDescr", ignore(case))]
    AccDescr,

    // Delimiters
    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    // Numbers (scores 1-5)
    #[regex(r"[0-9]+", priority = 2)]
    Number,

    // Identifiers (actor names, etc.)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", priority = 2)]
    Identifier,

    // Quoted strings
    #[regex(r#""[^"]*""#)]
    QuotedString,

    // Newline
    #[regex(r"\n|\r\n")]
    Newline,
}

/// A token with its span information.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: JourneyToken,
    pub text: String,
    pub span: std::ops::Range<usize>,
}

/// Tokenize Journey diagram source.
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = JourneyToken::lexer(source);

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
        let input = "journey";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == JourneyToken::Journey));
    }

    #[test]
    fn test_tokenize_title() {
        let input = "title My Journey";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == JourneyToken::Title));
    }

    #[test]
    fn test_tokenize_section() {
        let input = "section Getting Started";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == JourneyToken::Section));
    }

    #[test]
    fn test_tokenize_task() {
        let input = "Make tea: 5: Me";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == JourneyToken::Colon));
        assert!(tokens.iter().any(|t| t.kind == JourneyToken::Number));
        assert!(tokens.iter().any(|t| t.kind == JourneyToken::Identifier));
    }

    #[test]
    fn test_tokenize_multiple_actors() {
        let input = "Do work: 3: Me, Cat, Dog";
        let tokens = tokenize(input);
        assert!(tokens.iter().filter(|t| t.kind == JourneyToken::Comma).count() >= 2);
    }
}
