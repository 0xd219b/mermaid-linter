//! Integration tests for User Journey diagrams.

use mermaid_linter::{parse, DiagramType};

#[test]
fn test_simple_journey() {
    let code = r#"journey
    title My Journey
    section Section 1
    Task 1: 5: Actor"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple journey: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Journey));
    assert!(result.ast.is_some());
}

#[test]
fn test_journey_multiple_sections() {
    let code = r#"journey
    title Customer Journey
    section Awareness
    See advertisement: 5: Customer
    section Consideration
    Research product: 4: Customer
    section Purchase
    Buy product: 5: Customer"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse multiple sections: {:?}", result.diagnostics);
}

#[test]
fn test_journey_multiple_actors() {
    let code = r#"journey
    title Team Collaboration
    section Planning
    Discuss project: 4: Alice, Bob
    section Development
    Write code: 5: Alice
    Review code: 4: Bob
    section Testing
    Run tests: 5: Alice, Bob, Charlie"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse multiple actors: {:?}", result.diagnostics);
}

#[test]
fn test_journey_scores() {
    let code = r#"journey
    title Experience Ratings
    section Experience
    Bad experience: 1: User
    Poor experience: 2: User
    Neutral: 3: User
    Good experience: 4: User
    Great experience: 5: User"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse scores: {:?}", result.diagnostics);
}

#[test]
fn test_journey_with_title() {
    let code = r#"journey
    title My User Journey Diagram"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with title: {:?}", result.diagnostics);
}

#[test]
fn test_journey_accessibility() {
    let code = r#"journey
    accTitle: User Journey Title
    accDescr: This describes the user journey
    title My Journey
    section Start
    Begin: 5: User"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse accessibility: {:?}", result.diagnostics);
}

#[test]
fn test_journey_complex() {
    let code = r#"journey
    title My working day
    section Go to work
    Make tea: 5: Me
    Go upstairs: 3: Me
    Do work: 1: Me, Cat
    section Go home
    Go downstairs: 5: Me
    Sit down: 5: Me"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse complex journey: {:?}", result.diagnostics);
}

#[test]
fn test_journey_case_insensitive() {
    let code = r#"JOURNEY
    TITLE My Journey
    SECTION Section 1
    Task: 5: Actor"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse case-insensitive: {:?}", result.diagnostics);
}

#[test]
fn test_journey_invalid() {
    let code = "not a journey diagram";

    let result = parse(code, None);
    assert!(result.diagram_type != Some(DiagramType::Journey) || !result.ok);
}
