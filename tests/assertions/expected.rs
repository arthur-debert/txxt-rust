//! Expected value structs for AST element assertions.
//!
//! Each element type has an Expected struct with optional fields. Tests specify
//! only the properties they want to validate - unspecified fields are not checked.
//!
//! # Design Pattern
//!
//! All fields are `Option<T>`:
//! - `Some(value)` - Assert this property matches value
//! - `None` - Skip validation for this property
//!
//! # Example
//!
//! ```rust
//! use tests::assertions::expected::VerbatimExpected;
//!
//! // Test only the properties that matter
//! let expected = VerbatimExpected {
//!     label: Some("python"),
//!     line_count: Some(3),
//!     ..Default::default()  // All other fields: None (not validated)
//! };
//! ```

use std::collections::HashMap;

#[cfg(feature = "new-ast")]
use txxt::ast::elements::{
    annotation::AnnotationContent, core::ElementType, list::NumberingStyle, verbatim::VerbatimType,
};

// ============================================================================
// Paragraph Expected
// ============================================================================

/// Expected properties for paragraph validation.
#[derive(Default)]
pub struct ParagraphExpected<'a> {
    /// Exact text match (after whitespace normalization)
    pub text: Option<&'a str>,

    /// Text contains this substring
    pub text_contains: Option<&'a str>,

    /// Text matches this regex pattern
    pub text_matches: Option<&'a str>,

    /// Has formatting beyond plain text (Bold, Italic, Code, Math)
    pub has_formatting: Option<bool>,

    /// Number of annotations attached
    pub annotation_count: Option<usize>,

    /// Has annotation with this label
    pub has_annotation: Option<&'a str>,

    /// Has parameter with this key-value pair
    pub has_parameter: Option<(&'a str, &'a str)>,
}

// ============================================================================
// List Expected
// ============================================================================

/// Expected properties for list validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during Parser 2.1.2
pub struct ListExpected<'a> {
    /// List decoration style (Plain, Numerical, Alphabetical, Roman)
    pub style: Option<NumberingStyle>,

    /// Number of list items
    pub item_count: Option<usize>,

    /// Text content of each item (in order)
    pub item_text: Option<Vec<&'a str>>,

    /// Which items have nested containers (in order)
    pub has_nested: Option<Vec<bool>>,

    /// Number of annotations attached to list
    pub annotation_count: Option<usize>,

    /// Has annotation with this label
    pub has_annotation: Option<&'a str>,
}

// ============================================================================
// Definition Expected
// ============================================================================

/// Expected properties for definition validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during Parser 2.1.3
pub struct DefinitionExpected<'a> {
    /// Exact term text
    pub term: Option<&'a str>,

    /// Term contains this substring
    pub term_contains: Option<&'a str>,

    /// Number of elements in definition content
    pub content_count: Option<usize>,

    /// Definition has non-empty content
    pub has_content: Option<bool>,

    /// Child element types in content (in order)
    pub content_types: Option<Vec<ElementType>>,

    /// Number of annotations
    pub annotation_count: Option<usize>,

    /// Has annotation with this label
    pub has_annotation: Option<&'a str>,
}

// ============================================================================
// Session Expected
// ============================================================================

/// Expected properties for session validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during Parser 2.1.5
pub struct SessionExpected<'a> {
    /// Exact title text
    pub title: Option<&'a str>,

    /// Title contains this substring
    pub title_contains: Option<&'a str>,

    /// Exact numbering marker (e.g., "1.", "1.1.", "a.")
    pub numbering: Option<&'a str>,

    /// Session has any numbering
    pub is_numbered: Option<bool>,

    /// Number of children in session container
    pub child_count: Option<usize>,

    /// Contains at least one nested session
    pub has_subsession: Option<bool>,

    /// Child element types (in order)
    pub child_types: Option<Vec<ElementType>>,

    /// Number of annotations
    pub annotation_count: Option<usize>,
}

// ============================================================================
// Verbatim Expected
// ============================================================================

