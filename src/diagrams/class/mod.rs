//! Class diagram parser.
//!
//! Supports the Mermaid class diagram syntax.
//!
//! # Example
//!
//! ```text
//! classDiagram
//!     class Animal {
//!         +String name
//!         +int age
//!         +makeSound()
//!     }
//!     Animal <|-- Dog
//!     Animal <|-- Cat
//! ```

mod lexer;
mod parser;

pub use parser::ClassParser;

use crate::ast::Span;

/// Visibility of a class member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Public,     // +
    Private,    // -
    Protected,  // #
    Package,    // ~
}

impl Visibility {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Visibility::Public),
            '-' => Some(Visibility::Private),
            '#' => Some(Visibility::Protected),
            '~' => Some(Visibility::Package),
            _ => None,
        }
    }
}

/// Type of relationship between classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelationType {
    #[default]
    Association,    // --
    Inheritance,    // <|--
    Composition,    // *--
    Aggregation,    // o--
    Dependency,     // ..>
    Realization,    // ..|>
    Link,           // --
    DashedLink,     // ..
}

impl RelationType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "<|--" | "--|>" => Some(RelationType::Inheritance),
            "*--" | "--*" => Some(RelationType::Composition),
            "o--" | "--o" => Some(RelationType::Aggregation),
            "..>" | "<.." => Some(RelationType::Dependency),
            "..|>" | "<|.." => Some(RelationType::Realization),
            "--" => Some(RelationType::Association),
            ".." => Some(RelationType::DashedLink),
            _ => None,
        }
    }
}

/// A member of a class (attribute or method).
#[derive(Debug, Clone)]
pub struct ClassMember {
    pub name: String,
    pub member_type: Option<String>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_method: bool,
    pub parameters: Option<String>,
    pub return_type: Option<String>,
    pub span: Span,
}

/// A class in the diagram.
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub stereotype: Option<String>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

/// A relationship between classes.
#[derive(Debug, Clone)]
pub struct ClassRelation {
    pub from: String,
    pub to: String,
    pub relation_type: RelationType,
    pub label: Option<String>,
    pub from_cardinality: Option<String>,
    pub to_cardinality: Option<String>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_from_char() {
        assert_eq!(Visibility::from_char('+'), Some(Visibility::Public));
        assert_eq!(Visibility::from_char('-'), Some(Visibility::Private));
        assert_eq!(Visibility::from_char('#'), Some(Visibility::Protected));
        assert_eq!(Visibility::from_char('~'), Some(Visibility::Package));
        assert_eq!(Visibility::from_char('x'), None);
    }

    #[test]
    fn test_relation_type_from_str() {
        assert_eq!(RelationType::from_str("<|--"), Some(RelationType::Inheritance));
        assert_eq!(RelationType::from_str("*--"), Some(RelationType::Composition));
        assert_eq!(RelationType::from_str("o--"), Some(RelationType::Aggregation));
        assert_eq!(RelationType::from_str("..>"), Some(RelationType::Dependency));
    }
}
