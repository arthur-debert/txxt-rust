//! Test for Ensemble 04: Multiple Sessions Flat
//!
//! This test validates flat structure with multiple peer sessions:
//! - 3 sessions at the same level (no nesting)
//! - Each session has numbered title (sequence marker)
//! - Each session contains one paragraph

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_ensemble_04_multiple_sessions_flat() {
    // Load ensemble document 04
    let corpus = TxxtCorpora::load_document("04-multiple-sessions-flat")
        .expect("Failed to load ensemble 04");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("04-multiple-sessions-flat.txxt".to_string()),
    )
    .expect("Failed to parse ensemble 04");

    println!("=== Parsed Document ===");
    println!("Total elements: {}", document.content.content.len());

    use txxt::ast::elements::session::session_container::SessionContainerElement;

    // DEBUG: Print all elements
    for (i, element) in document.content.content.iter().enumerate() {
        println!("Element {}: {:?}", i, element_type_name(element));
    }

    // Should have exactly 3 sessions at top level
    assert_eq!(
        document.content.content.len(),
        3,
        "Expected 3 elements (sessions), found {}",
        document.content.content.len()
    );

    // First session
    if let SessionContainerElement::Session(session) = &document.content.content[0] {
        println!("\nSession 1 title: \"{}\"", session.title_text());

        assert!(
            session.title_text().contains("First Section"),
            "First session title should contain 'First Section'"
        );

        // Should contain 1 paragraph
        let paragraph_count = session
            .content
            .content
            .iter()
            .filter(|e| matches!(e, SessionContainerElement::Paragraph(_)))
            .count();

        assert_eq!(
            paragraph_count, 1,
            "First session should contain 1 paragraph, found {}",
            paragraph_count
        );

        // Check paragraph content
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let text = extract_text(para);
            println!("  Paragraph: \"{}\"", text);

            assert!(
                text.contains("first section"),
                "First paragraph should contain 'first section'"
            );
            assert!(
                text.contains("complete thought"),
                "First paragraph should mention 'complete thought'"
            );
        } else {
            panic!("First element in session should be a paragraph");
        }
    } else {
        panic!("First element should be a session");
    }

    // Second session
    if let SessionContainerElement::Session(session) = &document.content.content[1] {
        println!("\nSession 2 title: \"{}\"", session.title_text());

        assert!(
            session.title_text().contains("Second Section"),
            "Second session title should contain 'Second Section'"
        );

        // Should contain 1 paragraph
        let paragraph_count = session
            .content
            .content
            .iter()
            .filter(|e| matches!(e, SessionContainerElement::Paragraph(_)))
            .count();

        assert_eq!(
            paragraph_count, 1,
            "Second session should contain 1 paragraph, found {}",
            paragraph_count
        );

        // Check paragraph content
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let text = extract_text(para);
            println!("  Paragraph: \"{}\"", text);

            assert!(
                text.contains("second section"),
                "Second paragraph should contain 'second section'"
            );
            assert!(
                text.contains("same level"),
                "Second paragraph should mention 'same level'"
            );
        } else {
            panic!("First element in session should be a paragraph");
        }
    } else {
        panic!("Second element should be a session");
    }

    // Third session
    if let SessionContainerElement::Session(session) = &document.content.content[2] {
        println!("\nSession 3 title: \"{}\"", session.title_text());

        assert!(
            session.title_text().contains("Third Section"),
            "Third session title should contain 'Third Section'"
        );

        // Should contain 1 paragraph
        let paragraph_count = session
            .content
            .content
            .iter()
            .filter(|e| matches!(e, SessionContainerElement::Paragraph(_)))
            .count();

        assert_eq!(
            paragraph_count, 1,
            "Third session should contain 1 paragraph, found {}",
            paragraph_count
        );

        // Check paragraph content
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let text = extract_text(para);
            println!("  Paragraph: \"{}\"", text);

            assert!(
                text.contains("third section"),
                "Third paragraph should contain 'third section'"
            );
            assert!(
                text.contains("simple document outline"),
                "Third paragraph should mention 'simple document outline'"
            );
        } else {
            panic!("First element in session should be a paragraph");
        }
    } else {
        panic!("Third element should be a session");
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
