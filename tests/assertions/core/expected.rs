//! Core expected value structs for AST element assertions.
//!
//! This module contains core expected structs that are used across multiple
//! element types or are fundamental to the assertion framework.

use txxt::ast::elements::core::ElementType;

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
