//! # Level 2: Type Classifiers
//!
//! Implementations of TypeClassifier for determining specific inline element types
//! from matched spans. For formatting elements, the type is implied by the
//! delimiter. For references, pattern analysis using ReferenceClassifier is required.

use crate::ast::elements::references::reference_types::{ReferenceClassifier, SimpleReferenceType};
use crate::semantic::elements::inlines::pipeline::{InlineType, SpanMatch, TypeClassifier};
use crate::semantic::elements::inlines::InlineParseError;

/// Formatting type classifier
///
/// For formatting elements (bold, italic, code, math), the type is directly
/// implied by the delimiter that was matched. This classifier simply maps
/// matcher names to inline types.
pub struct FormattingClassifier;

impl TypeClassifier for FormattingClassifier {
    fn classify(&self, span: &SpanMatch) -> Result<InlineType, InlineParseError> {
        match span.matcher_name.as_str() {
            "bold" => Ok(InlineType::Bold),
            "italic" => Ok(InlineType::Italic),
            "code" => Ok(InlineType::Code),
            "math" => Ok(InlineType::Math),
            _ => Err(InlineParseError::InvalidStructure(format!(
                "Unknown formatting matcher: {}",
                span.matcher_name
            ))),
        }
    }
}

/// Reference type classifier
///
/// For reference elements (`[...]`), the content must be analyzed to determine
/// the specific reference type. This classifier uses the existing ReferenceClassifier
/// to detect URL, File, Citation, Footnote, Section, TK, and NotSure references.
pub struct ReferenceTypeClassifier {
    classifier: ReferenceClassifier,
}

impl ReferenceTypeClassifier {
    pub fn new() -> Self {
        Self {
            classifier: ReferenceClassifier::new(),
        }
    }
}

impl Default for ReferenceTypeClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeClassifier for ReferenceTypeClassifier {
    fn classify(&self, span: &SpanMatch) -> Result<InlineType, InlineParseError> {
        // Extract content from inner tokens
        let content = span
            .inner_tokens
            .iter()
            .filter_map(|token| match token {
                crate::cst::ScannerToken::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        if content.trim().is_empty() {
            return Err(InlineParseError::EmptyContent(
                "Reference content cannot be empty".to_string(),
            ));
        }

        // Use ReferenceClassifier to determine type
        let ref_type = self.classifier.classify(&content);

        // Map SimpleReferenceType to InlineType
        let inline_type = match ref_type {
            SimpleReferenceType::Url => InlineType::Url,
            SimpleReferenceType::Section => InlineType::Section,
            SimpleReferenceType::Footnote => InlineType::Footnote,
            SimpleReferenceType::Citation => InlineType::Citation,
            SimpleReferenceType::ToComeTK => InlineType::ToComeTK,
            SimpleReferenceType::File => InlineType::File,
            SimpleReferenceType::NotSure => InlineType::NotSure,
        };

        Ok(inline_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, ScannerToken, SourceSpan};

    fn create_text(content: &str) -> ScannerToken {
        ScannerToken::Text {
            content: content.to_string(),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position {
                    row: 0,
                    column: content.len(),
                },
            },
        }
    }

    fn create_span(matcher: &str, content_tokens: Vec<ScannerToken>) -> SpanMatch {
        SpanMatch {
            start: 0,
            end: content_tokens.len() + 2,
            matcher_name: matcher.to_string(),
            inner_tokens: content_tokens.clone(),
            full_tokens: content_tokens,
        }
    }

    #[test]
    fn test_formatting_classifier_bold() {
        let span = create_span("bold", vec![create_text("hello")]);
        let classifier = FormattingClassifier;

        let result = classifier.classify(&span);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InlineType::Bold);
    }

    #[test]
    fn test_reference_classifier_citation() {
        let span = create_span("reference", vec![create_text("@smith2023")]);
        let classifier = ReferenceTypeClassifier::new();

        let result = classifier.classify(&span);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InlineType::Citation);
    }

    #[test]
    fn test_reference_classifier_url() {
        let span = create_span("reference", vec![create_text("https://example.com")]);
        let classifier = ReferenceTypeClassifier::new();

        let result = classifier.classify(&span);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InlineType::Url);
    }

    #[test]
    fn test_reference_classifier_section() {
        let span = create_span("reference", vec![create_text("#3")]);
        let classifier = ReferenceTypeClassifier::new();

        let result = classifier.classify(&span);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InlineType::Section);
    }

    #[test]
    fn test_reference_classifier_footnote() {
        let span = create_span("reference", vec![create_text("1")]);
        let classifier = ReferenceTypeClassifier::new();

        let result = classifier.classify(&span);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InlineType::Footnote);
    }
}
