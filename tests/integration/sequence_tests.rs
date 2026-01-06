//! Integration tests for sequence diagrams.

use mermaid_linter::{parse, detect_type, DiagramType};

#[test]
fn test_simple_sequence() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Hello Bob
    Bob-->>Alice: Hi Alice"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple sequence: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Sequence));
    assert!(result.ast.is_some());
}

#[test]
fn test_sequence_with_participants() {
    let code = r#"sequenceDiagram
    participant A as Alice
    participant B as Bob
    A->>B: Hello Bob
    B-->>A: Hi Alice"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence with participants: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_with_actors() {
    let code = r#"sequenceDiagram
    actor User
    participant Server
    User->>Server: Request
    Server-->>User: Response"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence with actors: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_arrow_types() {
    let code = r#"sequenceDiagram
    A->>B: Solid arrow
    B-->>A: Dotted arrow
    A->B: Solid line
    B-->A: Dotted line
    A-xB: Cross
    B--xA: Dotted cross
    A-)B: Open arrow
    B--)A: Dotted open arrow"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence arrow types: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_activations() {
    let code = r#"sequenceDiagram
    Alice->>+Bob: Hello
    Bob-->>-Alice: Hi"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence activations: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_explicit_activation() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Hello
    activate Bob
    Bob-->>Alice: Hi
    deactivate Bob"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse explicit activations: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_notes() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Hello
    Note right of Bob: Text in note
    Note over Alice,Bob: Spanning note"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence with notes: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_note_positions() {
    let code = r#"sequenceDiagram
    participant A
    participant B
    Note left of A: Left note
    Note right of B: Right note
    Note over A: Over single
    Note over A,B: Over multiple"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence note positions: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_loop() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Hello
    loop Every minute
        Bob->>Alice: Keep alive
    end"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence loop: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_alt() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Request
    alt Success
        Bob-->>Alice: Success response
    else Failure
        Bob-->>Alice: Error response
    end"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence alt: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_opt() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Request
    opt Extra processing
        Bob->>Bob: Process
    end
    Bob-->>Alice: Response"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence opt: {:?}", result.diagnostics);
}

#[test]
#[ignore = "par/and block syntax not yet implemented"]
fn test_sequence_par() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Hello
    par Parallel 1
        Bob->>Charlie: Hi
    and Parallel 2
        Bob->>David: Hi
    end"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence par: {:?}", result.diagnostics);
}

#[test]
#[ignore = "critical/option block syntax not yet implemented"]
fn test_sequence_critical() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Request
    critical Establish connection
        Alice->>Bob: Ping
        Bob-->>Alice: Pong
    option Timeout
        Alice->>Alice: Retry
    end"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence critical: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_break() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Request
    break Something went wrong
        Bob-->>Alice: Error
    end"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence break: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_rect() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Hello
    rect rgb(200, 150, 255)
        Bob->>Charlie: Forward
        Charlie-->>Bob: Reply
    end
    Bob-->>Alice: Response"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence rect: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_autonumber() {
    let code = r#"sequenceDiagram
    autonumber
    Alice->>Bob: Step 1
    Bob-->>Alice: Step 2
    Alice->>Bob: Step 3"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence autonumber: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_title() {
    let code = r#"sequenceDiagram
    title Simple Sequence
    Alice->>Bob: Hello"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence title: {:?}", result.diagnostics);
}

#[test]
#[ignore = "link directive not yet implemented"]
fn test_sequence_links() {
    let code = r#"sequenceDiagram
    participant A as Alice
    link A: Dashboard @ https://dashboard.example.com"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence links: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_with_comments() {
    let code = r#"sequenceDiagram
    %% This is a comment
    Alice->>Bob: Hello
    %% Another comment
    Bob-->>Alice: Hi"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence with comments: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_nested_control_flow() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Request
    alt Valid
        opt Has data
            Bob->>Charlie: Forward
            Charlie-->>Bob: Response
        end
        Bob-->>Alice: Success
    else Invalid
        Bob-->>Alice: Error
    end"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse nested control flow: {:?}", result.diagnostics);
}

#[test]
fn test_detect_sequence() {
    assert_eq!(detect_type("sequenceDiagram\nAlice->>Bob: Hi"), Some(DiagramType::Sequence));
}

#[test]
fn test_sequence_empty_message() {
    let code = r#"sequenceDiagram
    Alice->>Bob:"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence with empty message: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_create_destroy() {
    let code = r#"sequenceDiagram
    Alice->>Bob: Hello
    create participant Charlie
    Bob->>Charlie: Hello
    destroy Charlie
    Charlie-->>Bob: Goodbye"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence create/destroy: {:?}", result.diagnostics);
}

#[test]
fn test_sequence_box() {
    let code = r#"sequenceDiagram
    box Purple
        participant Alice
        participant Bob
    end
    Alice->>Bob: Hello"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse sequence box: {:?}", result.diagnostics);
}
