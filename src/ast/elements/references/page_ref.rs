//! Page Reference Span
//!
//! References to page numbers: [p. 45], [pp. 23-25]

use serde::{Deserialize, Serialize};

use crate::cst::ScannerTokenSequence;
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    ScannerTokenSequence,
use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
};

use super::super::core::{ElementType, SpanElement, TxxtElement};

/// Page reference span for page citations
///
/// Examples: [p. 45], [pp. 23-25], [page 100]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageReferenceSpan {
    /// Page specification (number, range, etc.)
    pub page_spec: String,

    /// Whether this is a single page (p.) or multiple pages (pp.)
    pub is_multiple: bool,

    /// Raw reference text as it appears in source
    pub raw_text: String,

    /// Annotations attached to this reference
    pub annotations: Vec<Annotation>,

    /// Parameters for this reference
    pub parameters: Parameters,

    /// Raw tokens for language server support
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for PageReferenceSpan {
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

impl SpanElement for PageReferenceSpan {
    fn text_content(&self) -> String {
        self.raw_text.clone()
    }
}

impl PageReferenceSpan {
    /// Create a new page reference span
    pub fn new(
        page_spec: String,
        is_multiple: bool,
        raw_text: String,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            page_spec,
            is_multiple,
            raw_text,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Parse page numbers from the specification
    pub fn page_numbers(&self) -> Vec<u32> {
        // Simple parsing - in practice this would be more sophisticated
        self.page_spec
            .split('-')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }

    /// Check if this is a page range
    pub fn is_range(&self) -> bool {
        self.page_spec.contains('-')
    }
}
