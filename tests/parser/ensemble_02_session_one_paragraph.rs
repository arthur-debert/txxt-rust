//! Test for Ensemble 02: Session with One Paragraph
//!
//! This test validates Phase 2 of the regex-based grammar engine:
//! - Session pattern matching with lookahead
//! - Container recursion (session contains paragraph)
//! - Indent/Dedent handling
//!
//! Pattern: <BlankLine> <TitleLine> <BlankLine> <Indent> <Paragraph> <Dedent>

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_ensemble_02_session_one_paragraph() {
    // Load ensemble document 02
    let corpus =
        TxxtCorpora::load_document("02-session-one-paragraph").expect("Failed to load ensemble 02");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("02-session-one-paragraph.txxt".to_string()),
    )
    .expect("Failed to parse ensemble 02");

    println!("=== Parsed Document ===");
    println!("Total elements: {}", document.content.content.len());

    // Test block by block using traversal and assertions
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
        println!("Session title: \"{}\"", session.title_text());

        // Check title
        assert_eq!(
            session.title_text().trim(),
            "Introduction",
            "Session title should be 'Introduction'"
        );

        // Session should contain 1 paragraph
        let paragraph_count = session
            .content
            .content
            .iter()
            .filter(|e| matches!(e, SessionContainerElement::Paragraph(_)))
            .count();

        println!("Paragraphs in session: {}", paragraph_count);

        assert_eq!(
            paragraph_count, 1,
            "Session should contain 1 paragraph, found {}",
            paragraph_count
        );

        // Check paragraph content
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let text = extract_text(para);
            println!("Paragraph text: \"{}\"", text);

            assert!(
                text.contains("simple session"),
                "Paragraph should contain 'simple session'"
            );
            assert!(
                text.contains("one paragraph"),
                "Paragraph should contain 'one paragraph'"
            );
        } else {
            panic!("First element in session should be a paragraph");
        }

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
