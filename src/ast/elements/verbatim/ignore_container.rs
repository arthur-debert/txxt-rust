//! Ignore Container  
//!
//! Ignore containers hold verbatim content that should not be parsed as TXXT.
//! Used exclusively for verbatim blocks to preserve formatting exactly.

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    scanner_tokens::ScannerTokenSequence,
};

use super::super::core::{ContainerElement, ContainerType, ElementType, TxxtElement};

/// Ignore container - container for verbatim content only
///
/// From `docs/specs/core/terminology.txxt`:
/// "Ignore Container: Can only hold ignore lines and blank lines.
/// Used for verbatim block content that should not be parsed as txxt."
///
/// Ignore containers follow the container architecture but hold only
/// ignore lines and blank lines. They maintain the consistent container
/// structure while preserving verbatim content exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgnoreContainer {
    /// Raw content lines preserved exactly  
    pub ignore_lines: Vec<IgnoreLine>,

    /// Blank lines within verbatim content
    pub blank_lines: Vec<super::super::core::BlankLine>,

    /// Annotations attached to this container
    pub annotations: Vec<Annotation>,

    /// Parameters for metadata (rare for ignore containers)
    pub parameters: Parameters,

    /// Source position information
    pub tokens: ScannerTokenSequence,
}

/// Ignore line - raw verbatim content preserved exactly
///
/// Ignore lines contain content that should not be parsed as TXXT.
/// They preserve exact spacing and formatting byte-for-byte.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgnoreLine {
    /// Raw content preserved byte-for-byte
    pub content: String,

    /// Source position information
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for IgnoreContainer {
    fn element_type(&self) -> ElementType {
        ElementType::Container
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

impl ContainerElement for IgnoreContainer {
    fn container_type(&self) -> ContainerType {
        ContainerType::Ignore
    }

    fn can_contain_sessions(&self) -> bool {
        false // Ignore containers only hold verbatim content
    }

    fn child_elements(&self) -> Vec<&dyn TxxtElement> {
        let mut elements: Vec<&dyn TxxtElement> = Vec::new();

        // Add ignore lines (they don't implement TxxtElement, so we skip them)
        // This is intentional - ignore lines are not semantic elements

        // Add blank lines
        for blank_line in &self.blank_lines {
            elements.push(blank_line as &dyn TxxtElement);
        }

        elements
    }
}

impl TxxtElement for IgnoreLine {
    fn element_type(&self) -> ElementType {
        ElementType::Line
    }

    fn tokens(&self) -> &ScannerTokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &[] // Ignore lines don't have annotations
    }

    fn parameters(&self) -> &Parameters {
        // Ignore lines don't have parameters - use a static empty instance
        use std::sync::OnceLock;
        static EMPTY_PARAMS: OnceLock<Parameters> = OnceLock::new();
        EMPTY_PARAMS.get_or_init(Parameters::default)
    }
}

impl IgnoreContainer {
    /// Create a new ignore container
    pub fn new(
        ignore_lines: Vec<IgnoreLine>,
        blank_lines: Vec<super::super::core::BlankLine>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            ignore_lines,
            blank_lines,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.ignore_lines.is_empty() && self.blank_lines.is_empty()
    }

    /// Get the total number of lines
    pub fn total_lines(&self) -> usize {
        self.ignore_lines.len() + self.blank_lines.len()
    }

    /// Get all content as a single string (for verbatim reconstruction)
    pub fn content_text(&self) -> String {
        // This would need proper line ordering based on tokens
        // For now, just concatenate ignore lines
        self.ignore_lines
            .iter()
            .map(|line| &line.content)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl IgnoreLine {
    /// Create a new ignore line
    pub fn new(content: String, tokens: ScannerTokenSequence) -> Self {
        Self { content, tokens }
    }

    /// Get the raw content
    pub fn content(&self) -> &str {
        &self.content
    }
}
