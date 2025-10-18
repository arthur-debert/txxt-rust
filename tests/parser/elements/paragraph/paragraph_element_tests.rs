//! Paragraph element parser tests
//!
//! Tests for the paragraph parser implementation using corpora examples
//! and ensemble documents. Leverages TxxtCorpora processing stages to
//! reduce boilerplate and focus on parser logic.

#[path = "../../../infrastructure/corpora.rs"]
mod corpora;

use crate::assertions::{assert_paragraph, ParagraphExpected};
use corpora::{ProcessingStage, TxxtCorpora};
use txxt::parser::elements::paragraph::paragraph::parse_paragraph;

// ============================================================================
// Corpora Tests (Using Processing Stages)
// ============================================================================

/// Test simple paragraph from corpora using pre-tokenized data
#[test]
fn test_corpora_simple_paragraph() {
    // Load corpus with tokenization already done
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.simple",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    // Verify we got the right content
    assert!(!corpus.source_text.is_empty(), "Corpus should have content");
    assert!(
        corpus.source_text.contains("paragraph"),
        "Should be about paragraphs"
    );

    // Get pre-tokenized data (currently placeholder, but shows the pattern)
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Parse the paragraph
    let result = parse_paragraph(&tokens);
    assert!(result.is_ok(), "Simple paragraph should parse successfully");

    let paragraph = result.unwrap();

    // Use assertion framework for validation
    assert_paragraph(
        &txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(
            paragraph,
        ),
        ParagraphExpected {
            text_contains: Some("paragraph"),
            has_formatting: Some(false),
            annotation_count: Some(0),
            ..Default::default()
        },
    );
}

/// Test multiline paragraph from corpora
#[test]
fn test_corpora_multiline_paragraph() {
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.multiline",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    assert!(!corpus.source_text.is_empty(), "Corpus should have content");
    assert!(
        corpus.source_text.contains("line"),
        "Should be about multiline content"
    );

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let result = parse_paragraph(&tokens);
    assert!(
        result.is_ok(),
        "Multiline paragraph should parse successfully"
    );

    let paragraph = result.unwrap();

    // Use assertion framework for validation
    assert_paragraph(
        &txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(
            paragraph,
        ),
        ParagraphExpected {
            text_contains: Some("line"),
            has_formatting: Some(false),
            annotation_count: Some(0),
            ..Default::default()
        },
    );
}

/// Test consistent indent paragraph from corpora
#[test]
fn test_corpora_consistent_indent_paragraph() {
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.consistent-indent",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    assert!(!corpus.source_text.is_empty(), "Corpus should have content");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let result = parse_paragraph(&tokens);
    assert!(
        result.is_ok(),
        "Consistent indent paragraph should parse successfully"
    );

    let paragraph = result.unwrap();

    // Use assertion framework for validation
    assert_paragraph(
        &txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(
            paragraph,
        ),
        ParagraphExpected {
            text_contains: Some("paragraph"),
            has_formatting: Some(false),
            annotation_count: Some(0),
            ..Default::default()
        },
    );
}

// ============================================================================
// Ensemble Document Tests
// ============================================================================

/// Test single paragraph ensemble document
#[test]
fn test_ensemble_single_paragraph() {
    let corpus =
        TxxtCorpora::load_document_with_processing("01-two-paragraphs", ProcessingStage::Tokens)
            .expect("Ensemble document should exist");

    assert!(
        !corpus.source_text.is_empty(),
        "Document should have content"
    );
    assert!(
        corpus.source_text.contains("paragraph"),
        "Should contain paragraph content"
    );

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let result = parse_paragraph(&tokens);
    assert!(
        result.is_ok(),
        "Single paragraph document should parse successfully"
    );

    let paragraph = result.unwrap();

    // Use assertion framework for validation
    assert_paragraph(
        &txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(
            paragraph,
        ),
        ParagraphExpected {
            text_contains: Some("paragraph"),
            has_formatting: Some(false),
            annotation_count: Some(0),
            ..Default::default()
        },
    );
}

