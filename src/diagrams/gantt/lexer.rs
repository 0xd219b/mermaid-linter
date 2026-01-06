//! Lexer for Gantt charts.

use logos::Logos;

/// Tokens for Gantt chart lexing.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum GanttToken {
    // Keywords
    #[token("gantt", ignore(case))]
    Gantt,

    #[token("title", ignore(case))]
    Title,

    #[token("dateFormat", ignore(case))]
    DateFormat,

    #[token("axisFormat", ignore(case))]
    AxisFormat,

    #[token("tickInterval", ignore(case))]
    TickInterval,

    #[token("excludes", ignore(case))]
    Excludes,

    #[token("includes", ignore(case))]
    Includes,

    #[token("todayMarker", ignore(case))]
    TodayMarker,

    #[token("weekday", ignore(case))]
    Weekday,

    #[token("section", ignore(case))]
    Section,

    #[token("accTitle", ignore(case))]
    AccTitle,

    #[token("accDescr", ignore(case))]
    AccDescr,

    // Task modifiers
    #[token("done", ignore(case))]
    Done,

    #[token("active", ignore(case))]
    Active,

    #[token("crit", ignore(case))]
    Crit,

    #[token("milestone", ignore(case))]
    Milestone,

    #[token("after", ignore(case))]
    After,

    #[token("until", ignore(case))]
    Until,

    // Day names (for excludes/includes)
    #[token("monday", ignore(case))]
    #[token("tuesday", ignore(case))]
    #[token("wednesday", ignore(case))]
    #[token("thursday", ignore(case))]
    #[token("friday", ignore(case))]
    #[token("saturday", ignore(case))]
    #[token("sunday", ignore(case))]
    DayName,

    // Delimiters
    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    // Duration patterns (e.g., 30d, 2w, 1M)
    #[regex(r"[0-9]+[dwmyhMs]", priority = 3)]
    Duration,

    // Date patterns (ISO format)
    #[regex(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", priority = 3)]
    Date,

    // Time patterns
    #[regex(r"[0-9]{2}:[0-9]{2}(:[0-9]{2})?", priority = 3)]
    Time,

    // Identifiers (task IDs, etc.)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", priority = 2)]
    Identifier,

    // Numbers
    #[regex(r"[0-9]+", priority = 1)]
    Number,

    // Quoted strings
    #[regex(r#""[^"]*""#)]
    QuotedString,

    // Newline
    #[regex(r"\n|\r\n")]
    Newline,

    // Percent format patterns (e.g., %m/%d, %Y-%m-%d)
    #[regex(r"%[a-zA-Z]", priority = 2)]
    FormatSpec,

    // Slash (for format separators)
    #[token("/")]
    Slash,

    // Dash (for format separators, used when not part of identifier)
    #[token("-")]
    Dash,
}

/// A token with its span information.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: GanttToken,
    pub text: String,
    pub span: std::ops::Range<usize>,
}

/// Tokenize Gantt chart source.
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = GanttToken::lexer(source);

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
        let input = "gantt";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Gantt));
    }

    #[test]
    fn test_tokenize_title() {
        let input = "title My Gantt Chart";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Title));
    }

    #[test]
    fn test_tokenize_date_format() {
        let input = "dateFormat YYYY-MM-DD";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GanttToken::DateFormat));
    }

    #[test]
    fn test_tokenize_section() {
        let input = "section Project Phase";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Section));
    }

    #[test]
    fn test_tokenize_task() {
        let input = "Task name :a1, 2024-01-01, 30d";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Colon));
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Comma));
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Date));
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Duration));
    }

    #[test]
    fn test_tokenize_modifiers() {
        let input = "done crit active milestone";
        let tokens = tokenize(input);
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Done));
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Crit));
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Active));
        assert!(tokens.iter().any(|t| t.kind == GanttToken::Milestone));
    }
}
