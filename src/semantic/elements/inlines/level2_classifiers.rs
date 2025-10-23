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

    // ============================================================================
    // Comprehensive Unit Tests for Type Classification
    // ============================================================================

    #[test]
    fn test_formatting_classifier_all_types() {
        let classifier = FormattingClassifier;
        let test_cases = vec![
            ("bold", InlineType::Bold),
            ("italic", InlineType::Italic),
            ("code", InlineType::Code),
            ("math", InlineType::Math),
        ];

        for (matcher_name, expected_type) in test_cases {
            let span = create_span(matcher_name, vec![create_text("content")]);
            let result = classifier.classify(&span).unwrap();
            assert_eq!(
                result, expected_type,
                "Failed for matcher: {}",
                matcher_name
            );
        }
    }

    #[test]
    fn test_formatting_classifier_unknown_matcher_error() {
        let classifier = FormattingClassifier;
        let span = create_span("unknown", vec![create_text("content")]);

        let result = classifier.classify(&span);
        assert!(result.is_err());
        assert!(matches!(result, Err(InlineParseError::InvalidStructure(_))));
    }

    #[test]
    fn test_reference_classifier_all_types() {
        let classifier = ReferenceTypeClassifier::new();

        let test_cases = vec![
            // URLs
            ("https://example.com", InlineType::Url),
            ("http://test.org", InlineType::Url),
            ("example.com", InlineType::Url),
            ("user@domain.com", InlineType::Url),
            // Citations (simple cases - complex parsing is CitationProcessor's job)
            ("@smith2023", InlineType::Citation),
            ("@doe2024, p. 45", InlineType::Citation),
            // Note: Multiple citations like "@key1; @key2" are handled at processor level
            // Sections
            ("#3", InlineType::Section),
            ("#2.1", InlineType::Section),
            ("#-1.2", InlineType::Section),
            // Footnotes (naked numerical)
            ("1", InlineType::Footnote),
            ("42", InlineType::Footnote),
            ("999", InlineType::Footnote),
            // Files
            ("./file.txt", InlineType::File),
            ("../dir/file.txt", InlineType::File),
            ("/absolute/path", InlineType::File),
            // TK
            ("TK", InlineType::ToComeTK),
            ("tk", InlineType::ToComeTK),
            ("TK-1", InlineType::ToComeTK),
            ("TK-someword", InlineType::ToComeTK),
            // Not Sure
            ("some-ambiguous-content", InlineType::NotSure),
        ];

        for (content, expected_type) in test_cases {
            let span = create_span("reference", vec![create_text(content)]);
            let result = classifier.classify(&span).unwrap();
            assert_eq!(
                result, expected_type,
                "Failed to classify '{}' as {:?}",
                content, expected_type
            );
        }
    }

    #[test]
    fn test_reference_classifier_empty_content_error() {
        let classifier = ReferenceTypeClassifier::new();
        let span = create_span("reference", vec![create_text("   ")]); // Only whitespace

        let result = classifier.classify(&span);
        assert!(result.is_err());
        assert!(matches!(result, Err(InlineParseError::EmptyContent(_))));
    }

    #[test]
    fn test_reference_classifier_multiple_tokens_concatenated() {
        // Test that multiple text tokens are properly concatenated
        let classifier = ReferenceTypeClassifier::new();
        let span = create_span(
            "reference",
            vec![create_text("@smith"), create_text("2023")],
        );

        let result = classifier.classify(&span).unwrap();
        assert_eq!(result, InlineType::Citation);
    }

    #[test]
    fn test_reference_classifier_ignores_non_text_tokens() {
        // Test that non-text tokens are filtered out during classification
        let classifier = ReferenceTypeClassifier::new();

        let mut tokens = vec![create_text("@citation")];
        // Add a non-text token (it should be filtered out)
        tokens.push(ScannerToken::Newline {
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 1 },
            },
        });

        let span = create_span("reference", tokens);
        let result = classifier.classify(&span).unwrap();
        assert_eq!(result, InlineType::Citation); // Should still recognize as citation
    }

    #[test]
    fn test_reference_classifier_case_sensitivity() {
        let classifier = ReferenceTypeClassifier::new();

        // TK should be case-insensitive
        let span_upper = create_span("reference", vec![create_text("TK")]);
        let span_lower = create_span("reference", vec![create_text("tk")]);

        assert_eq!(
            classifier.classify(&span_upper).unwrap(),
            InlineType::ToComeTK
        );
        assert_eq!(
            classifier.classify(&span_lower).unwrap(),
            InlineType::ToComeTK
        );
    }

    #[test]
    fn test_reference_classifier_precedence_order() {
        // Test that classifier follows correct precedence:
        // TK > Citation > Section > URL > File > Footnote > NotSure

        let classifier = ReferenceTypeClassifier::new();

        // Ambiguous case: "1" could be footnote or part of URL, but should be footnote
        let span = create_span("reference", vec![create_text("1")]);
        assert_eq!(classifier.classify(&span).unwrap(), InlineType::Footnote);

        // URL should take precedence over generic content
        let span = create_span("reference", vec![create_text("example.com")]);
        assert_eq!(classifier.classify(&span).unwrap(), InlineType::Url);

        // Citation (starts with @) has high precedence
        let span = create_span("reference", vec![create_text("@something")]);
        assert_eq!(classifier.classify(&span).unwrap(), InlineType::Citation);
    }

    #[test]
    fn test_reference_classifier_with_fragments() {
        let classifier = ReferenceTypeClassifier::new();

        // URLs with fragments should still be classified as URLs
        let span = create_span(
            "reference",
            vec![create_text("https://example.com#section")],
        );
        assert_eq!(classifier.classify(&span).unwrap(), InlineType::Url);

        // Files with sections should still be classified as files
        let span = create_span("reference", vec![create_text("./file.txt#section")]);
        assert_eq!(classifier.classify(&span).unwrap(), InlineType::File);
    }
}
