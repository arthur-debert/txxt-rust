//! Citation Reference Span
//!
//! Academic citations using @ syntax: [@smith2023], [@doe2024; @jones2025]

use serde::{Deserialize, Serialize};

use crate::ast::{
    annotations::Annotation,
    parameters::Parameters,
    reference_types::{CitationEntry, ReferenceTarget},
    scanner_tokens::ScannerTokenSequence,
};

use super::super::super::core::{ElementType, SpanElement, TxxtElement};

/// Citation span for academic references
///
/// Examples: [@smith2023], [@doe2024; @jones2025], [@smith2023, p. 45]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CitationSpan {
    /// Citation entries with keys and locators
    pub citations: Vec<CitationEntry>,

    /// Raw reference text as it appears in source
    pub raw_text: String,

    /// Annotations attached to this citation
    pub annotations: Vec<Annotation>,

    /// Parameters for this citation
    pub parameters: Parameters,

    /// Raw tokens for language server support
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for CitationSpan {
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

impl SpanElement for CitationSpan {
    fn text_content(&self) -> String {
        self.raw_text.clone()
    }

    fn is_formatted(&self) -> bool {
        false // Citations use their own rendering
    }
}

impl CitationSpan {
    /// Create a new citation span
    pub fn new(
        citations: Vec<CitationEntry>,
        raw_text: String,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            citations,
            raw_text,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Get all citation keys
    pub fn citation_keys(&self) -> Vec<&str> {
        self.citations.iter().map(|c| c.key.as_str()).collect()
    }

    /// Check if this is a multi-citation reference
    pub fn is_multi_citation(&self) -> bool {
        self.citations.len() > 1
    }

    /// Convert to ReferenceTarget for compatibility
    pub fn to_reference_target(&self) -> ReferenceTarget {
        ReferenceTarget::Citation {
            citations: self.citations.clone(),
            raw: self.raw_text.clone(),
            tokens: self.tokens.clone(),
        }
    }
}
