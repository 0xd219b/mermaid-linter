//! Comment removal from Mermaid diagrams.

/// Removes comment lines from text.
///
/// Comments in Mermaid are lines starting with `%%` (but not `%%{` which are directives).
///
/// # Example
///
/// ```
/// use mermaid_linter::preprocess::remove_comments;
///
/// let text = r#"graph TD
///     %% This is a comment
///     A --> B
///     %% Another comment
///     B --> C
/// "#;
///
/// let result = remove_comments(text);
/// assert!(!result.contains("This is a comment"));
/// assert!(result.contains("A --> B"));
/// ```
pub fn remove_comments(text: &str) -> String {
    let mut result = String::new();
    let mut first_non_comment = true;

    for line in text.lines() {
        let trimmed = line.trim_start();

        // Check if line is a comment (starts with %% but not %%{)
        let is_comment = trimmed.starts_with("%%") && !trimmed.starts_with("%%{");

        if !is_comment {
            if first_non_comment {
                result.push_str(line);
                first_non_comment = false;
            } else {
                result.push('\n');
                result.push_str(line);
            }
        }
    }

    // Handle trailing newline from original text
    if text.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_single_comment() {
        let text = "%% This is a comment\ngraph TD\n    A --> B";
        let result = remove_comments(text);

        assert!(!result.contains("comment"));
        assert!(result.starts_with("graph TD"));
    }

    #[test]
    fn test_remove_multiple_comments() {
        let text = r#"%% Comment 1
graph TD
    %% Comment 2
    A --> B
    %% Comment 3
    B --> C
"#;
        let result = remove_comments(text);

        assert!(!result.contains("Comment"));
        assert!(result.contains("graph TD"));
        assert!(result.contains("A --> B"));
    }

    #[test]
    fn test_preserve_directives() {
        let text = r#"%%{init: {"theme": "dark"}}%%
%% This is a comment
graph TD
    A --> B
"#;
        let result = remove_comments(text);

        // Directives should be preserved
        assert!(result.contains("%%{init"));
        // Comments should be removed
        assert!(!result.contains("This is a comment"));
    }

    #[test]
    fn test_inline_comment_marker() {
        // %% in the middle of a line is not a comment
        let text = "graph TD\n    A[\"%%test%%\"] --> B";
        let result = remove_comments(text);

        // Should preserve the text since %% is not at line start
        assert!(result.contains("%%test%%"));
    }

    #[test]
    fn test_comment_with_indentation() {
        let text = "graph TD\n    %% Indented comment\n    A --> B";
        let result = remove_comments(text);

        assert!(!result.contains("Indented comment"));
        assert!(result.contains("A --> B"));
    }

    #[test]
    fn test_empty_comment() {
        let text = "%%\ngraph TD\n    A --> B";
        let result = remove_comments(text);

        assert!(result.starts_with("graph TD"));
    }

    #[test]
    fn test_no_comments() {
        let text = "graph TD\n    A --> B\n    B --> C";
        let result = remove_comments(text);

        assert_eq!(result, text);
    }
}
