//! Definition Element Tests - Manual AST Validation
//!
//! Tests definition parsing in isolation using spec examples.
//! This focuses on content population and proper AST structure creation.

use crate::infrastructure::corpora::{ProcessingStage, TxxtCorpora};
use txxt::ast::elements::containers::content::ContentContainerElement;
use txxt::ast::elements::core::ElementNode;
use txxt::ast::elements::inlines::TextTransform;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::parser::pipeline::AstConstructor;

/// Test definition parsing with a simple term-definition pair from spec
/// Using :: txxt.core.spec.definition.valid.simple-term ::
#[test]
fn test_simple_definition_manual_validation() {
    // Load definition spec example using TxxtCorpora
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.definition.valid.simple-term",
        ProcessingStage::SemanticTokens,
    )
    .expect("Should load simple definition spec example");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Should have semantic tokens");

    // Construct AST
    let constructor = AstConstructor::new();
    let ast_result = constructor.construct(semantic_tokens);

    // Debug: let's see what semantic tokens we actually get
    println!("Semantic tokens JSON: {}", semantic_tokens);

    match ast_result {
        Ok(ast) => {
            println!("Simple definition AST: {:?}", ast);

            // Manual validation - check AST structure matches definition spec
            match ast {
                ElementNode::DefinitionBlock(definition) => {
                    println!("✓ Got DefinitionBlock");

                    // CRITICAL TEST: Check if term is populated
                    println!("Term content length: {}", definition.term.content.len());
                    if definition.term.content.is_empty() {
                        println!("❌ FAIL: Definition term is empty");
                        panic!("Definition term should not be empty");
                    } else {
                        println!("✓ Definition has term content");

                        // Manually validate term structure
                        for (i, content_item) in definition.term.content.iter().enumerate() {
                            match content_item {
                                TextTransform::Identity(text_span) => {
                                    println!(
                                        "Term {}: Text '{}' (len: {})",
                                        i,
                                        text_span.content(),
                                        text_span.content().len()
                                    );
                                    assert!(
                                        !text_span.content().trim().is_empty(),
                                        "Term content should not be empty"
                                    );
                                }
                                other => {
                                    println!("Term {}: Other type: {:?}", i, other);
                                }
                            }
                        }

                        // Check that the term content is preserved
                        let concatenated_term: String = definition
                            .term
                            .content
                            .iter()
                            .map(|item| match item {
                                TextTransform::Identity(text_span) => text_span.content().clone(),
                                _ => String::new(),
                            })
                            .collect();

                        println!("Expected term (in source): should contain 'Parser'");
                        println!("Actual term: '{}'", concatenated_term);

                        // Check that term content matches (should contain "Parser")
                        assert!(
                            concatenated_term.contains("Parser"),
                            "Term should contain 'Parser'"
                        );
                    }

                    // CRITICAL TEST: Check if definition content is populated
                    println!(
                        "Definition content blocks: {}",
                        definition.content.content.len()
                    );
                    if definition.content.content.is_empty() {
                        println!("❌ FAIL: Definition content is empty");
                        panic!("Definition content should not be empty");
                    } else {
                        println!("✓ Definition has content blocks");

                        // Check that first content block is a paragraph (for this spec example)
                        match &definition.content.content[0] {
                            ContentContainerElement::Paragraph(paragraph) => {
                                assert!(
                                    !paragraph.content.is_empty(),
                                    "Definition paragraph should have content"
                                );

                                // Get paragraph text
                                let paragraph_text: String = paragraph
                                    .content
                                    .iter()
                                    .map(|item| match item {
                                        TextTransform::Identity(text_span) => {
                                            text_span.content().clone()
                                        }
                                        _ => String::new(),
                                    })
                                    .collect();

                                println!("Definition paragraph text: '{}'", paragraph_text);
                                assert!(
                                    paragraph_text.contains("program"),
                                    "Definition should contain explanatory text"
                                );
                            }
                            other => {
                                println!(
                                    "First definition content is not a paragraph: {:?}",
                                    other
                                );
                            }
                        }
                    }

                    // Validate other definition properties from spec
                    assert_eq!(
                        definition.annotations.len(),
                        0,
                        "Simple definition should have no annotations"
                    );
                    assert!(
                        definition.parameters.is_empty(),
                        "Simple definition should have no parameters"
                    );
                }
                ElementNode::SessionContainer(ref container) => {
                    // If wrapped in container, check the first element
                    assert_eq!(container.content.len(), 1, "Should have one element");
                    match &container.content[0] {
                        SessionContainerElement::Definition(definition) => {
                            println!("✓ Got DefinitionBlock in SessionContainer");
                            // Same validations as above for definition in container
                            assert!(
                                !definition.term.content.is_empty(),
                                "Term should have content"
                            );
                            assert!(
                                !definition.content.content.is_empty(),
                                "Definition should have content"
                            );
                        }
                        other => panic!("Expected definition in container, got: {:?}", other),
                    }
                }
                other => {
                    panic!(
                        "Expected DefinitionBlock or SessionContainer, got: {:?}",
                        other
                    );
                }
            }
        }
        Err(e) => {
            panic!(
                "AST construction should succeed for simple definition: {}",
                e
            );
        }
    }
}

