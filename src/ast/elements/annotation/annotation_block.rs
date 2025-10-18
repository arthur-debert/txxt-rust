//! Annotation Block Element
//!
//! Annotation blocks provide metadata for document elements.
//! Migrated from src/ast/annotations.rs

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    scanner_tokens::ScannerTokenSequence,
};

use super::super::{
    containers::ContentContainer,
    core::{BlockElement, ElementType, TxxtElement},
    inlines::TextTransform,
};

/// Annotation block - metadata element
///
/// Annotations are metadata that attach to document elements based on proximity rules.
/// They support automated tooling including reviews, comments, and documentation workflows.
///
/// From the original annotation system:
/// - Document start → attach to document itself
/// - Before element → attach to following element (blank lines ignored)
/// - Last in level/group → attach to parent node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationBlock {
    /// Annotation label/type (e.g., "note", "author", "spec")
    pub label: String,

    /// Annotation content (inline or block)
    pub content: AnnotationContent,

    /// Parameters in key=value format (severity=high, ref=, id=, etc.)
    pub parameters: Parameters,

    /// Annotations attached to this annotation block (meta-annotations)
    pub annotations: Vec<Annotation>,

    /// Raw tokens for precise source reconstruction
    pub tokens: ScannerTokenSequence,

    /// Namespace information (if label contains dots)
    /// Example: "org.example.meta" → namespace="org.example", local_label="meta"
    pub namespace: Option<String>,
}

/// Content of an annotation
///
/// Annotations can contain either simple inline text or complex block content
/// for multiline annotations with indented blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationContent {
    /// Simple inline content
    /// Example: :: note :: This is a simple note
    Inline(Vec<TextTransform>),

    /// Block content for multiline annotations
    /// Example:
    /// ```txxt
    /// :: warning ::
    ///     This is a complex warning
    ///     with multiple lines.
    /// ```
    Block(ContentContainer),
}

impl TxxtElement for AnnotationBlock {
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

impl BlockElement for AnnotationBlock {
    fn content_summary(&self) -> String {
        format!("Annotation: {}", self.label)
    }
}

impl AnnotationBlock {
    /// Create a new annotation block
    pub fn new(
        label: String,
        content: AnnotationContent,
        parameters: Parameters,
        annotations: Vec<Annotation>,
        tokens: ScannerTokenSequence,
    ) -> Self {
        let namespace = if label.contains('.') {
            let parts: Vec<&str> = label.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                Some(parts[1].to_string())
            } else {
                None
            }
        } else {
            None
        };

        Self {
            label,
            content,
            parameters,
            annotations,
            tokens,
            namespace,
        }
    }

    /// Get the local label (without namespace)
    pub fn local_label(&self) -> &str {
        if self.namespace.is_some() {
            self.label.rsplit('.').next().unwrap_or(&self.label)
        } else {
            &self.label
        }
    }

    /// Check if this annotation has a namespace
    pub fn has_namespace(&self) -> bool {
        self.namespace.is_some()
    }

    /// Get the annotation content as text
    pub fn content_text(&self) -> String {
        match &self.content {
            AnnotationContent::Inline(transforms) => transforms
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
            AnnotationContent::Block(container) => {
                // This would need to be implemented based on container content
                format!("Block content with {} elements", container.len())
            }
        }
    }

    /// Check if this is an inline annotation
    pub fn is_inline(&self) -> bool {
        matches!(self.content, AnnotationContent::Inline(_))
    }

    /// Check if this is a block annotation
    pub fn is_block(&self) -> bool {
        matches!(self.content, AnnotationContent::Block(_))
    }
}
