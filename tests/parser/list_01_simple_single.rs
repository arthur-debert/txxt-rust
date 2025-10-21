//! Test for List 01: Simple Single List (No Session)
//!
//! This test validates basic list parsing at the top level:
//! - 3 items with plain `-` markers
//! - No session container
//! - Simplest possible list structure

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::transform::run_all;

#[test]
fn test_list_01_simple_single() {
    // Load list example 01
    let corpus = TxxtCorpora::load_document("list-01-simple-single")
        .expect("Failed to load list example 01");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("===================\n");

    // Parse through full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("01-simple-nosession-single-list.txxt".to_string()),
    )
    .expect("Failed to parse list example 01");

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

    // Should have exactly 1 list at top level
    assert_eq!(
        document.content.content.len(),
        1,
        "Expected 1 top-level list, found {}",
        document.content.content.len()
    );

    // First element: List with 3 items
    if let SessionContainerElement::List(list) = &document.content.content[0] {
        println!("\n=== List ===");
        println!("List items count: {}", list.items.len());

        // Should have exactly 3 items
        assert_eq!(
            list.items.len(),
            3,
            "List should have 3 items, found {}",
            list.items.len()
        );

        // Check each item
        let item1_text = extract_list_item_text(&list.items[0]);
        println!("  Item 1: \"{}\"", item1_text);
        assert!(
            item1_text.contains("First item"),
            "Item 1 should contain 'First item'"
        );

        let item2_text = extract_list_item_text(&list.items[1]);
        println!("  Item 2: \"{}\"", item2_text);
        assert!(
            item2_text.contains("Second item"),
            "Item 2 should contain 'Second item'"
        );

        let item3_text = extract_list_item_text(&list.items[2]);
        println!("  Item 3: \"{}\"", item3_text);
        assert!(
            item3_text.contains("Third item"),
            "Item 3 should contain 'Third item'"
        );

        println!("\n✅ All assertions passed!");
    } else {
        panic!("First element should be a list");
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

/// Helper to extract text from list item
fn extract_list_item_text(item: &txxt::ast::elements::list::ListItem) -> String {
    item.content
        .iter()
        .map(|t| t.text_content())
        .collect::<Vec<_>>()
        .join("")
}
