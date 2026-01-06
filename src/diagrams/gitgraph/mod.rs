//! GitGraph diagram parser.
//!
//! Parses git graph diagrams with commits, branches, and merges.
//!
//! # Syntax
//!
//! ```text
//! gitGraph
//!     commit
//!     branch develop
//!     checkout develop
//!     commit
//!     checkout main
//!     merge develop
//! ```

pub mod lexer;
pub mod parser;

pub use parser::GitGraphParser;
