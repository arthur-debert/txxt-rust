//! Formatting tokenization modules
//!
//! This module contains all formatting-type tokenizers that implement the formatting
//! patterns defined in docs/specs/elements/inlines/formatting.txxt.

pub mod delimiters;
pub mod math_span;

// Re-export the main functions and traits for easy access
pub use delimiters::{read_inline_delimiter, InlineDelimiterLexer};
pub use math_span::{read_math_span, MathSpanLexer};
