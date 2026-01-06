//! Integration tests for class diagrams.

use mermaid_linter::{parse, detect_type, DiagramType};

#[test]
fn test_simple_class_diagram() {
    let code = r#"classDiagram
    class Animal
    class Dog"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple class diagram: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Class));
    assert!(result.ast.is_some());
}

#[test]
fn test_class_with_members() {
    let code = r#"classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class with members: {:?}", result.diagnostics);
}

#[test]
fn test_class_visibility_modifiers() {
    let code = r#"classDiagram
    class MyClass {
        +publicMethod()
        -privateMethod()
        #protectedMethod()
        ~packageMethod()
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse visibility modifiers: {:?}", result.diagnostics);
}

#[test]
fn test_class_relationships() {
    let code = r#"classDiagram
    Animal <|-- Dog
    Animal <|-- Cat
    Vehicle o-- Wheel
    Company *-- Employee"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class relationships: {:?}", result.diagnostics);
}

#[test]
fn test_class_relationship_types() {
    let code = r#"classDiagram
    A <|-- B : Inheritance
    C *-- D : Composition
    E o-- F : Aggregation
    G --> H : Association
    I -- J : Link
    K ..> L : Dependency
    M ..|> N : Realization"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse relationship types: {:?}", result.diagnostics);
}

#[test]
fn test_class_cardinality() {
    let code = r#"classDiagram
    Customer "1" --> "*" Order
    Order "*" --> "1..*" LineItem
    Student "0..*" --> "0..*" Course"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse cardinality: {:?}", result.diagnostics);
}

#[test]
fn test_class_annotations() {
    let code = r#"classDiagram
    class Shape {
        <<interface>>
        +draw()
    }
    class AbstractFactory {
        <<abstract>>
        +createProduct()
    }
    class DatabaseService {
        <<service>>
        +connect()
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse annotations: {:?}", result.diagnostics);
}

#[test]
#[ignore = "Generic type syntax not yet implemented"]
fn test_class_generic_types() {
    let code = r#"classDiagram
    class List~T~ {
        +add(T item)
        +get(int index) T
    }
    class Map~K,V~ {
        +put(K key, V value)
        +get(K key) V
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse generic types: {:?}", result.diagnostics);
}

#[test]
#[ignore = "Namespace syntax not yet implemented"]
fn test_class_with_namespace() {
    let code = r#"classDiagram
    namespace Animals {
        class Dog {
            +bark()
        }
        class Cat {
            +meow()
        }
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse namespace: {:?}", result.diagnostics);
}

#[test]
fn test_class_direction() {
    let code = r#"classDiagram
    direction LR
    class A
    class B
    A --> B"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class direction: {:?}", result.diagnostics);
}

#[test]
fn test_class_note() {
    let code = r#"classDiagram
    class Shape
    note "This is a note"
    note for Shape "Specific note""#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class notes: {:?}", result.diagnostics);
}

#[test]
fn test_class_link() {
    let code = r#"classDiagram
    class Duck
    link Duck "https://example.com" "Tooltip""#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class link: {:?}", result.diagnostics);
}

#[test]
fn test_class_click() {
    let code = r#"classDiagram
    class Shape
    click Shape callback
    click Shape href "https://example.com""#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class click: {:?}", result.diagnostics);
}

#[test]
fn test_class_styling() {
    let code = r#"classDiagram
    class Shape
    style Shape fill:#f9f,stroke:#333"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class styling: {:?}", result.diagnostics);
}

#[test]
fn test_class_with_comments() {
    let code = r#"classDiagram
    %% This is a comment
    class Animal
    %% Another comment
    class Dog"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse class with comments: {:?}", result.diagnostics);
}

#[test]
fn test_class_return_types() {
    let code = r#"classDiagram
    class Service {
        +getData() String
        +getItems() List~String~
        +void process()
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse return types: {:?}", result.diagnostics);
}

#[test]
fn test_class_method_parameters() {
    let code = r#"classDiagram
    class Calculator {
        +add(int a, int b) int
        +multiply(double x, double y) double
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse method parameters: {:?}", result.diagnostics);
}

#[test]
fn test_class_static_abstract() {
    let code = r#"classDiagram
    class MyClass {
        +String staticField$
        +abstractMethod()*
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse static/abstract markers: {:?}", result.diagnostics);
}

#[test]
fn test_detect_class_diagram() {
    assert_eq!(detect_type("classDiagram\nclass Animal"), Some(DiagramType::Class));
}

#[test]
fn test_detect_class_diagram_legacy() {
    // Legacy `class` keyword only (without Diagram)
    // This might be detected differently
    let result = detect_type("classDiagram\nAnimal <|-- Dog");
    assert_eq!(result, Some(DiagramType::Class));
}

#[test]
fn test_class_complex_relationships() {
    let code = r#"classDiagram
    classA --|> classB : Inheritance
    classC --* classD : Composition
    classE --o classF : Aggregation
    classG --> classH : Association
    classI -- classJ : Link(Solid)
    classK ..> classL : Dependency
    classM ..|> classN : Realization
    classO .. classP : Link(Dashed)"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse complex relationships: {:?}", result.diagnostics);
}

#[test]
fn test_class_empty_class_body() {
    let code = r#"classDiagram
    class EmptyClass {
    }"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse empty class body: {:?}", result.diagnostics);
}

#[test]
#[ignore = "Inline class definition with label not yet implemented"]
fn test_class_inline_definition() {
    let code = r#"classDiagram
    class Animal["Custom Label"]"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse inline class definition: {:?}", result.diagnostics);
}
