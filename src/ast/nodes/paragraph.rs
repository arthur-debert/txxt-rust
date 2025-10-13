//! Paragraph AST Nodes
//!
//! AST node definitions for paragraph elements - the fundamental text blocks
//! that contain inline content and form the basic unit of readable text.

use crate::ast::{parameters::Parameters, tokens::TokenSequence};

/// Paragraph AST node
///
/// Represents a paragraph element containing inline content with
/// optional parameters and annotations.
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// Inline content of the paragraph
    pub content: Vec<InlineContent>,
    /// Optional parameters attached to the paragraph
    pub parameters: Parameters,
    /// Token sequence for source reconstruction
    pub tokens: TokenSequence,
}

/// Placeholder for inline content
/// TODO: Replace with actual inline AST nodes once implemented
#[derive(Debug, Clone, PartialEq)]
pub enum InlineContent {
    /// Plain text content
    Text(String),
    /// Formatted content (emphasis, strong, etc.)
    Formatted(String),
}

impl Paragraph {
    /// Create a new paragraph node
    pub fn new(content: Vec<InlineContent>, parameters: Parameters, tokens: TokenSequence) -> Self {
        Self {
            content,
            parameters,
            tokens,
        }
    }

    /// Get the plain text content of the paragraph
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .map(|content| match content {
                InlineContent::Text(text) => text.clone(),
                InlineContent::Formatted(text) => text.clone(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if the paragraph has any parameters
    pub fn has_parameters(&self) -> bool {
        // TODO: Implement parameters check once Parameters structure is known
        false
    }
}
