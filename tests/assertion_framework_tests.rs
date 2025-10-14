//! Tests for the AST assertion framework.
//!
//! Validates that the assertion helpers work correctly with proper error messages.

#[path = "assertions/mod.rs"]
mod assertions;

#[cfg(feature = "new-ast")]
mod framework_tests {
    use super::assertions::{assert_paragraph, ParagraphExpected};
    use txxt::ast::{
        elements::{
            containers::session::SessionContainerElement,
            core::{ElementType, TxxtElement},
            inlines::TextTransform,
            paragraph::ParagraphBlock,
        },
        parameters::Parameters,
        tokens::{Position, SourceSpan, Token, TokenSequence},
    };

    /// Helper to create a simple paragraph for testing
    fn make_test_paragraph(text: &str) -> SessionContainerElement {
        let text_transform = TextTransform::Identity(txxt::ast::elements::inlines::TextSpan {
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
        });

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
        use txxt::ast::elements::core::BlankLine;

        // Create a non-paragraph element
        let element = SessionContainerElement::BlankLine(BlankLine {
            tokens: TokenSequence::new(),
        });

        // Should panic - wrong type
        assert_paragraph(&element, ParagraphExpected::default());
    }

    #[test]
    fn test_assert_paragraph_text_contains_succeeds() {
        let element = make_test_paragraph("This is a test paragraph");

        // Should not panic - contains substring
        assert_paragraph(
            &element,
            ParagraphExpected {
                text_contains: Some("test paragraph"),
                ..Default::default()
            },
        );
    }

    #[test]
    #[should_panic(expected = "Text content validation failed")]
    fn test_assert_paragraph_text_contains_fails() {
        let element = make_test_paragraph("This is a test paragraph");

        // Should panic - doesn't contain substring
        assert_paragraph(
            &element,
            ParagraphExpected {
                text_contains: Some("missing text"),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_paragraph_optional_fields_work() {
        let element = make_test_paragraph("Any text here");

        // Should not panic - no validations specified
        assert_paragraph(&element, ParagraphExpected::default());

        // Should work with partial specification
        assert_paragraph(
            &element,
            ParagraphExpected {
                text_contains: Some("Any"),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_paragraph_annotation_count() {
        let element = make_test_paragraph("Text");

        // Should pass - has 0 annotations
        assert_paragraph(
            &element,
            ParagraphExpected {
                annotation_count: Some(0),
                ..Default::default()
            },
        );
    }

    #[test]
    #[should_panic(expected = "Annotation count mismatch")]
    fn test_assert_paragraph_annotation_count_fails() {
        let element = make_test_paragraph("Text");

        // Should panic - expected 1 but has 0
        assert_paragraph(
            &element,
            ParagraphExpected {
                annotation_count: Some(1),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_paragraph_has_formatting() {
        let element = make_test_paragraph("Plain text");

        // Should pass - plain text has no formatting
        assert_paragraph(
            &element,
            ParagraphExpected {
                has_formatting: Some(false),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_paragraph_exact_text() {
        let element = make_test_paragraph("Exact text match");

        // Should pass - exact match
        assert_paragraph(
            &element,
            ParagraphExpected {
                text: Some("Exact text match"),
                ..Default::default()
            },
        );
    }

    #[test]
    #[should_panic(expected = "Text content mismatch")]
    fn test_assert_paragraph_exact_text_fails() {
        let element = make_test_paragraph("Actual text");

        // Should panic - doesn't match exactly
        assert_paragraph(
            &element,
            ParagraphExpected {
                text: Some("Different text"),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_multiple_properties_validated() {
        let element = make_test_paragraph("Test paragraph content");

        // Validate multiple properties at once
        assert_paragraph(
            &element,
            ParagraphExpected {
                text_contains: Some("Test"),
                has_formatting: Some(false),
                annotation_count: Some(0),
                ..Default::default()
            },
        );
    }
}
