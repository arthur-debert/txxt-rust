//! Inline element detection and parsing
//!
//! This module handles inline elements within text content, including
//! formatting delimiters and parameter parsing.

pub mod citation_ref;
pub mod formatting;
pub mod math_span;
pub mod parameters;

// Re-export public interfaces
pub use citation_ref::{read_citation_ref, CitationRefLexer};
pub use formatting::{read_inline_delimiter, InlineDelimiterLexer};
pub use math_span::{read_math_span, MathSpanLexer};
pub use parameters::{parse_parameters, ParameterLexer};
