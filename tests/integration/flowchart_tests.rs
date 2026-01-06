//! Integration tests for flowchart diagrams.

use mermaid_linter::{parse, detect_type, DiagramType};

#[test]
fn test_simple_flowchart_graph_td() {
    let code = r#"graph TD
    A --> B
    B --> C"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple flowchart: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Flowchart));
    assert!(result.ast.is_some());
}

#[test]
fn test_simple_flowchart_graph_lr() {
    let code = r#"graph LR
    A --> B
    B --> C"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse LR flowchart: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Flowchart));
}

#[test]
fn test_flowchart_keyword() {
    let code = r#"flowchart TD
    A --> B
    B --> C"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart keyword: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::FlowchartV2));
}

#[test]
fn test_flowchart_with_node_labels() {
    let code = r#"graph TD
    A[Start] --> B[Process]
    B --> C[End]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with labels: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_with_node_shapes() {
    let code = r#"graph TD
    A[Rectangle] --> B(Rounded)
    B --> C{Diamond}
    C --> D([Stadium])
    D --> E[[Subroutine]]
    E --> F[(Database)]
    F --> G((Circle))"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with shapes: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_edge_types() {
    let code = r#"graph LR
    A --> B
    B --- C
    C -.-> D
    D ==> E
    E --o F
    F --x G"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with edge types: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_edge_labels() {
    let code = r#"graph LR
    A -->|text| B
    B -- text --> C
    C -.->|dotted| D"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with edge labels: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_subgraph() {
    let code = r#"graph TD
    subgraph one
        A --> B
    end
    subgraph two
        C --> D
    end
    B --> C"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with subgraph: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_nested_subgraphs() {
    let code = r#"graph TD
    subgraph outer
        subgraph inner
            A --> B
        end
        B --> C
    end"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with nested subgraphs: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_styling() {
    let code = r#"graph TD
    A --> B
    style A fill:#f9f,stroke:#333
    classDef className fill:#f00
    class B className"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with styling: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_chained_nodes() {
    let code = r#"graph LR
    A --> B --> C --> D"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with chained nodes: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_multiline() {
    let code = r#"graph TD
    A[This is a
    multiline label] --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with multiline: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_special_characters_in_id() {
    let code = r#"graph TD
    node1 --> node2
    endpoint --> sender
    default --> monograph"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with special ids: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_with_comments() {
    let code = r#"graph TD
    %% This is a comment
    A --> B
    %% Another comment
    B --> C"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with comments: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_directions() {
    let directions = ["TB", "TD", "BT", "RL", "LR"];

    for dir in &directions {
        let code = format!("graph {}\n    A --> B", dir);
        let result = parse(&code, None);
        assert!(result.ok, "Failed to parse flowchart with direction {}: {:?}", dir, result.diagnostics);
    }
}

#[test]
fn test_flowchart_link_styles() {
    let code = r#"graph LR
    A --> B
    linkStyle 0 stroke:#ff3,stroke-width:4px"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with link style: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_click_event() {
    let code = r#"graph TD
    A --> B
    click A callback
    click B "https://example.com""#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with click: {:?}", result.diagnostics);
}

#[test]
fn test_detect_flowchart_graph() {
    assert_eq!(detect_type("graph TD\nA-->B"), Some(DiagramType::Flowchart));
    assert_eq!(detect_type("graph LR\nA-->B"), Some(DiagramType::Flowchart));
}

#[test]
fn test_detect_flowchart_keyword() {
    assert_eq!(detect_type("flowchart TD\nA-->B"), Some(DiagramType::FlowchartV2));
    assert_eq!(detect_type("flowchart LR\nA-->B"), Some(DiagramType::FlowchartV2));
}

#[test]
fn test_flowchart_trailing_whitespace() {
    let code = "graph TD;\n\n\n A-->B; \n B-->C;";
    let result = parse(code, None);
    assert!(result.ok, "Failed to handle trailing whitespace: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_empty_lines() {
    let code = r#"
graph TD

    A --> B

    B --> C

"#;
    let result = parse(code, None);
    assert!(result.ok, "Failed to handle empty lines: {:?}", result.diagnostics);
}

#[test]
fn test_flowchart_semicolons() {
    let code = "graph TD;A-->B;B-->C;";
    let result = parse(code, None);
    assert!(result.ok, "Failed to parse flowchart with semicolons: {:?}", result.diagnostics);
}
