//! Test for Ensemble 05: Nested Sessions Basic
//!
//! This test validates nested session structure:
//! - Parent session contains paragraph + 2 child sessions
//! - Each child session contains one paragraph
//! - Second top-level session is a peer (flat structure)
//!
//! Structure:
//! - Session "1. Main Topic"
//!   - Paragraph (intro)
//!   - Session "1.1. First Subtopic"
//!     - Paragraph
//!   - Session "1.2. Second Subtopic"
//!     - Paragraph
//! - Session "2. Another Main Topic"
//!   - Paragraph

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_ensemble_05_nested_sessions_basic() {
    // Load ensemble document 05
    let corpus =
        TxxtCorpora::load_document("05-nested-sessions-basic").expect("Failed to load ensemble 05");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("05-nested-sessions-basic.txxt".to_string()),
    )
    .expect("Failed to parse ensemble 05");

    println!("=== Parsed Document ===");
    println!(
        "Total top-level elements: {}",
        document.content.content.len()
    );

    use txxt::ast::elements::session::session_container::SessionContainerElement;

    // DEBUG: Print all top-level elements
    for (i, element) in document.content.content.iter().enumerate() {
        println!("Top-level element {}: {:?}", i, element_type_name(element));
    }

    // Should have exactly 2 sessions at top level
    assert_eq!(
        document.content.content.len(),
        2,
        "Expected 2 top-level sessions, found {}",
        document.content.content.len()
    );

    // ========================================
    // First session: "1. Main Topic"
    // ========================================
    if let SessionContainerElement::Session(session) = &document.content.content[0] {
        println!("\n=== Session 1: Main Topic ===");
        println!("Title: \"{}\"", session.title_text());

        assert!(
            session.title_text().contains("Main Topic"),
            "First session title should contain 'Main Topic'"
        );

        // Debug: Print all children
        println!("Children count: {}", session.content.content.len());
        for (i, child) in session.content.content.iter().enumerate() {
            println!("  Child {}: {:?}", i, element_type_name(child));
        }

        // Should contain: 1 paragraph + 2 nested sessions = 3 children
        assert_eq!(
            session.content.content.len(),
            3,
            "Main topic should have 3 children (1 paragraph + 2 sessions), found {}",
            session.content.content.len()
        );

        // First child: Paragraph (intro)
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let text = extract_text(para);
            println!("  Intro paragraph: \"{}\"", text);

            assert!(
                text.contains("introduction to the main topic"),
                "Intro paragraph should mention 'introduction to the main topic'"
            );
        } else {
            panic!("First child should be a paragraph");
        }

        // Second child: Nested session "1.1. First Subtopic"
        if let SessionContainerElement::Session(subsession) = &session.content.content[1] {
            println!("\n  === Nested Session 1.1 ===");
            println!("  Title: \"{}\"", subsession.title_text());

            assert!(
                subsession.title_text().contains("First Subtopic"),
                "First nested session should contain 'First Subtopic'"
            );

            // Should have 1 paragraph
            assert_eq!(
                subsession.content.content.len(),
                1,
                "First subsection should have 1 paragraph, found {}",
                subsession.content.content.len()
            );

            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[0] {
                let text = extract_text(para);
                println!("    Paragraph: \"{}\"", text);

                assert!(
                    text.contains("first aspect"),
                    "Should mention 'first aspect'"
                );
                assert!(
                    text.contains("indented one level deeper"),
                    "Should mention 'indented one level deeper'"
                );
            } else {
                panic!("Subsection should contain a paragraph");
            }
        } else {
            panic!("Second child should be a nested session");
        }

        // Third child: Nested session "1.2. Second Subtopic"
        if let SessionContainerElement::Session(subsession) = &session.content.content[2] {
            println!("\n  === Nested Session 1.2 ===");
            println!("  Title: \"{}\"", subsession.title_text());

            assert!(
                subsession.title_text().contains("Second Subtopic"),
                "Second nested session should contain 'Second Subtopic'"
            );

            // Should have 1 paragraph
            assert_eq!(
                subsession.content.content.len(),
                1,
                "Second subsection should have 1 paragraph, found {}",
                subsession.content.content.len()
            );

            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[0] {
                let text = extract_text(para);
                println!("    Paragraph: \"{}\"", text);

                assert!(
                    text.contains("second aspect"),
                    "Should mention 'second aspect'"
                );
                assert!(
                    text.contains("hierarchical numbering"),
                    "Should mention 'hierarchical numbering'"
                );
            } else {
                panic!("Subsection should contain a paragraph");
            }
        } else {
            panic!("Third child should be a nested session");
        }
    } else {
        panic!("First element should be a session");
    }

    // ========================================
    // Second session: "2. Another Main Topic"
    // ========================================
    if let SessionContainerElement::Session(session) = &document.content.content[1] {
        println!("\n=== Session 2: Another Main Topic ===");
        println!("Title: \"{}\"", session.title_text());

        assert!(
            session.title_text().contains("Another Main Topic"),
            "Second session title should contain 'Another Main Topic'"
        );

        // Should have 1 paragraph
        assert_eq!(
            session.content.content.len(),
            1,
            "Another Main Topic should have 1 paragraph, found {}",
            session.content.content.len()
        );

        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let text = extract_text(para);
            println!("  Paragraph: \"{}\"", text);

            assert!(
                text.contains("second main topic"),
                "Should mention 'second main topic'"
            );
            assert!(
                text.contains("nested and flat structures"),
                "Should mention 'nested and flat structures'"
            );
        } else {
            panic!("Should contain a paragraph");
        }
    } else {
        panic!("Second element should be a session");
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