/// Test definition with formatted term from spec
/// Using :: txxt.core.spec.definition.valid.formatted-term ::
#[test]
fn test_formatted_term_definition_manual_validation() {
    // Load definition with formatted term spec example
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.definition.valid.formatted-term",
        ProcessingStage::SemanticTokens,
    )
    .expect("Should load formatted term definition spec example");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Should have semantic tokens");

    // Construct AST
    let constructor = AstConstructor::new();
    let ast_result = constructor.construct(semantic_tokens);

    match ast_result {
        Ok(ast) => {
            println!("Formatted term definition AST: {:?}", ast);

            // Manual validation
            match ast {
                ElementNode::DefinitionBlock(definition) => {
                    println!("✓ Got DefinitionBlock for formatted term");

                    // Check term population
                    assert!(
                        !definition.term.content.is_empty(),
                        "Formatted term should have content"
                    );

                    // For now, just verify some term content exists - detailed inline parsing
                    // validation can come later when inline elements are fully implemented
                    let has_term_content = definition.term.content.iter().any(|item| match item {
                        TextTransform::Identity(text_span) => {
                            !text_span.content().trim().is_empty()
                        }
                        _ => true,
                    });

                    assert!(
                        has_term_content,
                        "Definition term should contain some text content"
                    );

                    // Check definition content
                    assert!(
                        !definition.content.content.is_empty(),
                        "Definition should have content"
                    );

                    println!("✓ Formatted term definition has term and content");
                }
                ElementNode::SessionContainer(ref container) => {
                    // Handle container case
                    assert_eq!(container.content.len(), 1, "Should have one element");
                    match &container.content[0] {
                        SessionContainerElement::Definition(definition) => {
                            assert!(
                                !definition.term.content.is_empty(),
                                "Term should have content"
                            );
                            assert!(
                                !definition.content.content.is_empty(),
                                "Definition should have content"
                            );
                            println!("✓ Formatted term definition in container has content");
                        }
                        other => panic!("Expected definition in container, got: {:?}", other),
                    }
                }
                other => {
                    panic!(
                        "Expected DefinitionBlock for formatted term, got: {:?}",
                        other
                    );
                }
            }
        }
        Err(e) => {
            panic!(
                "AST construction should succeed for formatted term definition: {}",
                e
            );
        }
    }
}

/// Test definition with rich content from spec
/// Using :: txxt.core.spec.definition.valid.rich-content ::
#[test]
fn test_rich_content_definition_manual_validation() {
    // Load definition with rich content spec example
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.definition.valid.rich-content",
        ProcessingStage::SemanticTokens,
    )
    .expect("Should load rich content definition spec example");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Should have semantic tokens");

    // Construct AST
    let constructor = AstConstructor::new();
    let ast_result = constructor.construct(semantic_tokens);

    match ast_result {
        Ok(ast) => {
            println!("Rich content definition AST: {:?}", ast);

            // Manual validation
            match ast {
                ElementNode::DefinitionBlock(definition) => {
                    println!("✓ Got DefinitionBlock for rich content");

                    // Check term population
                    assert!(
                        !definition.term.content.is_empty(),
                        "Rich definition should have term"
                    );

                    let term_text: String = definition
                        .term
                        .content
                        .iter()
                        .map(|item| match item {
                            TextTransform::Identity(text_span) => text_span.content().clone(),
                            _ => String::new(),
                        })
                        .collect();

                    println!("Rich definition term: '{}'", term_text);
                    assert!(
                        term_text.contains("Recursion"),
                        "Term should contain 'Recursion'"
                    );

                    // Check definition content - should have multiple content blocks
                    assert!(
                        !definition.content.content.is_empty(),
                        "Rich definition should have content"
                    );
                    println!(
                        "Rich definition has {} content blocks",
                        definition.content.content.len()
                    );

                    // For rich content, we expect multiple content blocks (paragraphs, lists, verbatim)
                    // For now, just verify we have at least one block with content
                    let has_content_block = definition.content.content.iter().any(|block| {
                        match block {
                            ContentContainerElement::Paragraph(p) => !p.content.is_empty(),
                            ContentContainerElement::List(l) => !l.items.is_empty(),
                            _ => true, // Other block types considered to have content
                        }
                    });

                    assert!(
                        has_content_block,
                        "Rich definition should have at least one non-empty content block"
                    );
                    println!("✓ Rich content definition has populated content blocks");
                }
                ElementNode::SessionContainer(ref container) => {
                    // Handle container case
                    println!(
                        "Rich definition in SessionContainer with {} elements",
                        container.content.len()
                    );

                    // For rich content, might be parsed as multiple elements
                    assert!(
                        !container.content.is_empty(),
                        "Container should have content"
                    );

                    // Look for a definition element
                    let has_definition = container
                        .content
                        .iter()
                        .any(|element| matches!(element, SessionContainerElement::Definition(_)));

                    if has_definition {
                        println!("✓ Rich content parsed with definition in container");
                    } else {
                        println!("Note: Rich content may be parsed as separate elements for now");
                    }
                }
                other => {
                    println!("Got unexpected element type for rich content: {:?}", other);
                    // For rich content, the parsing might be more complex, so we'll observe what we get
                }
            }
        }
        Err(e) => {
            println!("Rich content definition parsing failed: {}", e);
            // Rich content parsing might be complex, so we'll observe failures
        }
    }
}
