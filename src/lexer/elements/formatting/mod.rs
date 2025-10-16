//! Formatting tokenization modules
//!
//! This module contains all formatting-type tokenizers that implement the formatting
//! patterns defined in docs/specs/elements/formatting/formatting.txxt.

pub mod delimiters;

// Re-export the main functions and traits for easy access
pub use delimiters::{read_inline_delimiter, InlineDelimiterLexer};
