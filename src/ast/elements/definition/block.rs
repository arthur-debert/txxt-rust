//! Definition Block Element
//!
//! Definition blocks for term-definition pairs.

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    scanner_tokens::ScannerTokenSequence,
};

use super::super::{
    containers::ContentContainer,
    core::{BlockElement, ContainerElement, ElementType, HeaderedBlock, TxxtElement},
    inlines::TextTransform,
};

/// Definition block - term and definition pairs
///
/// TODO: Migrate existing definition logic from src/ast/blocks.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionBlock {
    /// The term being defined
    pub term: DefinitionTerm,

    /// Definition content (indented)
    pub content: ContentContainer,

    /// Parameters for metadata including ref= for named anchors
    pub parameters: Parameters,

    /// Annotations attached to this definition
    pub annotations: Vec<Annotation>,

    /// Raw tokens for source reconstruction
    pub tokens: ScannerTokenSequence,
}

/// Term part of a definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionTerm {
    /// Term content (inline elements for formatting support)
    pub content: Vec<TextTransform>,

    /// Raw tokens for exact positioning
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for DefinitionBlock {
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

impl BlockElement for DefinitionBlock {
    fn content_summary(&self) -> String {
        format!("Definition: {}", self.term.text_content())
    }
}

impl HeaderedBlock for DefinitionBlock {
    fn header_text(&self) -> String {
        self.term.text_content()
    }

    fn tail_container(&self) -> Option<&dyn ContainerElement> {
        Some(&self.content)
    }
}

impl TxxtElement for DefinitionTerm {
    fn element_type(&self) -> ElementType {
        ElementType::Line // Definition terms are line elements
    }

    fn tokens(&self) -> &ScannerTokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &[] // Definition terms don't have direct annotations
    }

    fn parameters(&self) -> &Parameters {
        // Definition terms don't have parameters - use a static empty instance
        use std::sync::OnceLock;
        static EMPTY_PARAMS: OnceLock<Parameters> = OnceLock::new();
        EMPTY_PARAMS.get_or_init(Parameters::default)
    }
}

impl DefinitionBlock {
    /// Create a new definition block
    pub fn new(
        term: DefinitionTerm,
        content: ContentContainer,
        parameters: Parameters,
        annotations: Vec<Annotation>,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            term,
            content,
            parameters,
            annotations,
            tokens,
        }
    }

    /// Get the term text
    pub fn term_text(&self) -> String {
        self.term.text_content()
    }

    /// Check if the definition content is empty
    pub fn is_content_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Get the reference identifier if this definition can be referenced
    pub fn reference_id(&self) -> Option<&str> {
        self.parameters.reference_id().map(|s| s.as_str())
    }
}

impl DefinitionTerm {
    /// Create a new definition term
    pub fn new(content: Vec<TextTransform>, tokens: ScannerTokenSequence) -> Self {
        Self { content, tokens }
    }

    /// Get the text content of the term
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if the term is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty() || self.text_content().trim().is_empty()
    }
}
