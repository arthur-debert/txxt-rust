//! Tests for the AST assertion framework.
//!
//! Validates that the assertion helpers work correctly with proper error messages.

#[path = "assertions/mod.rs"]
mod assertions;

#[cfg(feature = "new-ast")]
mod framework_tests {
    use super::assertions::{
        assert_annotation, assert_content_container, assert_inline_content, assert_paragraph,
        assert_session_container, AnnotationExpected, ContentContainerExpected,
        InlineContentExpected, ParagraphExpected, SessionContainerExpected,
    };
    use txxt::ast::{
        elements::{
            annotation::{AnnotationBlock, AnnotationContent},
            containers::{
                content::{ContentContainer, ContentContainerElement},
                session::{SessionContainer, SessionContainerElement},
            },
            core::{ElementType, TxxtElement},
            inlines::{TextSpan, TextTransform},
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

    // ============================================================================
    // Annotation Assertion Tests
    // ============================================================================

    /// Helper to create a test annotation
    fn make_test_annotation(label: &str, content: &str) -> SessionContainerElement {
        let text_transform = TextTransform::Identity(TextSpan {
            tokens: TokenSequence {
                tokens: vec![Token::Text {
                    content: content.to_string(),
                    span: SourceSpan {
                        start: Position { row: 0, column: 0 },
                        end: Position {
                            row: 0,
                            column: content.len(),
                        },
                    },
                }],
            },
            annotations: vec![],
            parameters: Parameters::new(),
        });

        SessionContainerElement::Annotation(AnnotationBlock {
            label: label.to_string(),
            content: AnnotationContent::Inline(vec![text_transform]),
            parameters: Parameters::new(),
            annotations: vec![],
            tokens: TokenSequence::new(),
            namespace: None,
        })
    }

    #[test]
    fn test_assert_annotation_label() {
        let element = make_test_annotation("note", "This is a note");

        assert_annotation(
            &element,
            AnnotationExpected {
                label: Some("note"),
                ..Default::default()
            },
        );
    }

    #[test]
    #[should_panic(expected = "Element type assertion failed")]
    fn test_assert_annotation_type_check_fails() {
        let element = make_test_paragraph("Not an annotation");

        assert_annotation(&element, AnnotationExpected::default());
    }

    #[test]
    fn test_assert_annotation_has_content() {
        let element = make_test_annotation("warning", "Warning text");

        assert_annotation(
            &element,
            AnnotationExpected {
                has_content: Some(true),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_annotation_content_text() {
        let element = make_test_annotation("note", "Exact content");

        assert_annotation(
            &element,
            AnnotationExpected {
                content_text: Some("Exact content"),
                ..Default::default()
            },
        );
    }

    // ============================================================================
    // Container Assertion Tests
    // ============================================================================

    #[test]
    fn test_assert_content_container_element_count() {
        let para1 = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let para2 = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let container = ContentContainer {
            content: vec![
                ContentContainerElement::Paragraph(para1),
                ContentContainerElement::Paragraph(para2),
            ],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_content_container(
            &container,
            ContentContainerExpected {
                element_count: Some(2),
                ..Default::default()
            },
        );
    }

    #[test]
    #[should_panic(expected = "element count mismatch")]
    fn test_assert_content_container_element_count_fails() {
        let container = ContentContainer {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_content_container(
            &container,
            ContentContainerExpected {
                element_count: Some(5),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_session_container_has_session() {
        use txxt::ast::elements::session::{SessionBlock, SessionTitle};

        let session = SessionBlock {
            title: SessionTitle {
                content: vec![],
                numbering: None,
                tokens: TokenSequence::new(),
            },
            content: SessionContainer {
                content: vec![],
                annotations: vec![],
                parameters: Parameters::new(),
                tokens: TokenSequence::new(),
            },
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let container = SessionContainer {
            content: vec![SessionContainerElement::Session(session)],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_session_container(
            &container,
            SessionContainerExpected {
                has_session: Some(true),
                session_count: Some(1),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_session_container_no_sessions() {
        let para = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let container = SessionContainer {
            content: vec![SessionContainerElement::Paragraph(para)],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_session_container(
            &container,
            SessionContainerExpected {
                has_session: Some(false),
                session_count: Some(0),
                element_count: Some(1),
                ..Default::default()
            },
        );
    }

    // ============================================================================
    // Inline Content Assertion Tests
    // ============================================================================

    #[test]
    fn test_assert_inline_content_transform_count() {
        let text_span = TextSpan {
            tokens: TokenSequence::new(),
            annotations: vec![],
            parameters: Parameters::new(),
        };

        let transforms = vec![
            TextTransform::Identity(text_span.clone()),
            TextTransform::Identity(text_span),
        ];

        assert_inline_content(
            &transforms,
            InlineContentExpected {
                transform_count: Some(2),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_inline_content_has_bold() {
        let text_span = TextSpan {
            tokens: TokenSequence::new(),
            annotations: vec![],
            parameters: Parameters::new(),
        };

        let transforms = vec![TextTransform::Strong(vec![TextTransform::Identity(
            text_span,
        )])];

        assert_inline_content(
            &transforms,
            InlineContentExpected {
                has_bold: Some(true),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_inline_content_no_formatting() {
        let text_span = TextSpan {
            tokens: TokenSequence::new(),
            annotations: vec![],
            parameters: Parameters::new(),
        };

        let transforms = vec![TextTransform::Identity(text_span)];

        assert_inline_content(
            &transforms,
            InlineContentExpected {
                has_bold: Some(false),
                has_italic: Some(false),
                has_code: Some(false),
                has_math: Some(false),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_inline_content_has_italic() {
        let text_span = TextSpan {
            tokens: TokenSequence::new(),
            annotations: vec![],
            parameters: Parameters::new(),
        };

        let transforms = vec![TextTransform::Emphasis(vec![TextTransform::Identity(
            text_span,
        )])];

        assert_inline_content(
            &transforms,
            InlineContentExpected {
                has_italic: Some(true),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_inline_content_has_code() {
        let text_span = TextSpan {
            tokens: TokenSequence::new(),
            annotations: vec![],
            parameters: Parameters::new(),
        };

        let transforms = vec![TextTransform::Code(text_span)];

        assert_inline_content(
            &transforms,
            InlineContentExpected {
                has_code: Some(true),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_inline_content_has_math() {
        let text_span = TextSpan {
            tokens: TokenSequence::new(),
            annotations: vec![],
            parameters: Parameters::new(),
        };

        let transforms = vec![TextTransform::Math(text_span)];

        assert_inline_content(
            &transforms,
            InlineContentExpected {
                has_math: Some(true),
                ..Default::default()
            },
        );
    }

    // ============================================================================
    // Enhanced Container Assertion Tests
    // ============================================================================

    #[test]
    fn test_assert_content_container_element_types() {
        use txxt::ast::elements::{
            core::ElementType,
            list::{ListBlock, ListDecorationType},
        };

        let para = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let list = ListBlock {
            decoration_type: ListDecorationType::default(),
            items: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let container = ContentContainer {
            content: vec![
                ContentContainerElement::Paragraph(para),
                ContentContainerElement::List(list),
            ],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_content_container(
            &container,
            ContentContainerExpected {
                element_types: Some(vec![ElementType::Block, ElementType::Block]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_content_container_has_element_type() {
        use txxt::ast::elements::core::ElementType;

        let para = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let container = ContentContainer {
            content: vec![ContentContainerElement::Paragraph(para)],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_content_container(
            &container,
            ContentContainerExpected {
                has_element_type: Some(ElementType::Block),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_content_container_all_same_type() {
        use txxt::ast::elements::core::ElementType;

        let para1 = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let para2 = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let container = ContentContainer {
            content: vec![
                ContentContainerElement::Paragraph(para1),
                ContentContainerElement::Paragraph(para2),
            ],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_content_container(
            &container,
            ContentContainerExpected {
                all_same_type: Some(ElementType::Block),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_session_container_element_types() {
        use txxt::ast::elements::core::ElementType;

        let para = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let container = SessionContainer {
            content: vec![SessionContainerElement::Paragraph(para)],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        assert_session_container(
            &container,
            SessionContainerExpected {
                element_types: Some(vec![ElementType::Block]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_assert_annotation_block_content_contains() {
        // Create annotation with block content
        let para = ParagraphBlock {
            content: vec![TextTransform::Identity(TextSpan {
                tokens: TokenSequence {
                    tokens: vec![Token::Text {
                        content: "Block content text".to_string(),
                        span: SourceSpan {
                            start: Position { row: 0, column: 0 },
                            end: Position {
                                row: 0,
                                column: 18,
                            },
                        },
                    }],
                },
                annotations: vec![],
                parameters: Parameters::new(),
            })],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let block_content = ContentContainer {
            content: vec![ContentContainerElement::Paragraph(para)],
            annotations: vec![],
            parameters: Parameters::new(),
            tokens: TokenSequence::new(),
        };

        let element = SessionContainerElement::Annotation(AnnotationBlock {
            label: "note".to_string(),
            content: AnnotationContent::Block(block_content),
            parameters: Parameters::new(),
            annotations: vec![],
            tokens: TokenSequence::new(),
            namespace: None,
        });

        assert_annotation(
            &element,
            AnnotationExpected {
                content_contains: Some("Block content"),
                ..Default::default()
            },
        );
    }
}
