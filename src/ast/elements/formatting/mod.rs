//! Formatting Elements
//!
//! Text formatting elements for emphasis, code, math, etc.

pub mod spans;

// Re-export formatting types
pub use spans::{BoldSpan, CodeSpan, ItalicSpan, MathSpan};
