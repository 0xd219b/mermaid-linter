//! Lexer for Pie charts.

use logos::Logos;

/// Tokens for Pie chart lexing.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum PieToken {
    // Keywords
    #[token("pie", ignore(case))]
    Pie,

    #[token("showData", ignore(case))]
    ShowData,

    #[token("title", ignore(case))]
    Title,

    #[token("accTitle", ignore(case))]
    AccTitle,

    #[token("accDescr", ignore(case))]
    AccDescr,

    // Delimiters
    #[token(":")]
    Colon,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    // Numbers (for slice values)
    #[regex(r"[0-9]+(\.[0-9]+)?", priority = 2)]
    Number,

    // Quoted strings (for slice labels)
    #[regex(r#""[^"]*""#)]
    QuotedString,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", priority = 2)]
    Identifier,

    // Newline
    #[regex(r"\n|\r\n")]
    Newline,
}

/// A token with its span information.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: PieToken,
    pub text: String,
    pub span: std::ops::Range<usize>,
}

/// Tokenize Pie chart source.
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = PieToken::lexer(source);

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
        let input = "pie";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == PieToken::Pie));
    }

    #[test]
    fn test_tokenize_show_data() {
        let input = "pie showData";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == PieToken::ShowData));
    }

    #[test]
    fn test_tokenize_slice() {
        let input = r#""Calcium" : 42.96"#;
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == PieToken::QuotedString));
        assert!(tokens.iter().any(|t| t.kind == PieToken::Colon));
        assert!(tokens.iter().any(|t| t.kind == PieToken::Number));
    }
}
