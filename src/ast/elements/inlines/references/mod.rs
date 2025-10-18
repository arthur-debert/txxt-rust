//! Reference Span Elements
//!
//! References and citations using bracket notation for linking to
//! various targets (files, sections, citations, etc.)

pub mod citations;
pub mod footnote_ref;
pub mod page_ref;
pub mod session_ref;

use serde::{Deserialize, Serialize};

use crate::ast::{
    annotations::Annotation, parameters::Parameters, reference_types::ReferenceTarget,
    scanner_tokens::ScannerTokenSequence,
};

use super::super::core::{ElementType, SpanElement, TxxtElement};
use super::TextTransform;

// Re-export reference types
pub use citations::CitationSpan;
pub use footnote_ref::FootnoteReferenceSpan;
pub use page_ref::PageReferenceSpan;
pub use session_ref::SessionReferenceSpan;

/// General reference span for links and cross-references
///
/// Handles various reference types: [file.txxt], [#section], [url], etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceSpan {
    /// Comprehensive reference target with full type information
    pub target: ReferenceTarget,

    /// Optional custom display content (if not auto-generated)
    pub content: Option<Vec<TextTransform>>,

    /// Annotations attached to this reference
    pub annotations: Vec<Annotation>,

    /// Parameters for this reference
    pub parameters: Parameters,

    /// Raw tokens for language server support
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for ReferenceSpan {
    fn element_type(&self) -> ElementType {
        ElementType::Span
    }

    fn tokens(&self) -> &ScannerTokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl SpanElement for ReferenceSpan {
    fn text_content(&self) -> String {
        match &self.content {
            Some(content) => content
                .iter()
                .map(|transform| transform.text_content())
                .collect::<Vec<_>>()
                .join(""),
            None => {
                // Auto-generate display text from target
                format!("[{}]", self.target.display_text())
            }
        }
    }

    fn is_formatted(&self) -> bool {
        self.content.is_some()
    }
}

impl ReferenceSpan {
    /// Create a new reference span
    pub fn new(
        target: ReferenceTarget,
        content: Option<Vec<TextTransform>>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            target,
            content,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Get the reference target
    pub fn target(&self) -> &ReferenceTarget {
        &self.target
    }

    /// Check if this reference has custom content
    pub fn has_custom_content(&self) -> bool {
        self.content.is_some()
    }
}
