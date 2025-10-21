//! Test for Ensemble 06: Nested Sessions Multiple
//!
//! This test validates complex nested session structures:
//! - 3 levels of nesting (session > session > session)
//! - Multiple paragraphs within sessions
//! - Mixed content (paragraphs and sessions at same level)
//!
//! Structure:
//! - Session "Main Topic"
//!   - Paragraph (intro)
//!   - Session "First Subtopic"
//!     - Paragraph 1
//!     - Paragraph 2
//!   - Session "Second Subtopic"
//!     - Paragraph 1
//!     - Paragraph 2
//!     - Session "Third Subtopic" (3 levels deep!)
//!       - Paragraph
//! - Session "Another Main Topic"
//!   - Paragraph

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_ensemble_06_nested_sessions_multiple() {
    // Load ensemble document 06
    let corpus = TxxtCorpora::load_document("06-nested-sessions-multiple")
        .expect("Failed to load ensemble 06");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("06-nested-sessions-multiple.txxt".to_string()),
    )
    .expect("Failed to parse ensemble 06");

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
    // First session: "Main Topic"
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

        // ========================================
        // Second child: Nested session "First Subtopic"
        // ========================================
        if let SessionContainerElement::Session(subsession) = &session.content.content[1] {
            println!("\n  === Nested Session: First Subtopic ===");
            println!("  Title: \"{}\"", subsession.title_text());

            assert!(
                subsession.title_text().contains("First Subtopic"),
                "First nested session should contain 'First Subtopic'"
            );

            // Debug: Print children
            println!("  Children count: {}", subsession.content.content.len());
            for (i, child) in subsession.content.content.iter().enumerate() {
                println!("    Child {}: {:?}", i, element_type_name(child));
            }

            // Should have 2 paragraphs
            let para_count = subsession
                .content
                .content
                .iter()
                .filter(|e| matches!(e, SessionContainerElement::Paragraph(_)))
                .count();

            assert_eq!(
                para_count, 2,
                "First subsection should have 2 paragraphs, found {}",
                para_count
            );

            // First paragraph
            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[0] {
                let text = extract_text(para);
                println!("    Paragraph 1: \"{}\"", text);

                assert!(
                    text.contains("first aspect"),
                    "Should mention 'first aspect'"
                );
            } else {
                panic!("First child should be a paragraph");
            }

            // Second paragraph
            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[1] {
                let text = extract_text(para);
                println!("    Paragraph 2: \"{}\"", text);

                assert!(
                    text.contains("continuation"),
                    "Should mention 'continuation'"
                );
            } else {
                panic!("Second child should be a paragraph");
            }
        } else {
            panic!("Second child should be a nested session");
        }

        // ========================================
        // Third child: Nested session "Second Subtopic"
        // ========================================
        if let SessionContainerElement::Session(subsession) = &session.content.content[2] {
            println!("\n  === Nested Session: Second Subtopic ===");
            println!("  Title: \"{}\"", subsession.title_text());

            assert!(
                subsession.title_text().contains("Second Subtopic"),
                "Second nested session should contain 'Second Subtopic'"
            );

            // Debug: Print children
            println!("  Children count: {}", subsession.content.content.len());
            for (i, child) in subsession.content.content.iter().enumerate() {
                println!("    Child {}: {:?}", i, element_type_name(child));
            }

            // Should have 2 paragraphs + 1 session = 3 children
            assert_eq!(
                subsession.content.content.len(),
                3,
                "Second subsection should have 3 children (2 paragraphs + 1 session), found {}",
                subsession.content.content.len()
            );

            // First paragraph
            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[0] {
                let text = extract_text(para);
                println!("    Paragraph 1: \"{}\"", text);

                assert!(
                    text.contains("second aspect"),
                    "Should mention 'second aspect'"
                );
            } else {
                panic!("First child should be a paragraph");
            }

            // Second paragraph
            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[1] {
                let text = extract_text(para);
                println!("    Paragraph 2: \"{}\"", text);

                assert!(
                    text.contains("three level deep"),
                    "Should mention 'three level deep'"
                );
            } else {
                panic!("Second child should be a paragraph");
            }

            // ========================================
            // Third child: Deeply nested session "Third Subtopic" (3 levels!)
            // ========================================
            if let SessionContainerElement::Session(deep_session) = &subsession.content.content[2] {
                println!("\n    === Deeply Nested Session: Third Subtopic (Level 3!) ===");
                println!("    Title: \"{}\"", deep_session.title_text());

                assert!(
                    deep_session.title_text().contains("Third Subtopic"),
                    "Third nested session should contain 'Third Subtopic'"
                );

                // Should have 1 paragraph
                assert_eq!(
                    deep_session.content.content.len(),
                    1,
                    "Third subsection should have 1 paragraph, found {}",
                    deep_session.content.content.len()
                );

                if let SessionContainerElement::Paragraph(para) = &deep_session.content.content[0] {
                    let text = extract_text(para);
                    println!("      Paragraph: \"{}\"", text);

                    assert!(text.contains("rabit whole"), "Should mention 'rabit whole'");
                } else {
                    panic!("Should contain a paragraph");
                }
            } else {
                panic!("Third child should be a deeply nested session");
            }
        } else {
            panic!("Third child should be a nested session");
        }
    } else {
        panic!("First element should be a session");
    }

    // ========================================
    // Second session: "Another Main Topic"
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
