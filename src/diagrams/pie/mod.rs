//! Pie chart parser.
//!
//! Parses pie charts with slices and values.
//!
//! # Syntax
//!
//! ```text
//! pie showData
//!     title Key elements
//!     "Calcium" : 42.96
//!     "Potassium" : 50.05
//!     "Magnesium" : 10.01
//! ```

pub mod lexer;
pub mod parser;

pub use parser::PieParser;
