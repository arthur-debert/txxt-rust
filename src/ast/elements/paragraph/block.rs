//! Paragraph Block Element
//!
//! Paragraphs are the basic unit of flowing text, containing inline content
//! and serving as leaf blocks (cannot contain other blocks).

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    scanner_tokens::ScannerTokenSequence,
};

use super::super::{
    core::{BlockElement, ElementType, TxxtElement},
    inlines::TextTransform,
};

/// Paragraph block - the basic unit of flowing text
///
/// From `docs/specs/core/terminology.txxt`:
/// "Paragraph: One or more text lines"
///
/// Paragraphs contain inline content and are leaf blocks. They represent
/// a single logical unit of text that flows together, terminated by a
/// blank line or structural change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphBlock {
    /// Paragraph content with inline formatting
    pub content: Vec<TextTransform>,

    /// Annotations attached to this paragraph
    pub annotations: Vec<Annotation>,

    /// Parameters for this paragraph
    pub parameters: Parameters,

    /// Raw tokens for precise source reconstruction
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for ParagraphBlock {
    fn element_type(&self) -> ElementType {
        ElementType::Block
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

impl BlockElement for ParagraphBlock {
    fn can_contain_blocks(&self) -> bool {
        false // Paragraphs are leaf blocks
    }

    fn content_summary(&self) -> String {
        let text = self.text_content();
        if text.len() > 50 {
            format!("{}...", &text[..47])
        } else {
            text
        }
    }
}

impl ParagraphBlock {
    /// Create a new paragraph block
    pub fn new(
        content: Vec<TextTransform>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            content,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Get the plain text content of the paragraph
    /// (Migrated from existing Paragraph logic)
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if the paragraph is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty() || self.text_content().trim().is_empty()
    }

    /// Get the number of text transforms in this paragraph
    pub fn transform_count(&self) -> usize {
        self.content.len()
    }
}
