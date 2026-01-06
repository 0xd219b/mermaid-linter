//! User Journey diagram parser.
//!
//! Parses user journey diagrams with tasks, sections, and actors.
//!
//! # Syntax
//!
//! ```text
//! journey
//!     title My working day
//!     section Go to work
//!         Make tea: 5: Me
//!         Go upstairs: 3: Me
//!         Do work: 1: Me, Cat
//! ```

pub mod lexer;
pub mod parser;

pub use parser::JourneyParser;

#[cfg(test)]
mod tests {
    use super::parser::JourneyParser;

    #[test]
    fn test_parse_simple_journey() {
        let code = r#"journey
    title My Journey
    section Section 1
    Task 1: 5: Actor"#;

        let mut parser = JourneyParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }
}
