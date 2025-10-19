//! Integration tests for AST construction using corpora tool

mod corpora {
    include!("../../infrastructure/corpora.rs");
}

use corpora::{ProcessingStage, TxxtCorpora};

/// Test AST construction with simple paragraph example
#[test]
fn test_ast_construction_two_paragraphs() {
    // Load the two paragraphs example with semantic tokens processing
    let corpus = TxxtCorpora::load_document_with_processing(
        "01-two-paragraphs",
        ProcessingStage::SemanticTokens,
    );
    if let Err(e) = &corpus {
        println!("Failed to load corpus: {:?}", e);
    }
    assert!(corpus.is_ok(), "Failed to load corpus");
    let corpus = corpus.unwrap();

    // Get the semantic tokens
    let semantic_tokens_str = corpus.semantic_tokens();
    assert!(semantic_tokens_str.is_some(), "No semantic tokens found");

    // For now, we'll just verify that the corpus loaded successfully
    // TODO: Parse the semantic tokens string into SemanticTokenList and test AST construction
    println!("Successfully loaded two paragraphs corpus with semantic tokens");
}

/// Test AST construction with session and paragraph
#[test]
fn test_ast_construction_session_one_paragraph() {
    // Load the session with one paragraph example with semantic tokens processing
    let corpus = TxxtCorpora::load_document_with_processing(
        "02-session-one-paragraph",
        ProcessingStage::SemanticTokens,
    );
    assert!(corpus.is_ok(), "Failed to load corpus");
    let corpus = corpus.unwrap();

    // Get the semantic tokens
    let semantic_tokens_str = corpus.semantic_tokens();
    assert!(semantic_tokens_str.is_some(), "No semantic tokens found");

    // For now, we'll just verify that the corpus loaded successfully
    // TODO: Parse the semantic tokens string into SemanticTokenList and test AST construction
    println!("Successfully loaded session with paragraph corpus with semantic tokens");
}

/// Test AST construction with nested sessions
#[test]
fn test_ast_construction_nested_sessions() {
    // Load the nested sessions example with semantic tokens processing
    let corpus = TxxtCorpora::load_document_with_processing(
        "05-nested-sessions-basic",
        ProcessingStage::SemanticTokens,
    );
    assert!(corpus.is_ok(), "Failed to load corpus");
    let corpus = corpus.unwrap();

    // Get the semantic tokens
    let semantic_tokens_str = corpus.semantic_tokens();
    assert!(semantic_tokens_str.is_some(), "No semantic tokens found");

    // For now, we'll just verify that the corpus loaded successfully
    // TODO: Parse the semantic tokens string into SemanticTokenList and test AST construction
    println!("Successfully loaded nested sessions corpus with semantic tokens");
}
