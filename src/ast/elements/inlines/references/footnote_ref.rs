//! Footnote Reference Span
//!
//! References to footnotes: [1], [^label]

use serde::{Deserialize, Serialize};

use crate::ast::{
    annotations::Annotation, parameters::Parameters, reference_types::ReferenceTarget,
    ScannerTokenSequence,
};

use super::super::super::core::{ElementType, SpanElement, TxxtElement};

/// Footnote reference span for footnote citations
///
/// Examples: [1], [2], [^label], [^named-footnote]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FootnoteReferenceSpan {
    /// Footnote identifier (number or label)
    pub footnote_id: String,

    /// Whether this is a numbered or labeled footnote
    pub is_labeled: bool,

    /// Raw reference text as it appears in source
    pub raw_text: String,

    /// Annotations attached to this reference
    pub annotations: Vec<Annotation>,

    /// Parameters for this reference
    pub parameters: Parameters,

    /// Raw tokens for language server support
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for FootnoteReferenceSpan {
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

impl SpanElement for FootnoteReferenceSpan {
    fn text_content(&self) -> String {
        self.raw_text.clone()
    }
}

impl FootnoteReferenceSpan {
    /// Create a new footnote reference span
    pub fn new(
        footnote_id: String,
        is_labeled: bool,
        raw_text: String,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            footnote_id,
            is_labeled,
            raw_text,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Get the footnote number (if this is a numbered footnote)
    pub fn footnote_number(&self) -> Option<u32> {
        if !self.is_labeled {
            self.footnote_id.parse().ok()
        } else {
            None
        }
    }

    /// Convert to ReferenceTarget for compatibility
    pub fn to_reference_target(&self) -> ReferenceTarget {
        if let Ok(number) = self.footnote_id.parse::<u32>() {
            ReferenceTarget::NakedNumerical {
                number,
                raw: self.raw_text.clone(),
                tokens: self.tokens.clone(),
            }
        } else {
            ReferenceTarget::NamedAnchor {
                anchor: self.footnote_id.clone(),
                raw: self.raw_text.clone(),
                tokens: self.tokens.clone(),
            }
        }
    }
}
