//! Test for Ensemble 01: Two Paragraphs
//!
//! This test validates Phase 1 of the regex-based grammar engine:
//! - Basic engine structure (token loop, pattern matching)
//! - Paragraph pattern (catch-all for PlainTextLine)
//! - Multiple paragraphs separated by blank lines
//!
//! Success criteria: Parse two paragraphs correctly, test block by block

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_ensemble_01_two_paragraphs() {
    // Load ensemble document 01
    let corpus = TxxtCorpora::load_document("01-two-paragraphs")
        .expect("Failed to load ensemble 01-two-paragraphs");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("01-two-paragraphs.txxt".to_string()),
    )
    .expect("Failed to parse ensemble 01");

    println!("=== Parsed Document ===");
    println!("Total elements: {}", document.content.content.len());

    // Test block by block using traversal and assertions
    use txxt::ast::elements::session::session_container::SessionContainerElement;

    let mut paragraph_count = 0;
    for (i, element) in document.content.content.iter().enumerate() {
        println!("\nElement {}: {:?}", i, element_type_name(element));

        if let SessionContainerElement::Paragraph(para) = element {
            paragraph_count += 1;
            let text = extract_text(para);
            println!("  Text: \"{}\"", text);
        }
    }

    println!("\n======================");
    println!("Total paragraphs: {}", paragraph_count);

    // Assertions: We should have exactly 2 paragraphs
    assert_eq!(
        paragraph_count, 2,
        "Expected 2 paragraphs, found {}",
        paragraph_count
    );

    // Test first paragraph content
    if let SessionContainerElement::Paragraph(para1) = &document.content.content[0] {
        let text = extract_text(para1);
        assert!(
            text.contains("first paragraph"),
            "First paragraph should contain 'first paragraph'"
        );
        assert!(
            text.contains("plain text"),
            "First paragraph should contain 'plain text'"
        );
    } else {
        panic!("First element should be a paragraph");
    }

    // Test second paragraph content
    if let SessionContainerElement::Paragraph(para2) = &document.content.content[1] {
        let text = extract_text(para2);
        assert!(
            text.contains("second paragraph"),
            "Second paragraph should contain 'second paragraph'"
        );
        assert!(
            text.contains("blank line"),
            "Second paragraph should contain 'blank line'"
        );
    } else {
        panic!("Second element should be a paragraph");
    }

    println!("\n✅ All assertions passed!");
}

/// Helper to get element type name
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

/// Helper to extract text from paragraph
fn extract_text(paragraph: &txxt::ast::elements::paragraph::ParagraphBlock) -> String {
    paragraph
        .content
        .iter()
        .map(|t| t.text_content())
        .collect::<Vec<_>>()
        .join("")
}
