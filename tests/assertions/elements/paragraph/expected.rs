//! Paragraph element expected value structs for assertions.

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
