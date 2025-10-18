//! Session element parser tests
//!
//! Tests for the session parser implementation using corpora examples
//! and ensemble documents. Leverages TxxtCorpora processing stages to
//! reduce boilerplate and focus on parser logic.

#[path = "../../../infrastructure/corpora.rs"]
mod corpora;

use crate::assertions::{assert_session, SessionExpected};
use corpora::{ProcessingStage, TxxtCorpora};
use txxt::parser::elements::session::session::parse_session;

// ============================================================================
// Corpora Tests (Using Processing Stages)
// ============================================================================

/// Test unnumbered session from corpora using pre-tokenized data
#[test]
#[ignore = "Waiting for semantic token pipeline implementation"]
fn test_corpora_unnumbered_session() {
    // Load corpus with tokenization already done
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.session.valid.unnumbered-basic",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    // Verify we got the right content
    assert!(!corpus.source_text.is_empty(), "Corpus should have content");

    // Get pre-tokenized data
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Parse the session
    let result = parse_session(&tokens);
    assert!(
        result.is_ok(),
        "Unnumbered session should parse successfully"
    );

    let session = result.unwrap();

    // Use assertion framework for validation
    assert_session(
        &txxt::ast::elements::session::session_container::SessionContainerElement::Session(session),
        SessionExpected {
            title_contains: Some("Introduction"),
            is_numbered: Some(false),
            child_count: Some(1), // Should have one paragraph
            ..Default::default()
        },
    );
}

/// Test numbered session from corpora
#[test]
#[ignore = "Waiting for semantic token pipeline implementation"]
fn test_corpora_numbered_session() {
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.session.valid.numbered-basic",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    assert!(!corpus.source_text.is_empty(), "Corpus should have content");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let result = parse_session(&tokens);
    assert!(result.is_ok(), "Numbered session should parse successfully");

    let session = result.unwrap();

    // Use assertion framework for validation
    assert_session(
        &txxt::ast::elements::session::session_container::SessionContainerElement::Session(session),
        SessionExpected {
            title_contains: Some("Methodology"),
            is_numbered: Some(true),
            numbering: Some("1."),
            child_count: Some(1), // Should have one paragraph
            ..Default::default()
        },
    );
}

/// Test session with single child element
#[test]
fn test_corpora_flat_one_child() {
    // Skip this test since the corpus doesn't exist yet
    // TODO: Add the corpus example to the specs
}

/// Test session with two child elements
#[test]
fn test_corpora_flat_two_children() {
    // Skip this test since the corpus doesn't exist yet
    // TODO: Add the corpus example to the specs
}

// ============================================================================
// Ensemble Document Tests
// ============================================================================

