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

    // Expected: 7 top-level elements (2 paragraphs + 5 sessions)
    assert_eq!(
        document.content.content.len(),
        7,
        "Expected 7 top-level elements, got {}",
        document.content.content.len()
    );

    // Verify structure in detail
    assert!(matches!(
        document.content.content[0],
        SessionContainerElement::Paragraph(_)
    ));
    assert!(matches!(
        document.content.content[1],
        SessionContainerElement::Paragraph(_)
    ));

    // Session 1: "1. Sources"
    if let SessionContainerElement::Session(session) = &document.content.content[2] {
        println!("\n=== Session: Sources ===");
        assert!(session.title_text().contains("Sources"));
        println!("Children: {}", session.content.content.len());

        // Should have: paragraph + list + paragraph = 3
        assert_eq!(
            session.content.content.len(),
            3,
            "Sources should have 3 children"
        );
        assert!(matches!(
            session.content.content[0],
            SessionContainerElement::Paragraph(_)
        ));
        assert!(matches!(
            session.content.content[1],
            SessionContainerElement::List(_)
        ));
        assert!(matches!(
            session.content.content[2],
            SessionContainerElement::Paragraph(_)
        ));
    } else {
        panic!("Element 2 should be Session 'Sources'");
    }

    // Session 2: "2. General idea" (with nested session)
    if let SessionContainerElement::Session(session) = &document.content.content[3] {
        println!("\n=== Session: General idea ===");
        assert!(session.title_text().contains("General idea"));
        println!("Children: {}", session.content.content.len());

        // Should have: paragraph + nested session "1. Sessions" = 2
        assert_eq!(
            session.content.content.len(),
            2,
            "General idea should have 2 children"
        );
        assert!(matches!(
            session.content.content[0],
            SessionContainerElement::Paragraph(_)
        ));

        // Check nested session "1. Sessions"
        if let SessionContainerElement::Session(nested) = &session.content.content[1] {
            println!("  Nested session: {}", nested.title_text());
            assert!(nested.title_text().contains("Sessions"));
            println!("  Nested children: {}", nested.content.content.len());

            // Should have multiple paragraphs and lists
            assert!(
                !nested.content.content.is_empty(),
                "Nested session should have content"
            );
        } else {
            panic!("Child 1 should be nested session 'Sessions'");
        }
    } else {
        panic!("Element 3 should be Session 'General idea'");
    }

    // Session 3: "2. Lists"
    if let SessionContainerElement::Session(session) = &document.content.content[4] {
        println!("\n=== Session: Lists ===");
        assert!(session.title_text().contains("Lists"));
        println!("Children: {}", session.content.content.len());

        // Should have paragraphs and lists
        assert!(
            !session.content.content.is_empty(),
            "Lists should have content"
        );
    } else {
        panic!("Element 4 should be Session 'Lists'");
    }

    // Session 4: "3. Paragraphs: Dialogs..."
    if let SessionContainerElement::Session(session) = &document.content.content[5] {
        println!("\n=== Session: Paragraphs ===");
        assert!(session.title_text().contains("Paragraphs"));
        println!("Children: {}", session.content.content.len());

        assert!(
            !session.content.content.is_empty(),
            "Paragraphs should have content"
        );
    } else {
        panic!("Element 5 should be Session 'Paragraphs'");
    }

    // Session 5: "4. .Indentation and Containers" (with nested sessions)
    if let SessionContainerElement::Session(session) = &document.content.content[6] {
        println!("\n=== Session: Indentation and Containers ===");
        assert!(session.title_text().contains("Indentation"));
        println!("Children: {}", session.content.content.len());

        // Should have paragraphs + nested sessions (4.1, 4.2)
        assert!(
            session.content.content.len() >= 5,
            "Indentation should have multiple children"
        );

        // Check for nested sessions
        let nested_sessions: Vec<_> = session
            .content
            .content
            .iter()
            .filter(|e| matches!(e, SessionContainerElement::Session(_)))
            .collect();

        println!("  Nested sessions: {}", nested_sessions.len());
        for nested in &nested_sessions {
            if let SessionContainerElement::Session(s) = nested {
                println!("    - {}", s.title_text());
            }
        }

        assert!(
            nested_sessions.len() >= 2,
            "Should have at least 2 nested sessions (4.1, 4.2)"
        );
    } else {
        panic!("Element 6 should be Session 'Indentation'");
    }

    println!("\n✅ All structure assertions passed!");
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
