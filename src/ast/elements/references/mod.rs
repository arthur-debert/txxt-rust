//! Reference Elements
//!
//! Reference elements for links, citations, footnotes, etc.

pub mod citations;
pub mod footnote_ref;
pub mod page_ref;
pub mod reference_types;
pub mod session_ref;

// Re-export reference types
pub use citations::CitationSpan;
pub use footnote_ref::FootnoteReferenceSpan;
pub use page_ref::PageReferenceSpan;
pub use reference_types::*;
pub use session_ref::SessionReferenceSpan;

// General reference span for links and cross-references
// (copied from inlines/references/mod.rs for functionality-driven organization)

use crate::ast::elements::formatting::inlines::TextTransform;
use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    tokens::TokenSequence,
};
use serde::{Deserialize, Serialize};

/// General reference span for links and cross-references
///
/// Handles various reference types: [file.txxt], [#section], [url], etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceSpan {
    /// Comprehensive reference target with full type information
    pub target: crate::ast::elements::references::reference_types::ReferenceTarget,

    /// Optional custom display content (if not auto-generated)
    pub content: Option<Vec<TextTransform>>,

    /// Annotations attached to this reference
    pub annotations: Vec<Annotation>,

    /// Parameters for this reference
    pub parameters: Parameters,

    /// Raw tokens for language server support
    pub tokens: TokenSequence,
}

/// Reference inline element for the formatting system
///
/// This is the core reference type used in the inline formatting system.
/// It provides a simpler interface than ReferenceSpan for basic reference parsing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reference {
    /// Reference target with comprehensive type information
    pub target: ReferenceTarget,

    /// Optional content (usually None for simple references)
    pub content: Option<Vec<TextTransform>>,

    /// Source tokens for positioning
    pub tokens: TokenSequence,
}

// ReferenceSpan is already defined above, no need to re-export
