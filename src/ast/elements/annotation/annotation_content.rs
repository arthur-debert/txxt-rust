//! Annotations: metadata that attaches to document elements
//!
//! This module defines the annotation system that provides rich metadata
//! capabilities for document elements, supporting automated tooling including
//! reviews, comments, and documentation workflows.
//!
//! src/parser/mod.rs has the full architecture overview.

use serde::{Deserialize, Serialize};

use super::super::{
    blocks::Block, components::parameters::Parameters, formatting::inlines::Inline,
};
use crate::cst::ScannerTokenSequence;

/// Annotation - metadata that attaches to document elements
///
/// Annotations are not part of the document structure per se, but rather
/// metadata that attaches to nodes based on proximity rules. They support
/// automated tooling including reviews, comments, and documentation workflows.
///
/// Attachment rules (applied during document assembly phase):
/// 1. Document start → attach to document itself
/// 2. Before element → attach to following element (blank lines ignored)
/// 3. Last in level/group → attach to parent node
///
/// Examples:
/// ```txxt
/// :: spec :: This is the txxt spec       // Attaches to document
/// :: author :: arthur
///
/// :: note :: This is important           // Attaches to following paragraph
/// This paragraph gets the note annotation.
///
/// 1. Session Title
///     Content here
///     :: comment :: Final comment        // Attaches to Session (parent)
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    /// Annotation label/type (e.g., "note", "author", "spec")
    pub label: String,

    /// Optional parameters in key=value format
    /// Example: :: warning:severity=high :: Content
    /// Supports ref=, id=, severity=, category= and other metadata
    pub parameters: Parameters,

    /// Annotation content (can be rich text with formatting)
    pub content: AnnotationContent,

    /// Raw tokens for source reconstruction and positioning
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
    Inline(Vec<Inline>),

    /// Block content for multiline annotations
    /// Example:
    /// ```txxt
    /// :: warning ::
    ///     This is a complex warning
    ///     with multiple lines.
    /// ```
    Block(Vec<Block>),

    /// Empty content (legal but rare)
    /// Example: :: empty ::
    Empty,
}

/// Annotation attachment context
///
/// Provides information about where and how an annotation was attached
/// during the assembly phase. Useful for debugging and tooling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationAttachment {
    /// How this annotation was attached
    pub attachment_rule: AttachmentRule,

    /// Source position where annotation appeared
    pub source_position: ScannerTokenSequence,

    /// Target element information (for debugging)
    pub target_info: Option<String>,
}

/// Rules for annotation attachment during assembly
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttachmentRule {
    /// Attached to document (appeared at document start)
    DocumentStart,

    /// Attached to following element
    BeforeElement,

    /// Attached to parent (was last in its level/group)
    Parent,

    /// Could not be attached (orphaned)
    Orphaned,
}

/// Annotation validation and processing utilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationProcessor {
    /// Whether to validate reserved namespaces
    pub validate_namespaces: bool,

    /// Whether to allow custom annotation types
    pub allow_custom_types: bool,

    /// Known annotation types for validation
    pub known_types: Vec<String>,
}

impl Annotation {
    /// Extract the local label (without namespace)
    pub fn local_label(&self) -> &str {
        if let Some(dot_pos) = self.label.rfind('.') {
            &self.label[dot_pos + 1..]
        } else {
            &self.label
        }
    }

    /// Check if this annotation has a specific namespace
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.namespace.as_deref() == Some(namespace)
    }

    /// Check if this annotation is reserved (txxt.* namespace)
    pub fn is_reserved(&self) -> bool {
        self.has_namespace("txxt")
    }

    /// Get parameter value by key
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }

    /// Get the reference identifier (ref or id parameter) for named anchors
    pub fn reference_id(&self) -> Option<&String> {
        self.parameters.reference_id()
    }

    /// Check if this annotation can be referenced by name
    pub fn is_referenceable(&self) -> bool {
        self.parameters.is_referenceable()
    }

    /// Get annotation severity (for warnings, errors)
    pub fn severity(&self) -> Option<&String> {
        self.parameters.severity()
    }

    /// Get annotation category
    pub fn category(&self) -> Option<&String> {
        self.parameters.category()
    }

    /// Check if this is a specific annotation type
    pub fn is_type(&self, annotation_type: &str) -> bool {
        self.local_label() == annotation_type
    }

    /// Create a new annotation with minimal information
    pub fn new(label: String, content: AnnotationContent) -> Self {
        Self {
            label,
            parameters: Parameters::new(),
            content,
            tokens: ScannerTokenSequence::new(),
            namespace: None,
        }
    }
}

