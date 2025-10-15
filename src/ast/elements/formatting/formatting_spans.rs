//! Formatting Span Elements
//!
//! Text formatting elements like emphasis, strong emphasis, code, and math spans.
//! These represent inline formatting that can be nested and combined.

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    tokens::TokenSequence,
};

use super::super::core::{ElementType, SpanElement, TxxtElement};
use super::super::inlines::{TextSpan, TextTransform};

/// Bold span - strong emphasis formatting
///
/// Typically renders as bold text. Can contain nested formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoldSpan {
    /// Nested text transforms within the bold span
    pub content: Vec<TextTransform>,

    /// Annotations attached to this span
    pub annotations: Vec<Annotation>,

    /// Parameters for this span
    pub parameters: Parameters,

    /// Raw tokens for precise positioning
    pub tokens: TokenSequence,
}

/// Italic span - emphasis formatting
///
/// Typically renders as italic text. Can contain nested formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItalicSpan {
    /// Nested text transforms within the italic span
    pub content: Vec<TextTransform>,

    /// Annotations attached to this span
    pub annotations: Vec<Annotation>,

    /// Parameters for this span
    pub parameters: Parameters,

    /// Raw tokens for precise positioning
    pub tokens: TokenSequence,
}

/// Code span - inline code formatting
///
/// Monospace formatting for code. Cannot contain nested formatting by design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeSpan {
    /// Plain text content (no nested formatting allowed)
    pub content: TextSpan,

    /// Annotations attached to this span
    pub annotations: Vec<Annotation>,

    /// Parameters for this span
    pub parameters: Parameters,

    /// Raw tokens for precise positioning
    pub tokens: TokenSequence,
}

/// Math span - mathematical expression formatting
///
/// Mathematical expressions using appropriate rendering. Cannot contain nested formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MathSpan {
    /// Mathematical expression content (no nested formatting allowed)
    pub content: TextSpan,

    /// Annotations attached to this span
    pub annotations: Vec<Annotation>,

    /// Parameters for this span
    pub parameters: Parameters,

    /// Raw tokens for precise positioning
    pub tokens: TokenSequence,
}

// Implement TxxtElement for all formatting spans

impl TxxtElement for BoldSpan {
    fn element_type(&self) -> ElementType {
        ElementType::Span
    }

    fn tokens(&self) -> &TokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl SpanElement for BoldSpan {
    fn text_content(&self) -> String {
        self.content
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }

    fn is_formatted(&self) -> bool {
        true
    }
}

impl TxxtElement for ItalicSpan {
    fn element_type(&self) -> ElementType {
        ElementType::Span
    }

    fn tokens(&self) -> &TokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl SpanElement for ItalicSpan {
    fn text_content(&self) -> String {
        self.content
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }

    fn is_formatted(&self) -> bool {
        true
    }
}

impl TxxtElement for CodeSpan {
    fn element_type(&self) -> ElementType {
        ElementType::Span
    }

    fn tokens(&self) -> &TokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl SpanElement for CodeSpan {
    fn text_content(&self) -> String {
        self.content.text_content()
    }

    fn is_formatted(&self) -> bool {
        true
    }
}

impl TxxtElement for MathSpan {
    fn element_type(&self) -> ElementType {
        ElementType::Span
    }

    fn tokens(&self) -> &TokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl SpanElement for MathSpan {
    fn text_content(&self) -> String {
        self.content.text_content()
    }

    fn is_formatted(&self) -> bool {
        true
    }
}

// Implement constructors and helper methods

impl BoldSpan {
    /// Create a new bold span
    pub fn new(
        content: Vec<TextTransform>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: TokenSequence,
    ) -> Self {
        Self {
            content,
            annotations,
            parameters,
            tokens,
        }
    }
}

impl ItalicSpan {
    /// Create a new italic span
    pub fn new(
        content: Vec<TextTransform>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: TokenSequence,
    ) -> Self {
        Self {
            content,
            annotations,
            parameters,
            tokens,
        }
    }
}

impl CodeSpan {
    /// Create a new code span
    pub fn new(
        content: TextSpan,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: TokenSequence,
    ) -> Self {
        Self {
            content,
            annotations,
            parameters,
            tokens,
        }
    }
}

impl MathSpan {
    /// Create a new math span
    pub fn new(
        content: TextSpan,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: TokenSequence,
    ) -> Self {
        Self {
            content,
            annotations,
            parameters,
            tokens,
        }
    }
}
