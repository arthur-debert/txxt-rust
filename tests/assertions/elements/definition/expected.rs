//! Definition element expected value structs for assertions.

use txxt::ast::elements::core::ElementType;

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