/// Expected properties for verbatim block validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during verbatim parsing
pub struct VerbatimExpected<'a> {
    /// Verbatim mode (InFlow or Stretched)
    pub mode: Option<VerbatimType>,

    /// Exact label value
    pub label: Option<&'a str>,

    /// Label starts with this prefix
    pub label_starts_with: Option<&'a str>,

    /// Exact title text
    pub title: Option<&'a str>,

    /// Title contains this substring
    pub title_contains: Option<&'a str>,

    /// Number of content lines (ignore_lines count)
    pub line_count: Option<usize>,

    /// Content contains this substring
    pub content_contains: Option<&'a str>,

    /// All parameters match these key-value pairs
    pub parameters: Option<HashMap<&'a str, &'a str>>,

    /// Has specific parameter key-value pair
    pub has_parameter: Option<(&'a str, &'a str)>,

    /// Number of annotations
    pub annotation_count: Option<usize>,
}

// ============================================================================
// Annotation Expected
// ============================================================================

/// Expected properties for annotation validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during annotation parsing
pub struct AnnotationExpected<'a> {
    /// Exact label value
    pub label: Option<&'a str>,

    /// Content type (Inline or Block)
    pub content_type: Option<AnnotationContentType>,

    /// Has non-empty content
    pub has_content: Option<bool>,

    /// Exact content text (for inline annotations)
    pub content_text: Option<&'a str>,

    /// Content contains this substring
    pub content_contains: Option<&'a str>,

    /// All parameters match
    pub parameters: Option<HashMap<&'a str, &'a str>>,

    /// Has specific parameter
    pub has_parameter: Option<(&'a str, &'a str)>,
}

/// Helper enum for annotation content type checking
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Will be used during annotation parsing
pub enum AnnotationContentType {
    Inline,
    Block,
    Empty,
}

impl AnnotationContentType {
    #[cfg(feature = "new-ast")]
    #[allow(dead_code)] // Will be used during annotation parsing
    pub fn matches(&self, content: &AnnotationContent) -> bool {
        match (self, content) {
            (AnnotationContentType::Inline, AnnotationContent::Inline(_)) => true,
            (AnnotationContentType::Block, AnnotationContent::Block(_)) => true,
            (AnnotationContentType::Empty, _) => {
                // Check if content is actually empty
                match content {
                    AnnotationContent::Inline(v) => v.is_empty(),
                    AnnotationContent::Block(c) => c.content.is_empty(),
                }
            }
            _ => false,
        }
    }
}

// ============================================================================
// Container Expected
// ============================================================================

/// Expected properties for content container validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during container parsing
pub struct ContentContainerExpected {
    /// Total number of elements
    pub element_count: Option<usize>,

    /// Element types in order
    pub element_types: Option<Vec<ElementType>>,

    /// Contains at least one element of this type
    pub has_element_type: Option<ElementType>,

    /// All children are this type
    pub all_same_type: Option<ElementType>,
}

/// Expected properties for session container validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during session parsing
pub struct SessionContainerExpected {
    /// Total number of elements
    pub element_count: Option<usize>,

    /// Element types in order
    pub element_types: Option<Vec<ElementType>>,

    /// Contains at least one nested session
    pub has_session: Option<bool>,

    /// Number of session blocks
    pub session_count: Option<usize>,
}

// ============================================================================
// Inline Content Expected
// ============================================================================

/// Expected properties for inline content validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during inline parsing
pub struct InlineContentExpected {
    /// Contains Bold spans
    pub has_bold: Option<bool>,

    /// Contains Italic spans
    pub has_italic: Option<bool>,

    /// Contains Code spans
    pub has_code: Option<bool>,

    /// Contains Math spans
    pub has_math: Option<bool>,

    /// Number of transforms
    pub transform_count: Option<usize>,
}

// ============================================================================
// Document Expected
// ============================================================================

/// Expected properties for document validation.
#[derive(Default)]
#[allow(dead_code)] // Will be used during document parsing
pub struct DocumentExpected<'a> {
    /// Number of top-level elements
    pub element_count: Option<usize>,

    /// Has title annotation in document metadata
    pub has_title: Option<bool>,

    /// Exact title text
    pub title: Option<&'a str>,

    /// Number of session blocks
    pub session_count: Option<usize>,

    /// Maximum nesting depth
    pub max_depth: Option<usize>,

    /// Has any document-level annotations
    pub has_annotations: Option<bool>,
}
