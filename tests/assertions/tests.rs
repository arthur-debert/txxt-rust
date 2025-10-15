//! Tests for the assertion framework itself.
//!
//! These tests validate that:
//! - Assertions pass when properties match
//! - Assertions fail with helpful messages when properties don't match
//! - Optional fields work correctly (None skips validation)
//! - Shared validators behave consistently

#[cfg(test)]
#[cfg(feature = "new-ast")]
mod assertion_tests {
    use super::super::{assert_paragraph, ParagraphExpected};
    use txxt::ast::{
        elements::{
            containers::session::SessionContainerElement,
            core::ElementType,
            inlines::TextTransform,
            paragraph::ParagraphBlock,
        },
        parameters::Parameters,
        tokens::{Position, SourceSpan, Token, TokenSequence},
    };

    /// Helper to create a simple paragraph for testing
    fn make_test_paragraph(text: &str) -> SessionContainerElement {
        let text_transform = TextTransform::Identity(
            txxt::ast::elements::inlines::TextSpan {
                tokens: TokenSequence {
                    tokens: vec![Token::Text {
                        content: text.to_string(),
                        span: SourceSpan {
                            start: Position { row: 0, column: 0 },
                            end: Position {
                                row: 0,
                                column: text.len(),
                            },
                        },
                    }],
                },
                annotations: vec![],
                parameters: Parameters::new(),
            }
        );

        SessionContainerElement::Paragraph(ParagraphBlock {
            content: vec![text_transform],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        })
    }

    #[test]
    fn test_assert_paragraph_type_check_succeeds() {
        let element = make_test_paragraph("Test paragraph");

        // Should not panic - correct type
        let result = assert_paragraph(&element, ParagraphExpected::default());

        // Should return reference
        assert_eq!(result.element_type(), ElementType::Block);
    }

    #[test]
    #[should_panic(expected = "Element type assertion failed")]
    fn test_assert_paragraph_type_check_fails() {
        // Create a non-paragraph element
        let element = SessionContainerElement::BlankLine(
            txxt::ast::elements::core::BlankLine {
                tokens: TokenSequence::new(),
            }
        );

        // Should panic - wrong type
        assert_paragraph(&element, ParagraphExpected::default());
    }

    #[test]
    fn test_assert_paragraph_text_contains_succeeds() {
        let element = make_test_paragraph("This is a test paragraph");

        // Should not panic - contains substring
        assert_paragraph(&element, ParagraphExpected {
            text_contains: Some("test paragraph"),
            ..Default::default()
        });
    }

    #[test]
    #[should_panic(expected = "Text content validation failed")]
    fn test_assert_paragraph_text_contains_fails() {
        let element = make_test_paragraph("This is a test paragraph");

        // Should panic - doesn't contain substring
        assert_paragraph(&element, ParagraphExpected {
            text_contains: Some("missing text"),
            ..Default::default()
        });
    }

    #[test]
    fn test_assert_paragraph_optional_fields_skip_validation() {
        let element = make_test_paragraph("Any text here");

        // Should not panic - no validations specified (all None)
        assert_paragraph(&element, ParagraphExpected::default());

        // Should also work with partial specification
        assert_paragraph(&element, ParagraphExpected {
            text_contains: Some("Any"),
            // All other fields: None - not validated
            ..Default::default()
        });
    }

    #[test]
    fn test_assert_paragraph_annotation_count() {
        let element = make_test_paragraph("Text");

        // Should pass - has 0 annotations
        assert_paragraph(&element, ParagraphExpected {
            annotation_count: Some(0),
            ..Default::default()
        });
    }

    #[test]
    #[should_panic(expected = "Annotation count mismatch")]
    fn test_assert_paragraph_annotation_count_fails() {
        let element = make_test_paragraph("Text");

        // Should panic - expected 1 but has 0
        assert_paragraph(&element, ParagraphExpected {
            annotation_count: Some(1),
            ..Default::default()
        });
    }
}

#[cfg(test)]
mod validator_tests {
    use super::super::validators::*;
    use std::collections::HashMap;

    #[test]
    #[cfg(feature = "new-ast")]
    fn test_validate_parameters_succeeds() {
        use txxt::ast::elements::components::parameters::Parameters;
        
        let mut params = Parameters::new();
        params.insert("key1".to_string(), "value1".to_string());
        params.insert("key2".to_string(), "value2".to_string());

        let mut expected = HashMap::new();
        expected.insert("key1", "value1");
        expected.insert("key2", "value2");

        // Should not panic
        validate_parameters(&params, &expected);
    }

    #[test]
    #[should_panic(expected = "Parameter validation failed for key")]
    #[cfg(feature = "new-ast")]
    fn test_validate_parameters_value_mismatch() {
        use txxt::ast::elements::components::parameters::Parameters;
        
        let mut params = Parameters::new();
        params.insert("key1".to_string(), "wrong_value".to_string());

        let mut expected = HashMap::new();
        expected.insert("key1", "expected_value");

        // Should panic - value mismatch
        validate_parameters(&params, &expected);
    }

    #[test]
    #[should_panic(expected = "missing key")]
    #[cfg(feature = "new-ast")]
    fn test_validate_parameters_missing_key() {
        use txxt::ast::elements::components::parameters::Parameters;
        
        let params = Parameters::new(); // Empty

        let mut expected = HashMap::new();
        expected.insert("missing_key", "value");

        // Should panic - key not found
        validate_parameters(&params, &expected);
    }
}

// ============================================================================
// Documentation Examples
// ============================================================================

/// # Example: Basic Paragraph Assertion
///
/// ```rust,ignore
/// let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple").unwrap();
/// let para = parse_paragraph(&corpus.source_text).unwrap();
///
/// assert_paragraph(&para, ParagraphExpected {
///     text_contains: Some("paragraph"),
///     ..Default::default()
/// });
/// ```

/// # Example: Comprehensive Validation
///
/// ```rust,ignore
/// assert_paragraph(&para, ParagraphExpected {
///     text: Some("Exact text match"),
///     has_formatting: Some(false),
///     annotation_count: Some(0),
///     ..Default::default()
/// });
/// ```

/// # Example: Multiple Properties
///
/// ```rust,ignore
/// assert_paragraph(&para, ParagraphExpected {
///     text_contains: Some("important"),
///     has_formatting: Some(true),
///     annotation_count: Some(1),
///     has_annotation: Some("note"),
///     ..Default::default()
/// });
/// ```

