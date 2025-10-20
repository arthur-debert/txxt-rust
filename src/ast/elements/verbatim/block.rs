//! Verbatim Block Element
//!
//! Verbatim blocks that preserve content exactly without TXXT parsing.

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    scanner_tokens::ScannerTokenSequence,
};

use super::super::{
    core::{BlockElement, ContainerElement, ElementType, HeaderedBlock, TxxtElement},
    inlines::TextTransform,
};

use super::ignore_container::IgnoreContainer;

/// Verbatim block - content that bypasses all TXXT parsing
///
/// TODO: Migrate existing verbatim logic from src/ast/blocks.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbatimBlock {
    /// Title content with inline formatting support
    pub title: Vec<TextTransform>,

    /// Verbatim content preserved exactly using IgnoreContainer
    pub content: IgnoreContainer,

    /// Mandatory label for format identification
    pub label: String,

    /// Type of verbatim block (in-flow vs stretched)
    pub verbatim_type: VerbatimType,

    /// Parameters from verbatim block declaration
    pub parameters: Parameters,

    /// Annotations attached to this verbatim block
    pub annotations: Vec<Annotation>,

    /// Raw tokens for source reconstruction
    pub tokens: ScannerTokenSequence,
}

/// Types of verbatim blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerbatimType {
    /// In-flow verbatim (inline with regular content)
    InFlow,

    /// Stretched verbatim (separate block with clear boundaries)
    Stretched,
}

impl TxxtElement for VerbatimBlock {
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

impl BlockElement for VerbatimBlock {
    fn content_summary(&self) -> String {
        format!("Verbatim block ({})", self.label)
    }
}

impl HeaderedBlock for VerbatimBlock {
    fn header_text(&self) -> String {
        self.title_text()
    }

    fn tail_container(&self) -> Option<&dyn ContainerElement> {
        Some(&self.content)
    }
}

impl VerbatimBlock {
    /// Create a new verbatim block
    pub fn new(
        title: Vec<TextTransform>,
        content: IgnoreContainer,
        label: String,
        verbatim_type: VerbatimType,
        parameters: Parameters,
        annotations: Vec<Annotation>,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            title,
            content,
            label,
            verbatim_type,
            parameters,
            annotations,
            tokens,
        }
    }

    /// Get the title text content
    pub fn title_text(&self) -> String {
        self.title
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get the verbatim content as text
    pub fn content_text(&self) -> String {
        self.content.content_text()
    }

    /// Check if this is an in-flow verbatim block
    pub fn is_in_flow(&self) -> bool {
        matches!(self.verbatim_type, VerbatimType::InFlow)
    }

    /// Check if this is a stretched verbatim block
    pub fn is_stretched(&self) -> bool {
        matches!(self.verbatim_type, VerbatimType::Stretched)
    }

    /// Get the label for syntax highlighting or tooling
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Check if the title is empty (minimal form)
    pub fn has_title(&self) -> bool {
        !self.title.is_empty() && !self.title_text().trim().is_empty()
    }
}
