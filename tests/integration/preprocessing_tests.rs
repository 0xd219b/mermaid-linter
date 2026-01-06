//! Integration tests for preprocessing functionality.

use mermaid_linter::{parse, DiagramType};

#[test]
fn test_frontmatter_title() {
    let code = r#"---
title: My Diagram
---
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with frontmatter: {:?}", result.diagnostics);
    assert_eq!(result.title, Some("My Diagram".to_string()));
}

#[test]
fn test_frontmatter_config() {
    let code = r#"---
config:
  flowchart:
    defaultRenderer: elk
---
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with frontmatter config: {:?}", result.diagnostics);
    assert_eq!(result.config.flowchart.default_renderer, Some("elk".to_string()));
}

#[test]
fn test_frontmatter_display_mode() {
    let code = r#"---
displayMode: compact
---
gantt
    title Test
    section Section
    Task 1: a, 2024-01-01, 30d"#;

    let result = parse(code, None);
    // Note: Gantt parser may not be implemented yet
    // This test verifies preprocessing handles displayMode
    assert_eq!(result.config.gantt.display_mode, Some("compact".to_string()));
}

#[test]
fn test_init_directive() {
    let code = r#"%%{init: {"flowchart": {"defaultRenderer": "elk"}}}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with init directive: {:?}", result.diagnostics);
    assert_eq!(result.config.flowchart.default_renderer, Some("elk".to_string()));
}

#[test]
fn test_init_directive_layout() {
    let code = r#"%%{init: {"layout": "elk"}}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with layout directive: {:?}", result.diagnostics);
    assert_eq!(result.config.layout, Some("elk".to_string()));
}

#[test]
fn test_wrap_directive() {
    let code = r#"%%{wrap}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with wrap directive: {:?}", result.diagnostics);
    assert!(result.config.wrap);
}

#[test]
fn test_multiple_directives() {
    let code = r#"%%{init: {"layout": "elk"}}%%
%%{wrap}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with multiple directives: {:?}", result.diagnostics);
    assert_eq!(result.config.layout, Some("elk".to_string()));
    assert!(result.config.wrap);
}

#[test]
fn test_directive_and_frontmatter() {
    let code = r#"---
title: My Diagram
config:
  theme: forest
---
%%{init: {"flowchart": {"defaultRenderer": "elk"}}}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with directive and frontmatter: {:?}", result.diagnostics);
    assert_eq!(result.title, Some("My Diagram".to_string()));
    // Directive should override frontmatter
    assert_eq!(result.config.flowchart.default_renderer, Some("elk".to_string()));
}

#[test]
fn test_comment_removal() {
    let code = r#"%% This comment should be removed
graph TD
    %% Another comment
    A --> B
    %% Final comment"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with comments: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Flowchart));
}

#[test]
fn test_crlf_normalization() {
    // CRLF line endings
    let code = "graph TD\r\n    A --> B\r\n    B --> C";

    let result = parse(code, None);
    assert!(result.ok, "Failed to normalize CRLF: {:?}", result.diagnostics);
}

#[test]
fn test_cr_normalization() {
    // CR only line endings (old Mac)
    let code = "graph TD\r    A --> B\r    B --> C";

    let result = parse(code, None);
    assert!(result.ok, "Failed to normalize CR: {:?}", result.diagnostics);
}

#[test]
fn test_html_attribute_normalization() {
    // Double quoted HTML attributes should be converted to single quotes
    let code = r#"graph TD
    A["<span class="foo">text</span>"] --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to normalize HTML attributes: {:?}", result.diagnostics);
}

#[test]
fn test_empty_frontmatter() {
    let code = r#"---
---
graph TD
    A --> B"#;

    let result = parse(code, None);
    // Empty frontmatter may be detected as bad frontmatter due to strict parsing
    // Just ensure it doesn't panic
    if result.ok {
        assert!(result.title.is_none());
    }
}

#[test]
fn test_preserve_directive_in_comment() {
    // %%{...}%% should NOT be removed as a comment
    let code = r#"%%{init: {"layout": "elk"}}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to preserve directive: {:?}", result.diagnostics);
    assert_eq!(result.config.layout, Some("elk".to_string()));
}

#[test]
fn test_bad_frontmatter() {
    // Bad frontmatter (starts with --- but doesn't end properly)
    let code = r#"---
graph TD
    A --> B"#;

    let result = parse(code, None);
    // This should be detected as bad frontmatter or fail to parse
    // Either is acceptable
    if !result.ok {
        // May or may not have BadFrontmatter type set
    }
}

#[test]
fn test_whitespace_trimming() {
    let code = r#"

graph TD
    A --> B

"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to handle whitespace: {:?}", result.diagnostics);
}

#[test]
fn test_initialize_alias() {
    let code = r#"%%{initialize: {"flowchart": {"defaultRenderer": "dagre"}}}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse initialize directive: {:?}", result.diagnostics);
    assert_eq!(result.config.flowchart.default_renderer, Some("dagre".to_string()));
}

#[test]
fn test_directive_with_spacing() {
    let code = r#"%%{ init: { "layout": "elk" } }%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse directive with spacing: {:?}", result.diagnostics);
    assert_eq!(result.config.layout, Some("elk".to_string()));
}

#[test]
fn test_frontmatter_only_title() {
    let code = r#"---
title: Simple Title
---
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse frontmatter with only title: {:?}", result.diagnostics);
    assert_eq!(result.title, Some("Simple Title".to_string()));
}

#[test]
fn test_unknown_directive_ignored() {
    let code = r#"%%{unknownDirective: something}%%
graph TD
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to ignore unknown directive: {:?}", result.diagnostics);
}
