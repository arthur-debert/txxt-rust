//! Inline element detection and parsing
//!
//! This module handles inline elements within text content, organized by
//! the specification structure in docs/specs/elements/inlines/.

pub mod formatting;
pub mod parameters;
pub mod references;

// Re-export public interfaces
pub use formatting::{read_inline_delimiter, read_math_span, InlineDelimiterLexer, MathSpanLexer};
pub use parameters::{parse_parameters, ParameterLexer};
pub use references::{
    read_citation_ref, read_page_ref, read_session_ref, CitationRefLexer, PageRefLexer,
    ReferenceLexer, SessionRefLexer,
};
