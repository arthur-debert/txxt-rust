//! Session element expected value structs for assertions.

use txxt::ast::elements::core::ElementType;

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
