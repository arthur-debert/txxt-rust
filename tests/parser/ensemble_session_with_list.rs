//! Test for Ensemble: Session with List
//!
//! This test introduces list pattern matching:
//! - List requires 2+ consecutive SequenceTextLine tokens
//! - No blank lines between list items
//! - List embedded within session content
//!
//! Structure:
//! - Session "1. Key Features"
//!   - Paragraph (intro)
//!   - List (4 items with `-` markers)
//!   - Paragraph (outro)

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::api::run_all_unified;

#[test]
fn test_ensemble_session_with_list() {
    // Load ensemble document
    let corpus = TxxtCorpora::load_document("06-session-with-list")
        .expect("Failed to load session-with-list ensemble");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all_unified(
        &corpus.source_text,
        Some("06-session-with-list.txxt".to_string()),
    )
    .expect("Failed to parse session-with-list ensemble");

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

    // Should have exactly 1 session at top level
    assert_eq!(
        document.content.content.len(),
        1,
        "Expected 1 top-level session, found {}",
        document.content.content.len()
    );

    // ========================================
    // Session: "1. Key Features"
    // ========================================
    if let SessionContainerElement::Session(session) = &document.content.content[0] {
        println!("\n=== Session: Key Features ===");
        println!("Title: \"{}\"", session.title_text());

        assert!(
            session.title_text().contains("Key Features"),
            "Session title should contain 'Key Features'"
        );

        // Debug: Print all children
        println!("Children count: {}", session.content.content.len());
        for (i, child) in session.content.content.iter().enumerate() {
            println!("  Child {}: {:?}", i, element_type_name(child));
        }

        // Should contain: 1 paragraph + 1 list + 1 paragraph = 3 children
        assert_eq!(
            session.content.content.len(),
            3,
            "Session should have 3 children (paragraph + list + paragraph), found {}",
            session.content.content.len()
        );

        // First child: Paragraph (intro)
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let text = extract_text(para);
            println!("  Intro paragraph: \"{}\"", text);

            assert!(
                text.contains("demonstrates how lists integrate"),
                "Intro should mention 'demonstrates how lists integrate'"
            );
        } else {
            panic!("First child should be a paragraph");
        }

        // Second child: List (4 items)
        if let SessionContainerElement::List(list) = &session.content.content[1] {
            println!("\n  === List ===");
            println!("  List items count: {}", list.items.len());

            // Should have exactly 4 items
            assert_eq!(
                list.items.len(),
                4,
                "List should have 4 items, found {}",
                list.items.len()
            );

            // Check each item
            let item1_text = extract_list_item_text(&list.items[0]);
            println!("    Item 1: \"{}\"", item1_text);
            assert!(
                item1_text.contains("First feature"),
                "Item 1 should contain 'First feature'"
            );
            assert!(
                item1_text.contains("Easy to read"),
                "Item 1 should contain 'Easy to read'"
            );

            let item2_text = extract_list_item_text(&list.items[1]);
            println!("    Item 2: \"{}\"", item2_text);
            assert!(
                item2_text.contains("Second feature"),
                "Item 2 should contain 'Second feature'"
            );
            assert!(
                item2_text.contains("Minimal syntax"),
                "Item 2 should contain 'Minimal syntax'"
            );

            let item3_text = extract_list_item_text(&list.items[2]);
            println!("    Item 3: \"{}\"", item3_text);
            assert!(
                item3_text.contains("Third feature"),
                "Item 3 should contain 'Third feature'"
            );
            assert!(
                item3_text.contains("hierarchical structure"),
                "Item 3 should contain 'hierarchical structure'"
            );

            let item4_text = extract_list_item_text(&list.items[3]);
            println!("    Item 4: \"{}\"", item4_text);
            assert!(
                item4_text.contains("Fourth feature"),
                "Item 4 should contain 'Fourth feature'"
            );
            assert!(
                item4_text.contains("Plain text format"),
                "Item 4 should contain 'Plain text format'"
            );
        } else {
            panic!("Second child should be a list");
        }

        // Third child: Paragraph (outro)
        if let SessionContainerElement::Paragraph(para) = &session.content.content[2] {
            let text = extract_text(para);
            println!("\n  Outro paragraph: \"{}\"", text);

            assert!(
                text.contains("well-suited for technical documentation"),
                "Outro should mention 'well-suited for technical documentation'"
            );
        } else {
            panic!("Third child should be a paragraph");
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

/// Helper to extract text from list item
fn extract_list_item_text(item: &txxt::ast::elements::list::ListItem) -> String {
    item.content
        .iter()
        .map(|t| t.text_content())
        .collect::<Vec<_>>()
        .join("")
}