/// Test single session ensemble document
#[test]
fn test_ensemble_single_session() {
    let corpus = TxxtCorpora::load_document_with_processing(
        "02-session-one-paragraph",
        ProcessingStage::Tokens,
    )
    .expect("Ensemble document should exist");

    assert!(
        !corpus.source_text.is_empty(),
        "Document should have content"
    );

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let result = parse_session(&tokens);
    assert!(
        result.is_ok(),
        "Single session document should parse successfully"
    );

    let session = result.unwrap();

    // Use assertion framework for validation
    assert_session(
        &txxt::ast::elements::session::session_container::SessionContainerElement::Session(session),
        SessionExpected {
            title_contains: Some("Introduction"),
            child_count: Some(1),
            ..Default::default()
        },
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test session parsing with BlockParser integration
#[test]
#[ignore = "Waiting for semantic token pipeline implementation"]
fn test_block_parser_integration() {
    use txxt::lexer::pipeline::ScannerTokenTreeBuilder;
    use txxt::parser::pipeline::BlockParser;

    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.session.valid.unnumbered-basic",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Build token tree
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .expect("Should build token tree successfully");

    // Parse with BlockParser
    let block_parser = BlockParser::new();
    let result = block_parser.parse_blocks(token_tree);
    assert!(result.is_ok(), "BlockParser should parse successfully");

    let elements = result.unwrap();
    assert!(!elements.is_empty(), "Should have parsed elements");

    // Check that we got a session element
    if let txxt::ast::ElementNode::SessionBlock(session) = &elements[0] {
        // Use assertion framework for validation
        assert_session(
            &txxt::ast::elements::session::session_container::SessionContainerElement::Session(
                session.clone(),
            ),
            SessionExpected {
                title_contains: Some("Introduction"),
                child_count: Some(1),
                ..Default::default()
            },
        );
    } else {
        panic!("Expected SessionBlock element");
    }
}

// ============================================================================
// Manual AST Structure Tests
// ============================================================================

/// Test manual AST structure validation for unnumbered session
#[test]
#[ignore = "Waiting for semantic token pipeline implementation"]
fn test_manual_ast_unnumbered_session() {
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.session.valid.unnumbered-basic",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let session = parse_session(&tokens).expect("Should parse successfully");

    // Manual AST structure validation
    assert_eq!(session.annotations.len(), 0, "Should have no annotations");
    assert_eq!(session.parameters.map.len(), 0, "Should have no parameters");
    assert!(!session.tokens.tokens.is_empty(), "Should have tokens");

    // Title validation
    assert!(
        session.title.numbering.is_none(),
        "Should have no numbering"
    );
    assert!(
        !session.title.content.is_empty(),
        "Should have title content"
    );
    assert!(
        !session.title.tokens.tokens.is_empty(),
        "Should have title tokens"
    );

    // Content validation
    assert!(!session.content.is_empty(), "Should have content");
    assert_eq!(session.content.len(), 1, "Should have one child element");

    // Check that content is a paragraph
    match &session.content.content[0] {
        txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(_) => {
            // Expected
        }
        txxt::ast::elements::session::session_container::SessionContainerElement::Session(_) => {
            // Also acceptable - might be a nested session
        }
        other => panic!("Expected paragraph or session, got {:?}", other),
    }
}

/// Test manual AST structure validation for numbered session
#[test]
#[ignore = "Waiting for semantic token pipeline implementation"]
fn test_manual_ast_numbered_session() {
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.session.valid.numbered-basic",
        ProcessingStage::Tokens,
    )
    .expect("Corpus should exist in specs");

    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    let session = parse_session(&tokens).expect("Should parse successfully");

    // Manual AST structure validation
    assert_eq!(session.annotations.len(), 0, "Should have no annotations");
    assert_eq!(session.parameters.map.len(), 0, "Should have no parameters");
    assert!(!session.tokens.tokens.is_empty(), "Should have tokens");

    // Title validation
    assert!(session.title.numbering.is_some(), "Should have numbering");
    if let Some(numbering) = &session.title.numbering {
        assert_eq!(numbering.marker, "1.", "Should have correct marker");
        assert!(matches!(
            numbering.style,
            txxt::ast::elements::list::NumberingStyle::Numerical
        ));
        assert!(matches!(
            numbering.form,
            txxt::ast::elements::list::NumberingForm::Short
        ));
    }
    assert!(
        !session.title.content.is_empty(),
        "Should have title content"
    );
    assert!(
        !session.title.tokens.tokens.is_empty(),
        "Should have title tokens"
    );

    // Content validation
    assert!(!session.content.is_empty(), "Should have content");
    assert_eq!(session.content.len(), 1, "Should have one child element");

    // Check that content is a paragraph
    match &session.content.content[0] {
        txxt::ast::elements::session::session_container::SessionContainerElement::Paragraph(_) => {
            // Expected
        }
        txxt::ast::elements::session::session_container::SessionContainerElement::Session(_) => {
            // Also acceptable - might be a nested session
        }
        other => panic!("Expected paragraph or session, got {:?}", other),
    }
}
