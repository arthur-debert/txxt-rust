//! Test for Ensemble 03: Session with Multiple Paragraphs
//!
//! This test validates container recursion with multiple children:
//! - Session contains 3 paragraphs
//! - All paragraphs parsed correctly
//! - Session title with sequence marker (numbered session)

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_ensemble_03_session_multiple_paragraphs() {
    // Load ensemble document 03
    let corpus = TxxtCorpora::load_document("03-session-multiple-paragraphs")
        .expect("Failed to load ensemble 03");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("03-session-multiple-paragraphs.txxt".to_string()),
    )
    .expect("Failed to parse ensemble 03");

    println!("=== Parsed Document ===");
    println!("Total elements: {}", document.content.content.len());

    // Test block by block
    use txxt::ast::elements::session::session_container::SessionContainerElement;

    // DEBUG: Print all elements
    for (i, element) in document.content.content.iter().enumerate() {
        println!("Element {}: {:?}", i, element_type_name(element));
    }

    // Should have exactly 1 session
    assert_eq!(
        document.content.content.len(),
        1,
        "Expected 1 element (session), found {}",
        document.content.content.len()
    );

    // First element should be a session
    if let SessionContainerElement::Session(session) = &document.content.content[0] {
        println!("\nSession title: \"{}\"", session.title_text());

        // Check title contains "Getting Started"
        assert!(
            session.title_text().contains("Getting Started"),
            "Session title should contain 'Getting Started'"
        );

        // Session should contain 3 paragraphs
        let paragraph_count = session
            .content
            .content
            .iter()
            .filter(|e| matches!(e, SessionContainerElement::Paragraph(_)))
            .count();

        println!("Paragraphs in session: {}", paragraph_count);

        assert_eq!(
            paragraph_count, 3,
            "Session should contain 3 paragraphs, found {}",
            paragraph_count
        );

        // Check each paragraph content
        let paragraphs: Vec<_> = session
            .content
            .content
            .iter()
            .filter_map(|e| match e {
                SessionContainerElement::Paragraph(p) => Some(p),
                _ => None,
            })
            .collect();

        // First paragraph
        let text1 = extract_text(paragraphs[0]);
        println!("\nParagraph 1: \"{}\"", text1);
        assert!(
            text1.contains("first paragraph"),
            "First paragraph should contain 'first paragraph'"
        );
        assert!(
            text1.contains("introduces the topic"),
            "First paragraph should mention introducing the topic"
        );

        // Second paragraph
        let text2 = extract_text(paragraphs[1]);
        println!("Paragraph 2: \"{}\"", text2);
        assert!(
            text2.contains("second paragraph"),
            "Second paragraph should contain 'second paragraph'"
        );
        assert!(
            text2.contains("expands on the introduction"),
            "Second paragraph should mention expanding on introduction"
        );

        // Third paragraph
        let text3 = extract_text(paragraphs[2]);
        println!("Paragraph 3: \"{}\"", text3);
        assert!(
            text3.contains("third and final paragraph"),
            "Third paragraph should contain 'third and final paragraph'"
        );
        assert!(
            text3.contains("wraps up"),
            "Third paragraph should mention wrapping up"
        );

        println!("\n✅ All assertions passed!");
    } else {
        panic!("First element should be a session");
    }
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