impl AnnotationContent {
    /// Check if the annotation content is empty
    pub fn is_empty(&self) -> bool {
        match self {
            AnnotationContent::Empty => true,
            AnnotationContent::Inline(inlines) => inlines.is_empty(),
            AnnotationContent::Block(blocks) => blocks.is_empty(),
        }
    }

    /// Extract plain text content (for simple cases)
    pub fn as_text(&self) -> Option<String> {
        match self {
            AnnotationContent::Inline(inlines) => {
                if inlines.len() == 1 {
                    // Would need actual inline text extraction implementation
                    Some("text content".to_string()) // Placeholder
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if this is inline content
    pub fn is_inline(&self) -> bool {
        matches!(self, AnnotationContent::Inline(_))
    }

    /// Check if this is block content
    pub fn is_block(&self) -> bool {
        matches!(self, AnnotationContent::Block(_))
    }
}

impl Default for AnnotationProcessor {
    fn default() -> Self {
        Self {
            validate_namespaces: true,
            allow_custom_types: true,
            known_types: vec![
                "note".to_string(),
                "warning".to_string(),
                "error".to_string(),
                "info".to_string(),
                "todo".to_string(),
                "fixme".to_string(),
                "author".to_string(),
                "date".to_string(),
                "title".to_string(),
                "spec".to_string(),
                "version".to_string(),
                "comment".to_string(),
                "review".to_string(),
            ],
        }
    }
}

impl AnnotationProcessor {
    /// Validate an annotation according to processor rules
    pub fn validate(&self, annotation: &Annotation) -> Result<(), AnnotationError> {
        // Validate namespace restrictions
        if self.validate_namespaces && annotation.is_reserved() {
            return Err(AnnotationError::ReservedNamespace(annotation.label.clone()));
        }

        // Validate known types
        if !self.allow_custom_types
            && !self
                .known_types
                .contains(&annotation.local_label().to_string())
        {
            return Err(AnnotationError::UnknownType(
                annotation.local_label().to_string(),
            ));
        }

        // Validate parameters
        if let Some(severity) = annotation.severity() {
            match severity.as_str() {
                "low" | "medium" | "high" | "critical" => {}
                _ => return Err(AnnotationError::InvalidSeverity(severity.clone())),
            }
        }

        Ok(())
    }

    /// Process annotation and extract semantic information
    pub fn process(&self, annotation: &Annotation) -> AnnotationInfo {
        AnnotationInfo {
            local_label: annotation.local_label().to_string(),
            namespace: annotation.namespace.clone(),
            is_reserved: annotation.is_reserved(),
            reference_id: annotation.reference_id().cloned(),
            severity: annotation.severity().cloned(),
            category: annotation.category().cloned(),
            is_referenceable: annotation.is_referenceable(),
            content_type: match &annotation.content {
                AnnotationContent::Inline(_) => "inline".to_string(),
                AnnotationContent::Block(_) => "block".to_string(),
                AnnotationContent::Empty => "empty".to_string(),
            },
        }
    }
}

/// Processed annotation information
///
/// Contains extracted semantic information from annotations for easy access
/// by tooling and processing systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationInfo {
    /// Local label (without namespace)
    pub local_label: String,

    /// Namespace (if any)
    pub namespace: Option<String>,

    /// Whether this is a reserved annotation
    pub is_reserved: bool,

    /// Reference identifier (if referenceable)
    pub reference_id: Option<String>,

    /// Severity level (if applicable)
    pub severity: Option<String>,

    /// Category classification
    pub category: Option<String>,

    /// Whether this annotation can be referenced
    pub is_referenceable: bool,

    /// Type of content (inline, block, empty)
    pub content_type: String,
}

/// Annotation-related errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationError {
    /// Use of reserved namespace
    ReservedNamespace(String),

    /// Unknown annotation type
    UnknownType(String),

    /// Invalid severity value
    InvalidSeverity(String),

    /// Annotation attachment failed
    AttachmentFailed(String),

    /// Invalid annotation format
    InvalidFormat(String),

    /// Custom validation error
    Custom(String),
}

impl From<super::annotation_block::AnnotationBlock> for Annotation {
    fn from(block: super::annotation_block::AnnotationBlock) -> Self {
        let content = match block.content {
            super::annotation_block::AnnotationContent::Inline(transforms) => {
                AnnotationContent::Inline(
                    transforms
                        .into_iter()
                        .map(|t| t.to_inline())
                        .collect(),
                )
            }
            super::annotation_block::AnnotationContent::Block(container) => {
                AnnotationContent::Block(
                    container
                        .content
                        .into_iter()
                        .map(|c| c.into())
                        .collect(),
                )
            }
        };

        Self {
            label: block.label,
            parameters: block.parameters,
            content,
            tokens: block.tokens,
            namespace: block.namespace,
        }
    }
}
