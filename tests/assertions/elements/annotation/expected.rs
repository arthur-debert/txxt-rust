//! Annotation element expected value structs for assertions.

use std::collections::HashMap;

use txxt::ast::elements::annotation::AnnotationContent;

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
