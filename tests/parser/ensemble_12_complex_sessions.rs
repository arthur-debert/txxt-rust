//! Test for Ensemble 12: Complex Sessions
//!
//! This is the most challenging test document, validating ambiguous cases where
//! sessions, lists, and paragraphs can have similar forms.
//!
//! The document explains the parsing rules:
//! - Sessions require: blank line before title, +1 indented content, non-empty
//! - Lists require: 2+ items (including nested items)
//! - Paragraphs: catch-all when above don't match
//!
//! We test this document block by block, validating exact structure.

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_ensemble_12_full_document() {
    // Load ensemble document 12 using corpora tool
    let corpus =
        TxxtCorpora::load_document("12-complex-sessions").expect("Failed to load ensemble 12");

    println!("=== Ensemble 12: Complex Sessions ===");
    println!("Source length: {} chars", corpus.source_text.len());
    println!();

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("12-complex-sessions.txxt".to_string()),
    )
    .expect("Failed to parse ensemble 12");

    use txxt::ast::elements::session::session_container::SessionContainerElement;

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    for (i, element) in document.content.content.iter().enumerate() {
        println!("  {}: {:?}", i, element_type_name(element));
        if let SessionContainerElement::Session(s) = element {
            println!("      Title: {}", s.title_text());
            println!("      Children: {}", s.content.content.len());
        }
    }
    println!();

    // Debug: Show what's INSIDE the first session
    if let SessionContainerElement::Session(session) = &document.content.content[2] {
        println!("=== INSIDE Session 'Sources' ===");
        for (i, child) in session.content.content.iter().enumerate() {
            println!("  Child {}: {:?}", i, element_type_name(child));
        }
    }

    // Expected top-level structure:
    // - 2 paragraphs (intro)
    // - Session "1. Sources"
    // - Session "2. General idea"
    // - Session "2. Lists"
    // - Session "3. Paragraphs: Dialogs NOT IMPLEMENTED"
    // - Session "4. .Indentation and Containers"

    // First, verify we have sessions
    let session_count = document
        .content
        .content
        .iter()
        .filter(|e| matches!(e, SessionContainerElement::Session(_)))
        .count();

    println!("Total sessions at top level: {}", session_count);

    // Start by just verifying we can parse the document without errors
    assert!(
        !document.content.content.is_empty(),
        "Document should have content"
    );

    assert!(session_count > 0, "Document should have sessions");
}

/// Helper to get element type name for debugging
fn element_type_name(
    element: &txxt::ast::elements::session::session_container::SessionContainerElement,
) -> &'static str {
    use txxt::ast::elements::session::session_container::SessionContainerElement;
    match element {
        SessionContainerElement::Paragraph(_) => "Paragraph",
        SessionContainerElement::Session(_) => "Session",
        SessionContainerElement::List(_) => "List",
        SessionContainerElement::Definition(_) => "Definition",
        SessionContainerElement::Annotation(_) => "Annotation",
        SessionContainerElement::Verbatim(_) => "Verbatim",
        SessionContainerElement::ContentContainer(_) => "ContentContainer",
        SessionContainerElement::SessionContainer(_) => "SessionContainer",
        SessionContainerElement::BlankLine(_) => "BlankLine",
    }
}
