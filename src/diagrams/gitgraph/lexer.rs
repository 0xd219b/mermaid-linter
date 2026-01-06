//! Lexer for GitGraph diagrams.

use logos::Logos;

/// Tokens for GitGraph lexing.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum GitGraphToken {
    // Keywords
    #[token("gitGraph", ignore(case))]
    GitGraph,

    #[token("commit", ignore(case))]
    Commit,

    #[token("branch", ignore(case))]
    Branch,

    #[token("checkout", ignore(case))]
    Checkout,

    #[token("merge", ignore(case))]
    Merge,

    #[token("cherry-pick", ignore(case))]
    CherryPick,

    #[token("id", ignore(case))]
    Id,

    #[token("msg", ignore(case))]
    Msg,

    #[token("tag", ignore(case))]
    Tag,

    #[token("type", ignore(case))]
    Type,

    #[token("order", ignore(case))]
    Order,

    #[token("LR")]
    LR,

    #[token("TB")]
    #[token("BT")]
    TB,

    #[token("accTitle", ignore(case))]
    AccTitle,

    #[token("accDescr", ignore(case))]
    AccDescr,

    // Commit types
    #[token("NORMAL", ignore(case))]
    Normal,

    #[token("REVERSE", ignore(case))]
    Reverse,

    #[token("HIGHLIGHT", ignore(case))]
    Highlight,

    // Delimiters
    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    // Quoted strings
    #[regex(r#""[^"]*""#)]
    QuotedString,

    // Identifiers (branch names, commit IDs, etc.)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", priority = 2)]
    Identifier,

    // Numbers
    #[regex(r"[0-9]+", priority = 1)]
    Number,

    // Newline
    #[regex(r"\n|\r\n")]
    Newline,
}

/// A token with its span information.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: GitGraphToken,
    pub text: String,
    pub span: std::ops::Range<usize>,
}

/// Tokenize GitGraph source.
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = GitGraphToken::lexer(source);

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
        let input = "gitGraph";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GitGraphToken::GitGraph));
    }

    #[test]
    fn test_tokenize_commit() {
        let input = "commit";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GitGraphToken::Commit));
    }

    #[test]
    fn test_tokenize_branch() {
        let input = "branch develop";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GitGraphToken::Branch));
        assert!(tokens.iter().any(|t| t.kind == GitGraphToken::Identifier));
    }

    #[test]
    fn test_tokenize_merge() {
        let input = "merge main";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GitGraphToken::Merge));
    }
}
