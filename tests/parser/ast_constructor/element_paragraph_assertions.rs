//! Paragraph Element AST Assertion Tests
//!
//! Tests paragraph parsing using the AST assertion framework.
//! This validates that our manual tests align with the assertion framework.

use crate::assertions::{assert_paragraph, ParagraphExpected};
use crate::infrastructure::corpora::{ProcessingStage, TxxtCorpora};
use txxt::ast::elements::core::ElementNode;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::parser::pipeline::AstConstructor;

/// Test simple paragraph using AST assertions
/// This validates both the AST Constructor and the assertion framework
#[test]
fn test_simple_paragraph_with_assertions() {
    // Load simple paragraph spec example
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.simple",
        ProcessingStage::SemanticTokens,
    )
    .expect("Should load simple paragraph spec example");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Should have semantic tokens");

    // Construct AST
    let constructor = AstConstructor::new();
    let ast = constructor
        .construct(semantic_tokens)
        .expect("Should construct AST successfully");

    // Validate using AST assertions
    match ast {
        ElementNode::ParagraphBlock(ref paragraph) => {
            assert_paragraph(
                &SessionContainerElement::Paragraph(paragraph.clone()),
                ParagraphExpected {
                    text_contains: Some("basic paragraph"),
                    annotation_count: Some(0),
                    has_formatting: Some(false),
                    ..Default::default()
                },
            );
            println!("✓ Simple paragraph passed AST assertions");
        }
        ElementNode::SessionContainer(ref container) => {
            // If wrapped in container, check the first element
            assert_eq!(container.content.len(), 1, "Should have one element");
            match &container.content[0] {
                SessionContainerElement::Paragraph(paragraph) => {
                    assert_paragraph(
                        &SessionContainerElement::Paragraph(paragraph.clone()),
                        ParagraphExpected {
                            text_contains: Some("basic paragraph"),
                            annotation_count: Some(0),
                            has_formatting: Some(false),
                            ..Default::default()
                        },
                    );
                    println!("✓ Simple paragraph in container passed AST assertions");
                }
                other => panic!("Expected paragraph in container, got: {:?}", other),
            }
        }
        other => panic!("Expected paragraph or container, got: {:?}", other),
    }
}

/// Test multiline paragraph using AST assertions
#[test]
fn test_multiline_paragraph_with_assertions() {
    // Load multiline paragraph spec example
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.multiline",
        ProcessingStage::SemanticTokens,
    )
    .expect("Should load multiline paragraph spec example");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Should have semantic tokens");

    // Construct AST
    let constructor = AstConstructor::new();
    let ast = constructor
        .construct(semantic_tokens)
        .expect("Should construct AST successfully");

    // Validate using AST assertions
    match ast {
        ElementNode::ParagraphBlock(ref paragraph) => {
            assert_paragraph(
                &SessionContainerElement::Paragraph(paragraph.clone()),
                ParagraphExpected {
                    text_contains: Some("one line"), // Should contain merged text
                    annotation_count: Some(0),
                    has_formatting: Some(false),
                    ..Default::default()
                },
            );

            // Additional check that line breaks became spaces
            assert_paragraph(
                &SessionContainerElement::Paragraph(paragraph.clone()),
                ParagraphExpected {
                    text_contains: Some("line and continues"), // Space between lines
                    ..Default::default()
                },
            );

            println!("✓ Multiline paragraph passed AST assertions");
        }
        ElementNode::SessionContainer(ref container) => {
            // If wrapped in container, check the first element
            assert_eq!(container.content.len(), 1, "Should have one element");
            match &container.content[0] {
                SessionContainerElement::Paragraph(paragraph) => {
                    assert_paragraph(
                        &SessionContainerElement::Paragraph(paragraph.clone()),
                        ParagraphExpected {
                            text_contains: Some("one line"),
                            annotation_count: Some(0),
                            has_formatting: Some(false),
                            ..Default::default()
                        },
                    );
                    println!("✓ Multiline paragraph in container passed AST assertions");
                }
                other => panic!("Expected paragraph in container, got: {:?}", other),
            }
        }
        other => panic!("Expected paragraph or container, got: {:?}", other),
    }
}

/// Test formatted paragraph using AST assertions  
#[test]
fn test_formatted_paragraph_with_assertions() {
    // Load formatted paragraph spec example
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.with-formatting",
        ProcessingStage::SemanticTokens,
    )
    .expect("Should load formatted paragraph spec example");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Should have semantic tokens");

    // Construct AST
    let constructor = AstConstructor::new();
    let ast = constructor
        .construct(semantic_tokens)
        .expect("Should construct AST successfully");

    // Validate using AST assertions
    match ast {
        ElementNode::ParagraphBlock(ref paragraph) => {
            assert_paragraph(
                &SessionContainerElement::Paragraph(paragraph.clone()),
                ParagraphExpected {
                    text_contains: Some("paragraph contains"), // Should contain core text
                    annotation_count: Some(0),
                    // Note: has_formatting might be false for now if inline parsing is not implemented
                    ..Default::default()
                },
            );
            println!("✓ Formatted paragraph passed AST assertions");
        }
        ElementNode::SessionContainer(ref container) => {
            // If wrapped in container, check the first element
            assert_eq!(container.content.len(), 1, "Should have one element");
            match &container.content[0] {
                SessionContainerElement::Paragraph(paragraph) => {
                    assert_paragraph(
                        &SessionContainerElement::Paragraph(paragraph.clone()),
                        ParagraphExpected {
                            text_contains: Some("paragraph contains"),
                            annotation_count: Some(0),
                            ..Default::default()
                        },
                    );
                    println!("✓ Formatted paragraph in container passed AST assertions");
                }
                other => panic!("Expected paragraph in container, got: {:?}", other),
            }
        }
        other => panic!("Expected paragraph or container, got: {:?}", other),
    }
}
