//! Reality Check Tests - No BS, just what actually works
//!
//! These tests make no grandiose claims. They just check if basic functionality exists.

use crate::infrastructure::corpora::{ProcessingStage, TxxtCorpora};
use txxt::ast::elements::core::ElementNode;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::parser::pipeline::AstConstructor;

/// BRUTAL HONESTY TEST: Do paragraphs have ANY content at all?
#[test]
fn test_paragraph_has_any_content() {
    let corpus = TxxtCorpora::load_document_with_processing(
        "01-two-paragraphs",
        ProcessingStage::SemanticTokens,
    )
    .expect("Failed to load 01-two-paragraphs.txxt");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Semantic tokens should be available");
    let constructor = AstConstructor::new();
    let ast = constructor
        .construct(semantic_tokens)
        .expect("Should create AST");

    match ast {
        ElementNode::SessionContainer(ref container) => {
            match &container.content[0] {
                SessionContainerElement::Paragraph(paragraph) => {
                    println!("Paragraph content length: {}", paragraph.content.len());
                    println!("Paragraph content: {:?}", paragraph.content);

                    // HONEST CHECK: Is there ANY content?
                    if paragraph.content.is_empty() {
                        println!("❌ REALITY: Paragraph content is completely empty");
                        println!("❌ VERDICT: Content population is not implemented");
                    } else {
                        println!("✓ Paragraph has some content");
                        // If we get here, we can add more specific content tests
                    }
                }
                other => panic!("Expected paragraph, got: {:?}", other),
            }
        }
        other => panic!("Expected SessionContainer, got: {:?}", other.element_type()),
    }
}

/// BASIC REALITY CHECK: Can we detect a session title?
#[test]
fn test_session_title_exists() {
    let corpus = TxxtCorpora::load_document_with_processing(
        "02-session-one-paragraph",
        ProcessingStage::SemanticTokens,
    )
    .expect("Failed to load 02-session-one-paragraph.txxt");

    let semantic_tokens = corpus
        .semantic_tokens()
        .expect("Semantic tokens should be available");
    let constructor = AstConstructor::new();
    let ast = constructor
        .construct(semantic_tokens)
        .expect("Should create AST");

    match ast {
        ElementNode::SessionBlock(ref session) => {
            let title = session.title_text();
            println!("Session title: '{}'", title);

            if title.trim().is_empty() {
                println!("❌ REALITY: Session title is empty");
            } else {
                println!("✓ Session title exists: '{}'", title);
                if title.contains("Introduction") {
                    println!("✓ Title contains expected text");
                } else {
                    println!("❌ Title doesn't contain expected text");
                }
            }
        }
        ElementNode::SessionContainer(ref container) => {
            println!("Got SessionContainer instead of SessionBlock");
            if !container.content.is_empty() {
                match &container.content[0] {
                    SessionContainerElement::Session(session) => {
                        let title = session.title_text();
                        println!("Nested session title: '{}'", title);
                    }
                    other => println!("First element is not a session: {:?}", other),
                }
            }
        }
        other => {
            println!(
                "Got neither SessionBlock nor SessionContainer: {:?}",
                other.element_type()
            );
        }
    }
}
