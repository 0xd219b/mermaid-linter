//! ER (Entity-Relationship) diagram parser.
//!
//! Parses entity-relationship diagrams with entities, attributes, and relationships.
//!
//! # Syntax
//!
//! ```text
//! erDiagram
//!     CUSTOMER ||--o{ ORDER : places
//!     CUSTOMER {
//!         string name
//!         string custNumber PK
//!     }
//! ```

pub mod lexer;
pub mod parser;

pub use parser::ErParser;

/// ER diagram cardinality types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// Exactly one (||)
    OnlyOne,
    /// Zero or one (|o or o|)
    ZeroOrOne,
    /// One or more (}| or |{)
    OneOrMore,
    /// Zero or more (}o or o{)
    ZeroOrMore,
    /// MD Parent (u) - for multi-domain relationships
    MdParent,
}

impl Cardinality {
    /// Parse cardinality from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "||" | "only one" | "1" => Some(Cardinality::OnlyOne),
            "|o" | "o|" | "zero or one" | "one or zero" => Some(Cardinality::ZeroOrOne),
            "}|" | "|{" | "one or more" | "one or many" | "1+" | "many(1)" => {
                Some(Cardinality::OneOrMore)
            }
            "}o" | "o{" | "zero or more" | "zero or many" | "0+" | "many(0)" => {
                Some(Cardinality::ZeroOrMore)
            }
            "u" => Some(Cardinality::MdParent),
            _ => None,
        }
    }

    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Cardinality::OnlyOne => "ONLY_ONE",
            Cardinality::ZeroOrOne => "ZERO_OR_ONE",
            Cardinality::OneOrMore => "ONE_OR_MORE",
            Cardinality::ZeroOrMore => "ZERO_OR_MORE",
            Cardinality::MdParent => "MD_PARENT",
        }
    }
}

/// Relationship identification type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentificationType {
    /// Identifying relationship (solid line, --)
    Identifying,
    /// Non-identifying relationship (dashed line, ..)
    NonIdentifying,
}

impl IdentificationType {
    /// Parse identification type from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "--" | "to" => Some(IdentificationType::Identifying),
            ".." | "optionally to" => Some(IdentificationType::NonIdentifying),
            _ => None,
        }
    }

    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            IdentificationType::Identifying => "IDENTIFYING",
            IdentificationType::NonIdentifying => "NON_IDENTIFYING",
        }
    }
}

/// Attribute key types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeKey {
    /// Primary key
    PrimaryKey,
    /// Foreign key
    ForeignKey,
    /// Unique key
    UniqueKey,
}

impl AttributeKey {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PK" => Some(AttributeKey::PrimaryKey),
            "FK" => Some(AttributeKey::ForeignKey),
            "UK" => Some(AttributeKey::UniqueKey),
            _ => None,
        }
    }

    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            AttributeKey::PrimaryKey => "PK",
            AttributeKey::ForeignKey => "FK",
            AttributeKey::UniqueKey => "UK",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_from_str() {
        assert_eq!(Cardinality::from_str("||"), Some(Cardinality::OnlyOne));
        assert_eq!(Cardinality::from_str("|o"), Some(Cardinality::ZeroOrOne));
        assert_eq!(Cardinality::from_str("}|"), Some(Cardinality::OneOrMore));
        assert_eq!(Cardinality::from_str("}o"), Some(Cardinality::ZeroOrMore));
    }

    #[test]
    fn test_identification_type_from_str() {
        assert_eq!(
            IdentificationType::from_str("--"),
            Some(IdentificationType::Identifying)
        );
        assert_eq!(
            IdentificationType::from_str(".."),
            Some(IdentificationType::NonIdentifying)
        );
    }

    #[test]
    fn test_attribute_key_from_str() {
        assert_eq!(
            AttributeKey::from_str("PK"),
            Some(AttributeKey::PrimaryKey)
        );
        assert_eq!(
            AttributeKey::from_str("FK"),
            Some(AttributeKey::ForeignKey)
        );
        assert_eq!(AttributeKey::from_str("UK"), Some(AttributeKey::UniqueKey));
    }
}
