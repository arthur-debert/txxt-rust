//! Verbatim element expected value structs for assertions.

use std::collections::HashMap;

use txxt::ast::elements::verbatim::VerbatimType;

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
