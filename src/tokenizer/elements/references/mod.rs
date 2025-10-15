//! Reference tokenization modules
//!
//! This module contains all reference-type tokenizers that implement the reference
//! patterns defined in docs/specs/elements/references/.

pub mod citations;
pub mod footnote_ref;
pub mod general;
pub mod page_ref;
pub mod session_ref;

// Re-export the main functions and traits for easy access
pub use citations::{read_citation_ref, CitationRefLexer};
pub use footnote_ref::{read_footnote_ref, FootnoteRefLexer, FootnoteType};
pub use general::ReferenceLexer;
pub use page_ref::{read_page_ref, PageRefLexer};
pub use session_ref::{read_session_ref, SessionRefLexer};
