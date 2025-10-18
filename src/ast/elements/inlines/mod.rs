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
    annotation::annotation_content::Annotation,
    components::parameters::Parameters,
    references::reference_types::ReferenceTarget,
    scanner_tokens::{ScannerToken, ScannerTokenSequence},
};

use super::core::{ElementType, SpanElement, TxxtElement};

// Re-export inline types from new functional modules
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

/// Text transform layer - uniform processing for all text content
///
/// This is the key architectural innovation from the existing AST.
/// Every piece of text goes through this transform layer for:
/// - Uniform processing across all text contexts
/// - Extensibility for new transform types
/// - Composability for nested formatting
/// - Performance optimization during rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextTransform {
    /// Identity transform (plain text) - no formatting applied
    Identity(TextSpan),

    /// Emphasis transform - typically renders as italic
    Emphasis(Vec<TextTransform>),

    /// Strong emphasis transform - typically renders as bold
    Strong(Vec<TextTransform>),

    /// Inline code transform - monospace formatting
    Code(TextSpan),

    /// Math transform - mathematical expressions
    Math(TextSpan),

    /// Composed transform - for complex nested cases
    Composed(Vec<TextTransform>),

    /// Custom transform for extensibility
    Custom {
        /// Transform type name
        name: String,
        /// Transform parameters
        parameters: HashMap<String, String>,
        /// Nested transforms (if applicable)
        content: Vec<TextTransform>,
    },
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
    pub fn simple(content: &str) -> Self {
        Self {
            tokens: ScannerTokenSequence {
                tokens: vec![ScannerToken::Text {
                    content: content.to_string(),
                    span: crate::ast::scanner_tokens::SourceSpan {
                        start: crate::ast::scanner_tokens::Position { row: 0, column: 0 },
                        end: crate::ast::scanner_tokens::Position {
                            row: 0,
                            column: content.len(),
                        },
                    },
                }],
            },
            annotations: Vec::new(),
            parameters: Parameters::new(),
        }
    }
}

impl TextTransform {
    /// Extract the text content from this transform recursively
    /// (Migrated from existing TextTransform logic)
    pub fn text_content(&self) -> String {
        match self {
            TextTransform::Identity(text) => text.content(),
            TextTransform::Emphasis(transforms) => transforms
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
            TextTransform::Strong(transforms) => transforms
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
            TextTransform::Code(text) => text.content(),
            TextTransform::Math(text) => text.content(),
            TextTransform::Composed(transforms) => transforms
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
            TextTransform::Custom { content, .. } => content
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
        }
    }

    /// Check if this transform is an identity (no formatting applied)
    /// (Migrated from existing TextTransform logic)
    pub fn is_identity(&self) -> bool {
        matches!(self, TextTransform::Identity(_))
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