/// Test multiple paragraphs ensemble document
#[test]
fn test_ensemble_multiple_paragraphs() {
    let corpus =
        TxxtCorpora::load_document_with_processing("01-two-paragraphs", ProcessingStage::Tokens)
            .expect("Ensemble document should exist");

    assert!(
        !corpus.source_text.is_empty(),
        "Document should have content"
    );
    assert!(
        corpus.source_text.contains("first paragraph"),
        "Should contain first paragraph"
    );
    assert!(
        corpus.source_text.contains("second paragraph"),
        "Should contain second paragraph"
    );

    let tokens = txxt::lexer::tokenize(&corpus.source_text);

    // Parse the first paragraph (up to first blank line)
    let first_paragraph_tokens: Vec<_> = tokens
        .iter()
        .take_while(|token| !matches!(token, txxt::ast::scanner_tokens::ScannerToken::BlankLine { .. }))
        .cloned()
        .collect();

    if !first_paragraph_tokens.is_empty() {
        let result = parse_paragraph(&first_paragraph_tokens);
        assert!(result.is_ok(), "First paragraph should parse successfully");

        let paragraph = result.unwrap();

        // Use assertion framework for validation
        assert_paragraph(
            &txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(
                paragraph,
            ),
            ParagraphExpected {
                text_contains: Some("This"),
                has_formatting: Some(false),
                annotation_count: Some(0),
                ..Default::default()
            },
        );
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test paragraph parsing with BlockParser integration
#[test]
fn test_block_parser_integration() {
    use txxt::lexer::pipeline::TokenTreeBuilder;
    use txxt::parser::pipeline::BlockParser;

    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.simple",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Build token tree
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .expect("Should build token tree successfully");

    // Parse with BlockParser
    let block_parser = BlockParser::new();
    let result = block_parser.parse_blocks(token_tree);
    assert!(result.is_ok(), "BlockParser should parse successfully");

    let elements = result.unwrap();
    assert!(!elements.is_empty(), "Should have parsed elements");

    // Check that we got a paragraph element
    if let txxt::ast::ElementNode::ParagraphBlock(paragraph) = &elements[0] {
        // Use assertion framework for validation
        assert_paragraph(
            &txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(
                paragraph.clone(),
            ),
            ParagraphExpected {
                text_contains: Some("paragraph"),
                has_formatting: Some(false),
                annotation_count: Some(0),
                ..Default::default()
            },
        );
    } else {
        panic!("Expected ParagraphBlock element");
    }
}

// ============================================================================
// Batch Tests (Using load_all capabilities)
// ============================================================================

/// Test all paragraph corpora in batch
#[test]
fn test_all_paragraph_corpora() {
    let corpora = TxxtCorpora::load_all_with_processing(ProcessingStage::Tokens)
        .expect("Should load all corpora");

    // Filter for paragraph corpora
    let paragraph_corpora: Vec<_> = corpora
        .into_iter()
        .filter(|corpus| corpus.name.contains("paragraph") && corpus.name.contains("valid"))
        .collect();

    assert!(
        !paragraph_corpora.is_empty(),
        "Should have paragraph corpora"
    );

    for corpus in paragraph_corpora {
        let tokens = txxt::lexer::tokenize(&corpus.source_text);
        let result = parse_paragraph(&tokens);

        assert!(
            result.is_ok(),
            "Paragraph corpus '{}' should parse successfully",
            corpus.name
        );

        let paragraph = result.unwrap();

        // Use assertion framework for validation
        assert_paragraph(
            &txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(
                paragraph,
            ),
            ParagraphExpected {
                has_formatting: Some(false),
                annotation_count: Some(0),
                ..Default::default()
            },
        );
    }
}

// ============================================================================
// Manual AST Structure Validation Tests
// ============================================================================

/// Test manual AST structure validation for simple paragraph
#[test]
fn test_manual_ast_simple_paragraph() {
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.simple",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let result = parse_paragraph(&tokens);
    assert!(result.is_ok(), "Simple paragraph should parse successfully");

    let paragraph = result.unwrap();

    // Manual AST structure validation
    assert_eq!(paragraph.annotations.len(), 0, "Should have no annotations");
    assert_eq!(
        paragraph.parameters.map.len(),
        0,
        "Should have no parameters"
    );
    assert!(!paragraph.tokens.tokens.is_empty(), "Should have tokens");
    assert_eq!(
        paragraph.content.len(),
        1,
        "Should have one content element"
    );

    // Check content structure
    if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) = &paragraph.content[0]
    {
        assert!(
            !text_span.tokens.tokens.is_empty(),
            "Should have text tokens"
        );
        assert!(
            text_span.tokens.text().contains("paragraph"),
            "Should contain paragraph text"
        );
    } else {
        panic!("Expected identity text transform");
    }
}

/// Test manual AST structure validation for multiline paragraph
#[test]
fn test_manual_ast_multiline_paragraph() {
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.multiline",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let result = parse_paragraph(&tokens);
    assert!(
        result.is_ok(),
        "Multiline paragraph should parse successfully"
    );

    let paragraph = result.unwrap();

    // Manual AST structure validation
    assert_eq!(paragraph.annotations.len(), 0, "Should have no annotations");
    assert_eq!(
        paragraph.parameters.map.len(),
        0,
        "Should have no parameters"
    );
    assert!(!paragraph.tokens.tokens.is_empty(), "Should have tokens");
    assert_eq!(
        paragraph.content.len(),
        1,
        "Should have one content element"
    );

    // Check content structure
    if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) = &paragraph.content[0]
    {
        assert!(
            !text_span.tokens.tokens.is_empty(),
            "Should have text tokens"
        );
        let text = text_span.tokens.text();
        assert!(text.contains("line"), "Should contain line text");
        assert!(text.contains("paragraph"), "Should contain paragraph text");
    } else {
        panic!("Expected identity text transform");
    }
}
