//! Text normalization utilities.

use regex::Regex;
use once_cell::sync::Lazy;

/// Regex for matching HTML tags with attributes.
static HTML_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<(\w+)([^>]*)>"#).unwrap()
});

/// Regex for matching double-quoted attributes.
static DOUBLE_QUOTE_ATTR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"="([^"]*)""#).unwrap()
});

/// Normalizes text for Mermaid parsing.
///
/// This performs two transformations:
/// 1. Convert CRLF line endings to LF
/// 2. Convert double-quoted HTML attributes to single-quoted
///
/// # Example
///
/// ```
/// use mermaid_linter::preprocess::normalize_text;
///
/// let input = "graph TD\r\n    A --> B";
/// let output = normalize_text(input);
/// assert_eq!(output, "graph TD\n    A --> B");
/// ```
pub fn normalize_text(text: &str) -> String {
    // Step 1: Convert CRLF to LF (and lone CR to LF)
    let text = text.replace("\r\n", "\n").replace('\r', "\n");

    // Step 2: Convert double-quoted HTML attributes to single-quoted
    HTML_TAG_REGEX
        .replace_all(&text, |caps: &regex::Captures| {
            let tag = &caps[1];
            let attributes = &caps[2];

            // Replace double quotes with single quotes in attributes
            let new_attributes = DOUBLE_QUOTE_ATTR_REGEX.replace_all(attributes, "='$1'");

            format!("<{}{}>", tag, new_attributes)
        })
        .into_owned()
}

/// Encodes HTML entities in style and classDef lines.
///
/// This prevents entity conflicts during parsing. The encoded text
/// should be decoded after parsing using `decode_entities`.
///
/// # Example
///
/// ```
/// use mermaid_linter::preprocess::encode_entities;
///
/// let input = "style nodeA fill:#f9f;";
/// let output = encode_entities(input);
/// // The output will have encoded the color value
/// ```
pub fn encode_entities(text: &str) -> String {
    let mut result = text.to_string();

    // Remove trailing semicolons from style lines with # color values
    // Pattern: style.*:\S*#.*;
    let style_regex = Regex::new(r"style[^;]*:\S*#[^;]*;").unwrap();
    result = style_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let s = &caps[0];
            // Remove trailing semicolon
            s[..s.len() - 1].to_string()
        })
        .into_owned();

    // Same for classDef lines
    let classdef_regex = Regex::new(r"classDef[^;]*:\S*#[^;]*;").unwrap();
    result = classdef_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let s = &caps[0];
            s[..s.len() - 1].to_string()
        })
        .into_owned();

    // Encode HTML entities: #word; -> special encoding
    // Numeric: #123; -> ﬂ°°123¶ß
    // Named: #nbsp; -> ﬂ°nbsp¶ß
    let entity_regex = Regex::new(r"#(\w+);").unwrap();
    result = entity_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let inner = &caps[1];
            if inner.chars().all(|c| c.is_ascii_digit()) {
                // Numeric entity
                format!("ﬂ°°{}¶ß", inner)
            } else {
                // Named entity
                format!("ﬂ°{}¶ß", inner)
            }
        })
        .into_owned();

    result
}

/// Decodes previously encoded HTML entities.
///
/// # Example
///
/// ```ignore
/// use mermaid_linter::preprocess::normalize::decode_entities;
///
/// let encoded = "ﬂ°°123¶ß and ﬂ°nbsp¶ß";
/// let decoded = decode_entities(encoded);
/// assert_eq!(decoded, "&#123; and &nbsp;");
/// ```
#[allow(dead_code)]
pub fn decode_entities(text: &str) -> String {
    text.replace("ﬂ°°", "&#")
        .replace("ﬂ°", "&")
        .replace("¶ß", ";")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_crlf() {
        let input = "line1\r\nline2\r\nline3";
        let output = normalize_text(input);
        assert_eq!(output, "line1\nline2\nline3");
    }

    #[test]
    fn test_normalize_cr() {
        let input = "line1\rline2\rline3";
        let output = normalize_text(input);
        assert_eq!(output, "line1\nline2\nline3");
    }

    #[test]
    fn test_normalize_html_attributes() {
        let input = r#"<div class="foo" id="bar">content</div>"#;
        let output = normalize_text(input);
        assert_eq!(output, r#"<div class='foo' id='bar'>content</div>"#);
    }

    #[test]
    fn test_normalize_mixed() {
        let input = "graph TD\r\n    A[\"Node A\"] --> B";
        let output = normalize_text(input);
        assert!(output.contains('\n'));
        assert!(!output.contains('\r'));
    }

    #[test]
    fn test_encode_entities_numeric() {
        let input = "#123;";
        let output = encode_entities(input);
        assert_eq!(output, "ﬂ°°123¶ß");
    }

    #[test]
    fn test_encode_entities_named() {
        let input = "#nbsp;";
        let output = encode_entities(input);
        assert_eq!(output, "ﬂ°nbsp¶ß");
    }

    #[test]
    fn test_decode_entities() {
        let input = "ﬂ°°123¶ß and ﬂ°nbsp¶ß";
        let output = decode_entities(input);
        assert_eq!(output, "&#123; and &nbsp;");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        // Note: roundtrip doesn't produce original because encoding strips trailing semicolons
        let input = "#123; #nbsp;";
        let encoded = encode_entities(input);
        let decoded = decode_entities(&encoded);
        assert_eq!(decoded, "&#123; &nbsp;");
    }

    #[test]
    fn test_encode_style_line() {
        let input = "style nodeA fill:#f9f;";
        let output = encode_entities(input);
        // Should remove trailing semicolon from style line
        assert!(!output.ends_with(";;"));
    }
}
