//! Integration tests for ER (Entity-Relationship) diagrams.

use mermaid_linter::{parse, DiagramType};

#[test]
fn test_simple_er_diagram() {
    let code = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple ER diagram: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Er));
    assert!(result.ast.is_some());
}

#[test]
fn test_er_multiple_relationships() {
    let code = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places
    ORDER ||--|{ LINE-ITEM : contains
    CUSTOMER }|..|{ DELIVERY-ADDRESS : uses"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse multiple relationships: {:?}", result.diagnostics);
}

#[test]
fn test_er_entity_with_attributes() {
    let code = r#"erDiagram
    CUSTOMER {
        string name
        string custNumber PK
        string sector
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse entity with attributes: {:?}", result.diagnostics);
}

#[test]
fn test_er_attribute_keys() {
    let code = r#"erDiagram
    ORDER {
        int orderNumber PK
        string customer FK
        string email UK
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse attribute keys: {:?}", result.diagnostics);
}

#[test]
fn test_er_combined_keys() {
    let code = r#"erDiagram
    LINE-ITEM {
        string productCode PK, FK
        int quantity
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse combined keys: {:?}", result.diagnostics);
}

#[test]
fn test_er_attribute_comments() {
    let code = r#"erDiagram
    CUSTOMER {
        string name "Full name of customer"
        int id PK "Unique identifier"
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse attribute comments: {:?}", result.diagnostics);
}

#[test]
fn test_er_quoted_entity_names() {
    let code = r#"erDiagram
    "Customer Entity" ||--o{ "Order Entity" : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse quoted entity names: {:?}", result.diagnostics);
}

#[test]
fn test_er_cardinality_only_one() {
    let code = r#"erDiagram
    PERSON ||--|| PASSPORT : has"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse only-one cardinality: {:?}", result.diagnostics);
}

#[test]
fn test_er_cardinality_zero_or_one() {
    let code = r#"erDiagram
    PERSON |o--o| SPOUSE : "married to""#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse zero-or-one cardinality: {:?}", result.diagnostics);
}

#[test]
fn test_er_cardinality_one_or_more() {
    let code = r#"erDiagram
    PARENT }|--|{ CHILD : "parent of""#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse one-or-more cardinality: {:?}", result.diagnostics);
}

#[test]
fn test_er_cardinality_zero_or_more() {
    let code = r#"erDiagram
    PERSON }o--o{ PET : owns"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse zero-or-more cardinality: {:?}", result.diagnostics);
}

#[test]
fn test_er_identifying_relationship() {
    let code = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse identifying relationship: {:?}", result.diagnostics);
}

#[test]
fn test_er_non_identifying_relationship() {
    let code = r#"erDiagram
    CUSTOMER ||..o{ ORDER : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse non-identifying relationship: {:?}", result.diagnostics);
}

#[test]
fn test_er_direction() {
    let code = r#"erDiagram
    direction LR
    CUSTOMER ||--o{ ORDER : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with direction: {:?}", result.diagnostics);
}

#[test]
fn test_er_full_example() {
    let code = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places
    CUSTOMER {
        string name
        string custNumber PK
        string sector
    }
    ORDER ||--|{ LINE-ITEM : contains
    ORDER {
        int orderNumber PK
        string deliveryAddress
    }
    LINE-ITEM {
        string productCode PK, FK
        int quantity
        float pricePerUnit
    }
    PRODUCT {
        int productId PK
        string name
        string description
        decimal price
    }
    LINE-ITEM ||--|| PRODUCT : "refers to""#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse full ER example: {:?}", result.diagnostics);
}

#[test]
fn test_er_case_insensitive() {
    let code = r#"ERDIAGRAM
    Customer ||--o{ Order : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse case-insensitive ER diagram: {:?}", result.diagnostics);
}

#[test]
fn test_er_hyphenated_names() {
    let code = r#"erDiagram
    ORDER-ITEM ||--|| PRODUCT-INFO : contains"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse hyphenated entity names: {:?}", result.diagnostics);
}

#[test]
fn test_er_empty_entity() {
    let code = r#"erDiagram
    CUSTOMER
    ORDER
    CUSTOMER ||--o{ ORDER : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse empty entities: {:?}", result.diagnostics);
}

#[test]
fn test_er_invalid_syntax() {
    let code = "not an er diagram";

    let result = parse(code, None);
    // Should either fail or return a different diagram type
    assert!(result.diagram_type != Some(DiagramType::Er) || !result.ok);
}

#[test]
fn test_er_accessibility() {
    let code = r#"erDiagram
    accTitle: ER Diagram Title
    accDescr: This diagram shows the entity relationships
    CUSTOMER ||--o{ ORDER : places"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse accessibility statements: {:?}", result.diagnostics);
}
