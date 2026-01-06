//! Text preprocessing for Mermaid diagrams.
//!
//! This module handles all preprocessing steps before parsing:
//! - Normalize line endings (CRLF -> LF)
//! - Convert HTML attribute quotes (double -> single)
//! - Extract and parse YAML frontmatter
//! - Extract and parse directives (%%{...}%%)
//! - Remove comments (%% ...)

mod comments;
mod directive;
mod frontmatter;
mod normalize;
pub mod preprocessor;

pub use comments::remove_comments;
pub use directive::{parse_directive, Directive, DirectiveType};
pub use frontmatter::{extract_frontmatter, FrontmatterResult};
pub use normalize::{encode_entities, normalize_text};
pub use preprocessor::{PreprocessResult, Preprocessor};
