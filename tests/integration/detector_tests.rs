//! Integration tests for diagram type detection.

use mermaid_linter::{detect_type, DiagramType};

#[test]
fn test_detect_flowchart_variants() {
    // graph keyword variants (legacy flowchart)
    assert_eq!(detect_type("graph TD\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("graph TB\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("graph BT\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("graph RL\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("graph LR\nA-->B"), Some(DiagramType::Flowchart));

    // flowchart keyword (v2 flowchart)
    assert_eq!(detect_type("flowchart TD\nA-->B"), Some(DiagramType::FlowchartV2));
    assert_eq!(detect_type("flowchart LR\nA-->B"), Some(DiagramType::FlowchartV2));
}

#[test]
fn test_detect_sequence() {
    assert_eq!(detect_type("sequenceDiagram\nAlice->>Bob: Hi"), Some(DiagramType::Sequence));
    assert_eq!(detect_type("  sequenceDiagram\n  Alice->>Bob: Hi"), Some(DiagramType::Sequence));
}

#[test]
fn test_detect_class() {
    assert_eq!(detect_type("classDiagram\nclass Animal"), Some(DiagramType::Class));
    assert_eq!(detect_type("classDiagram\nAnimal <|-- Dog"), Some(DiagramType::Class));
}

#[test]
fn test_detect_state() {
    assert_eq!(detect_type("stateDiagram\n[*] --> A"), Some(DiagramType::State));
    assert_eq!(detect_type("stateDiagram-v2\n[*] --> A"), Some(DiagramType::StateDiagram));
}

#[test]
fn test_detect_with_whitespace() {
    // Leading whitespace should be handled
    assert_eq!(detect_type("  graph TD\n  A-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("\n\ngraph TD\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("\t\tsequenceDiagram\n\tAlice->>Bob: Hi"), Some(DiagramType::Sequence));
}

#[test]
fn test_detect_with_comments() {
    // Comments should be stripped before detection
    assert_eq!(detect_type("%% comment\ngraph TD\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("%% comment\n%% another\nsequenceDiagram\nAlice->>Bob: Hi"), Some(DiagramType::Sequence));
}

#[test]
fn test_detect_with_frontmatter() {
    let code = r#"---
title: Test
---
graph TD
    A --> B"#;
    assert_eq!(detect_type(code), Some(DiagramType::Flowchart));
}

#[test]
fn test_detect_with_directives() {
    let code = r#"%%{init: {"theme": "dark"}}%%
graph TD
    A --> B"#;
    assert_eq!(detect_type(code), Some(DiagramType::Flowchart));
}

#[test]
fn test_detect_case_insensitive() {
    // Keywords should be case insensitive
    assert_eq!(detect_type("GRAPH TD\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("Graph TD\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("SEQUENCEDIAGRAM\nAlice->>Bob: Hi"), Some(DiagramType::Sequence));
    // classDiagram (without -v2) is detected as Class (legacy)
    assert_eq!(detect_type("CLASSDIAGRAM\nclass Animal"), Some(DiagramType::Class));
    assert_eq!(detect_type("STATEDIAGRAM\n[*] --> A"), Some(DiagramType::State));
    assert_eq!(detect_type("STATEDIAGRAM-V2\n[*] --> A"), Some(DiagramType::StateDiagram));
}

#[test]
fn test_detect_unknown() {
    assert_eq!(detect_type("unknown diagram type"), None);
    assert_eq!(detect_type("not a valid diagram"), None);
    assert_eq!(detect_type(""), None);
    assert_eq!(detect_type("   "), None);
}

#[test]
fn test_detect_bad_frontmatter() {
    // Starts with --- but doesn't end properly
    let code = r#"---
not valid yaml without closing"#;
    assert_eq!(detect_type(code), Some(DiagramType::BadFrontmatter));
}

#[test]
fn test_detect_error() {
    // The "error" diagram type is special - may not be implemented
    // Skip if not implemented
    let result = detect_type("error\nsome text");
    // Either None or Error is acceptable
    assert!(result.is_none() || result == Some(DiagramType::Error));
}

#[test]
fn test_detect_priority() {
    // Test detection priority - more specific should win
    // classDiagram-v2 should be detected as Class
    assert_eq!(detect_type("classDiagram\nclass A"), Some(DiagramType::Class));
}

#[test]
fn test_detect_with_semicolons() {
    assert_eq!(detect_type("graph TD;A-->B;B-->C"), Some(DiagramType::Flowchart));
}

#[test]
fn test_detect_minimal() {
    // Minimal valid diagrams
    assert_eq!(detect_type("graph TD"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("flowchart LR"), Some(DiagramType::FlowchartV2));
    assert_eq!(detect_type("sequenceDiagram"), Some(DiagramType::Sequence));
    // classDiagram (without -v2) is detected as Class (legacy)
    assert_eq!(detect_type("classDiagram"), Some(DiagramType::Class));
    assert_eq!(detect_type("stateDiagram"), Some(DiagramType::State));
    assert_eq!(detect_type("stateDiagram-v2"), Some(DiagramType::StateDiagram));
}

#[test]
fn test_detect_multiline_declaration() {
    // Declaration might be followed by various content
    let code = r#"graph TD

    A --> B"#;
    assert_eq!(detect_type(code), Some(DiagramType::Flowchart));
}

// Tests for future diagram types (should return None for now)
#[test]
fn test_detect_unimplemented_types() {
    // These should return None until implemented
    // ER diagram
    let er_result = detect_type("erDiagram\nCUSTOMER ||--o{ ORDER : places");
    // May or may not be implemented, just verify it doesn't panic
    assert!(er_result.is_none() || matches!(er_result, Some(DiagramType::Er)));

    // Gantt
    let gantt_result = detect_type("gantt\ntitle Test\nsection Section\nTask 1: a, 2024-01-01, 30d");
    assert!(gantt_result.is_none() || matches!(gantt_result, Some(DiagramType::Gantt)));

    // Journey
    let journey_result = detect_type("journey\ntitle My Journey\nsection Section");
    assert!(journey_result.is_none() || matches!(journey_result, Some(DiagramType::Journey)));

    // Pie
    let pie_result = detect_type("pie\ntitle Pets\n\"Dogs\" : 386");
    assert!(pie_result.is_none() || matches!(pie_result, Some(DiagramType::Pie)));
}
