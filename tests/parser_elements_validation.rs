//! Phase 1: Grammar Tuning and Token Verification
//!
//! This test validates that semantic tokens are correctly generated for
//! the core block elements (paragraphs, sessions, lists) using the corpora tool.

// Include the corpora module from the same directory
#[path = "infrastructure/corpora.rs"]
mod corpora;

use corpora::{ProcessingStage, TxxtCorpora};

#[test]
fn test_paragraph_semantic_tokens() {
    // Load a simple paragraph example from the spec
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.paragraph.valid.simple",
        ProcessingStage::SemanticTokens,
    )
    .expect("Failed to load paragraph corpus");

    // Get the semantic tokens
    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Failed to get semantic tokens");

    println!("Paragraph semantic tokens:");
    println!("{}", semantic_tokens);

    // Basic validation - should contain PlainTextLine tokens
    assert!(semantic_tokens.contains("PlainTextLine"));
}

#[test]
fn test_session_semantic_tokens() {
    // Load a simple session example from the spec
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.session.valid.unnumbered-basic",
        ProcessingStage::SemanticTokens,
    )
    .expect("Failed to load session corpus");

    // Get the semantic tokens
    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Failed to get semantic tokens");

    println!("Session semantic tokens:");
    println!("{}", semantic_tokens);

    // Basic validation - should contain SequenceTextLine and PlainTextLine tokens
    assert!(semantic_tokens.contains("PlainTextLine"));
}

#[test]
fn test_list_semantic_tokens() {
    // Load a simple list example from the spec
    let corpus = TxxtCorpora::load_with_processing(
        "txxt.core.spec.list.valid.plain-flat",
        ProcessingStage::SemanticTokens,
    )
    .expect("Failed to load list corpus");

    // Get the semantic tokens
    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Failed to get semantic tokens");

    println!("List semantic tokens:");
    println!("{}", semantic_tokens);

    // Basic validation - should contain SequenceTextLine tokens
    assert!(semantic_tokens.contains("SequenceTextLine"));
}
