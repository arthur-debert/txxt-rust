//! Inline Elements (Span Elements)
//!
//! This module implements inline elements following the text-transform layer
//! architecture. Every piece of text goes through a transform layer for
//! uniform processing across the AST.
//!
//! From `docs/specs/core/terminology.txxt`:
//! "Span: The smallest unit of text that does not include line breaks.
//! A line can host multiple spans."

// Legacy inline elements - these have been moved to functional modules
// pub mod formatting;  // moved to formatting/
// pub mod references;  // moved to references/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
};
use crate::cst::{ScannerToken, ScannerTokenSequence};

use super::core::{ElementType, SpanElement, TxxtElement};
use super::references::reference_types::ReferenceTarget;

// Re-export inline types from new functional modules
pub use super::formatting::inlines::{Text, TextTransform};
pub use super::formatting::{BoldSpan, CodeSpan, ItalicSpan, MathSpan};
pub use super::references::{
    CitationSpan, FootnoteReferenceSpan, PageReferenceSpan, ReferenceSpan, SessionReferenceSpan,
};

/// Text span - the fundamental span element for plain text
///
/// Text spans represent plain text content without formatting.
/// They are the foundation of all text content in the AST.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSpan {
    /// Token sequence with character-level precision
    pub tokens: ScannerTokenSequence,

    /// Annotations (rare for text spans)
    pub annotations: Vec<Annotation>,

    /// Parameters (rare for text spans)
    pub parameters: Parameters,
}

/// Text line - a complete line containing span elements
///
/// Text lines encompass full lines of text and may contain multiple spans.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine {
    /// Span elements within this line
    pub spans: Vec<TextTransform>,

    /// Annotations attached to this line
    pub annotations: Vec<Annotation>,

    /// Parameters for this line
    pub parameters: Parameters,

    /// Raw tokens for precise source reconstruction
    pub tokens: ScannerTokenSequence,
}

/// Link element for external references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// Link target (URL, file path, etc.)
    pub target: String,

    /// Link text content (can contain formatting)
    pub content: Vec<TextTransform>,

    /// Additional link attributes (title, class, etc.)
    pub attributes: HashMap<String, String>,

    /// Annotations attached to this link
    pub annotations: Vec<Annotation>,

    /// Parameters for this link
    pub parameters: Parameters,

    /// Raw tokens for precise positioning
    pub tokens: ScannerTokenSequence,
}

/// Reference element for document elements (citations, cross-refs, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reference {
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

// Implement TxxtElement for TextSpan
impl TxxtElement for TextSpan {
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

impl SpanElement for TextSpan {
    fn text_content(&self) -> String {
        self.tokens.text()
    }
}

// Implement TxxtElement for TextLine
impl TxxtElement for TextLine {
    fn element_type(&self) -> ElementType {
        ElementType::Line
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

// Implement business logic from existing AST

impl TextSpan {
    /// Get the text content by concatenating token content
    pub fn content(&self) -> String {
        self.tokens.text()
    }

    /// Create a simple text span from a string (for testing/convenience)
    ///
    /// Creates synthetic tokens with dummy positions for testing/backwards compatibility.
    pub fn simple(content: &str) -> Self {
        Self::simple_with_tokens(
            content,
            ScannerTokenSequence {
                tokens: vec![ScannerToken::Text {
                    content: content.to_string(),
                    span: crate::cst::SourceSpan {
                        start: crate::cst::Position { row: 0, column: 0 },
                        end: crate::cst::Position {
                            row: 0,
                            column: content.len(),
                        },
                    },
                }],
            },
        )
    }

    /// Create a text span with source tokens
    ///
    /// # Arguments
    /// * `content` - The text content (for validation/debugging)
    /// * `tokens` - Source token sequence from parent HighLevelToken
    ///
    /// # Panics
    /// Panics if tokens are empty while content is non-empty, indicating a bug
    /// in token extraction. All callers must ensure tokens match content.
    pub fn simple_with_tokens(content: &str, tokens: ScannerTokenSequence) -> Self {
        // Validate that tokens are provided when content is non-empty
        // This is a safety check - if it fails, it indicates a bug in token extraction
        assert!(
            !tokens.tokens.is_empty() || content.is_empty(),
            "BUG: simple_with_tokens called with empty tokens but non-empty content: {:?}",
            content
        );

        Self {
            tokens,
            annotations: Vec::new(),
            parameters: Parameters::new(),
        }
    }
}

impl TextLine {
    /// Get all spans within this line
    pub fn span_elements(&self) -> &[TextTransform] {
        &self.spans
    }

    /// Get the complete text content of the line
    pub fn line_content(&self) -> String {
        self.spans
            .iter()
            .map(|span| span.text_content())
            .collect::<Vec<_>>()
            .join("")
    }
}
