//! Integration tests for state diagrams.

use mermaid_linter::{parse, detect_type, DiagramType};

#[test]
fn test_simple_state_diagram() {
    let code = r#"stateDiagram-v2
    [*] --> State1
    State1 --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple state diagram: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::StateDiagram));
    assert!(result.ast.is_some());
}

#[test]
fn test_state_diagram_legacy() {
    let code = r#"stateDiagram
    [*] --> State1
    State1 --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse legacy state diagram: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::State));
}

#[test]
fn test_state_transitions() {
    let code = r#"stateDiagram-v2
    [*] --> Still
    Still --> [*]
    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state transitions: {:?}", result.diagnostics);
}

#[test]
fn test_state_transition_labels() {
    let code = r#"stateDiagram-v2
    State1 --> State2 : Trigger event
    State2 --> State3 : Another event"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse transition labels: {:?}", result.diagnostics);
}

#[test]
fn test_state_description() {
    let code = r#"stateDiagram-v2
    state "This is a state" as s1
    s1 --> s2
    s2 : This is state 2"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state descriptions: {:?}", result.diagnostics);
}

#[test]
fn test_state_composite() {
    let code = r#"stateDiagram-v2
    [*] --> First
    state First {
        [*] --> Second
        Second --> Third
        Third --> [*]
    }
    First --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse composite state: {:?}", result.diagnostics);
}

#[test]
fn test_state_nested_composite() {
    let code = r#"stateDiagram-v2
    state Outer {
        state Inner {
            [*] --> InnerState
            InnerState --> [*]
        }
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse nested composite state: {:?}", result.diagnostics);
}

#[test]
fn test_state_fork_join() {
    let code = r#"stateDiagram-v2
    state fork_state <<fork>>
    [*] --> fork_state
    fork_state --> State2
    fork_state --> State3

    state join_state <<join>>
    State2 --> join_state
    State3 --> join_state
    join_state --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse fork/join: {:?}", result.diagnostics);
}

#[test]
fn test_state_choice() {
    let code = r#"stateDiagram-v2
    state if_state <<choice>>
    [*] --> if_state
    if_state --> State1 : condition1
    if_state --> State2 : condition2"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse choice: {:?}", result.diagnostics);
}

#[test]
fn test_state_notes() {
    let code = r#"stateDiagram-v2
    State1 --> State2
    note right of State1
        This is a note
    end note"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state notes: {:?}", result.diagnostics);
}

#[test]
fn test_state_note_positions() {
    let code = r#"stateDiagram-v2
    State1 --> State2
    note left of State1 : Left note
    note right of State2 : Right note"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse note positions: {:?}", result.diagnostics);
}

#[test]
fn test_state_concurrent() {
    let code = r#"stateDiagram-v2
    [*] --> Active
    state Active {
        [*] --> NumLockOff
        NumLockOff --> NumLockOn : EvNumLockPressed
        NumLockOn --> NumLockOff : EvNumLockPressed
        --
        [*] --> CapsLockOff
        CapsLockOff --> CapsLockOn : EvCapsLockPressed
        CapsLockOn --> CapsLockOff : EvCapsLockPressed
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse concurrent states: {:?}", result.diagnostics);
}

#[test]
fn test_state_direction() {
    let code = r#"stateDiagram-v2
    direction LR
    [*] --> A
    A --> B
    B --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state direction: {:?}", result.diagnostics);
}

#[test]
fn test_state_with_comments() {
    let code = r#"stateDiagram-v2
    %% Comment 1
    [*] --> State1
    %% Comment 2
    State1 --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state with comments: {:?}", result.diagnostics);
}

#[test]
fn test_state_special_keywords() {
    let code = r#"stateDiagram-v2
    assemble --> assemblies
    state assemble
    state assemblies"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse states with 'as' in name: {:?}", result.diagnostics);
}

#[test]
fn test_state_styling() {
    let code = r#"stateDiagram-v2
    State1 --> State2
    classDef green fill:#0f0,stroke:#333
    class State1 green"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state styling: {:?}", result.diagnostics);
}

#[test]
fn test_state_title() {
    let code = r#"---
title: State Machine
---
stateDiagram-v2
    [*] --> State1"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state with title: {:?}", result.diagnostics);
    assert_eq!(result.title, Some("State Machine".to_string()));
}

#[test]
fn test_detect_state_diagram_v2() {
    assert_eq!(detect_type("stateDiagram-v2\n[*] --> A"), Some(DiagramType::StateDiagram));
}

#[test]
fn test_detect_state_diagram_legacy() {
    assert_eq!(detect_type("stateDiagram\n[*] --> A"), Some(DiagramType::State));
}

#[test]
fn test_state_start_end_markers() {
    let code = r#"stateDiagram-v2
    [*] --> Active
    Active --> [*]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse start/end markers: {:?}", result.diagnostics);
}

#[test]
fn test_state_multiple_transitions() {
    let code = r#"stateDiagram-v2
    State1 --> State2
    State1 --> State3
    State2 --> State4
    State3 --> State4"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse multiple transitions: {:?}", result.diagnostics);
}

#[test]
fn test_state_self_transition() {
    let code = r#"stateDiagram-v2
    State1 --> State1 : self loop"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse self transition: {:?}", result.diagnostics);
}

#[test]
fn test_state_as_keyword() {
    let code = r#"stateDiagram-v2
    state "as" as as
    as --> State1"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse state with 'as' as name: {:?}", result.diagnostics);
}

#[test]
fn test_state_escaping() {
    let code = r#"stateDiagram-v2
    State1 --> State2 : Event with \n newline"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse escaped characters: {:?}", result.diagnostics);
}
