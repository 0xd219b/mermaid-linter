//! Integration tests for Pie charts.

use mermaid_linter::{parse, DiagramType};

#[test]
fn test_simple_pie() {
    let code = r#"pie
    title Pet Types
    "Dogs" : 386
    "Cats" : 85"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple pie: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Pie));
    assert!(result.ast.is_some());
}

#[test]
fn test_pie_showdata() {
    let code = r#"pie showData
    title Distribution
    "A" : 30
    "B" : 70"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse showData: {:?}", result.diagnostics);
}

#[test]
fn test_pie_multiple_slices() {
    let code = r#"pie
    "Category A" : 42.96
    "Category B" : 50.05
    "Category C" : 10.01
    "Category D" : 25.50"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse multiple slices: {:?}", result.diagnostics);
}

#[test]
fn test_pie_decimal_values() {
    let code = r#"pie
    title Percentages
    "First" : 33.33
    "Second" : 33.33
    "Third" : 33.34"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse decimal values: {:?}", result.diagnostics);
}

#[test]
fn test_pie_with_title() {
    let code = r#"pie
    title My Pie Chart Title"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with title: {:?}", result.diagnostics);
}

#[test]
fn test_pie_accessibility() {
    let code = r#"pie
    accTitle: Pie Chart Title
    accDescr: This pie chart shows the distribution
    title Distribution
    "A" : 50
    "B" : 50"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse accessibility: {:?}", result.diagnostics);
}

#[test]
fn test_pie_case_insensitive() {
    let code = r#"PIE SHOWDATA
    TITLE Distribution
    "A" : 50
    "B" : 50"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse case-insensitive: {:?}", result.diagnostics);
}

#[test]
fn test_pie_invalid() {
    let code = "not a pie chart";

    let result = parse(code, None);
    assert!(result.diagram_type != Some(DiagramType::Pie) || !result.ok);
}
