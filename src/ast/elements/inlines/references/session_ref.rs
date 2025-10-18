//! Session Reference Span
//!
//! References to document sessions: [#1], [#2.1], etc.

use serde::{Deserialize, Serialize};

use crate::ast::{
    annotations::Annotation,
    parameters::Parameters,
    reference_types::{ReferenceTarget, SectionIdentifier},
    scanner_tokens::ScannerTokenSequence,
};

use super::super::super::core::{ElementType, SpanElement, TxxtElement};

/// Session reference span for cross-references to sessions
///
/// Examples: [#1], [#2.1], [#-1.3]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionReferenceSpan {
    /// Session identifier (numeric or hierarchical)
    pub session_identifier: SectionIdentifier,

    /// Raw reference text as it appears in source
    pub raw_text: String,

    /// Annotations attached to this reference
    pub annotations: Vec<Annotation>,

    /// Parameters for this reference
    pub parameters: Parameters,

    /// Raw tokens for language server support
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for SessionReferenceSpan {
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

impl SpanElement for SessionReferenceSpan {
    fn text_content(&self) -> String {
        self.raw_text.clone()
    }
}

impl SessionReferenceSpan {
    /// Create a new session reference span
    pub fn new(
        session_identifier: SectionIdentifier,
        raw_text: String,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            session_identifier,
            raw_text,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Check if this uses negative indexing
    pub fn uses_negative_index(&self) -> bool {
        self.session_identifier.uses_negative_index()
    }

    /// Convert to ReferenceTarget for compatibility
    pub fn to_reference_target(&self) -> ReferenceTarget {
        ReferenceTarget::Section {
            identifier: self.session_identifier.clone(),
            raw: self.raw_text.clone(),
            tokens: self.tokens.clone(),
        }
    }
}
