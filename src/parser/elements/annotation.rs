//! Annotation Element Parser
//!
//! Annotations are metadata elements that attach to nearby content using
//! proximity rules. They provide a flexible system for adding structured
//! metadata to TXXT documents.

/// Annotation parser implementation
pub struct AnnotationParser;

impl AnnotationParser {
    /// Parse tokens into an annotation AST node
    pub fn parse(_tokens: &[()]) -> Result<(), AnnotationParseError> {
        // TODO: Implement annotation parsing
        Err(AnnotationParseError::NotImplemented)
    }
}

/// Annotation parsing errors
#[derive(Debug, Clone)]
pub enum AnnotationParseError {
    NotImplemented,
}

impl std::fmt::Display for AnnotationParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnotationParseError::NotImplemented => write!(f, "Annotation parser not implemented"),
        }
    }
}

impl std::error::Error for AnnotationParseError {}
