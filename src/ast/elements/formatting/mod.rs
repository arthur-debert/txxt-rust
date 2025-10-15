//! Formatting Elements
//!
//! Text formatting elements for emphasis, code, math, etc.

pub mod formatting_spans;
pub mod inlines;
pub mod spans;

// Re-export formatting types
pub use inlines::*;
pub use spans::{BoldSpan, CodeSpan, ItalicSpan, MathSpan};
