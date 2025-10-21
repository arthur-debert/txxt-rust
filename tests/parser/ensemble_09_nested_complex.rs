//! Ensemble 09: Nested Complex
//!
//! Tests parsing of deeply nested document structures with multiple element types.
//! Document structure:
//! - Session: "1. Complex Document Structure"
//!   - Paragraph
//!   - Session: "1.1. First Major Subsection"
//!     - Paragraph
//!     - List (nested multi-level)
//!     - Paragraph
//!     - Definition: "Nested Definition ::"
//!       - Paragraph
//!   - Session: "1.2. Second Major Subsection"
//!     - Paragraph
//!     - Session: "1.2.1. Third Level Section"
//!       - Paragraph
//!       - List
//! - Session: "2. Conclusion"
//!   - Paragraph

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::ast::elements::containers::content::ContentContainerElement;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::transform::run_all;

#[test]
fn test_ensemble_09_nested_complex() {
    // Load ensemble document 09
    let corpus =
        TxxtCorpora::load_document("09-nested-complex").expect("Failed to load ensemble 09");

    let source = &corpus.source_text;

    println!("=== Ensemble 09: Nested Complex ===");
    println!("Source document:\n{}", source);
    println!();

    // Parse through full pipeline
    let document =
        run_all(source, Some("ensemble-09.txxt".to_string())).expect("Failed to parse ensemble 09");

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    // Should have 2 top-level sessions
    assert_eq!(
        document.content.content.len(),
        2,
        "Expected 2 top-level sessions"
    );

    // ========== Session 1: "1. Complex Document Structure" ==========
    if let SessionContainerElement::Session(session) = &document.content.content[0] {
        println!("\n=== Session 1: '1. Complex Document Structure' ===");

        let title_text: String = session
            .title
            .content
            .iter()
            .map(|t| t.text_content())
            .collect();
        println!("Title: {}", title_text);
        assert!(
            title_text.contains("Complex Document Structure"),
            "First session title should be 'Complex Document Structure'"
        );

        // Should have: paragraph + 2 nested sessions
        assert_eq!(
            session.content.content.len(),
            3,
            "Session 1 should have 3 elements: paragraph + 2 nested sessions"
        );

        // Element 0: Intro paragraph
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
            println!("\n[0] Paragraph: {}", para_text);
            assert!(para_text.contains("demonstrates deep nesting"));
        } else {
            panic!("Element 0 should be a Paragraph");
        }

        // Element 1: Session "1.1. First Major Subsection"
        if let SessionContainerElement::Session(subsession) = &session.content.content[1] {
            println!("\n[1] Session 1.1:");
            let sub_title: String = subsession
                .title
                .content
                .iter()
                .map(|t| t.text_content())
                .collect();
            println!("  Title: {}", sub_title);
            assert!(sub_title.contains("First Major Subsection"));

            // Should have: paragraph + list + paragraph + definition
            assert_eq!(
                subsession.content.content.len(),
                4,
                "Session 1.1 should have 4 elements"
            );

            // Element 0: Intro paragraph
            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[0] {
                let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
                println!("  [0] Paragraph: {}", para_text);
                assert!(para_text.contains("nested list"));
            } else {
                panic!("Session 1.1 element 0 should be a Paragraph");
            }

            // Element 1: First list with nested content
            if let SessionContainerElement::List(list) = &subsession.content.content[1] {
                println!("  [1] List:");
                println!("      Items: {}", list.items.len());
                assert_eq!(list.items.len(), 2, "Top level list should have 2 items");

                // Validate first item has nested content
                assert!(
                    list.items[0].has_nested_content(),
                    "First list item should have nested content (numbered sub-list)"
                );

                if let Some(nested_container) = &list.items[0].nested {
                    println!(
                        "      First item has {} nested elements",
                        nested_container.content.len()
                    );
                    assert!(
                        !nested_container.content.is_empty(),
                        "Nested container should have content"
                    );

                    // The nested content should be a list
                    use txxt::ast::elements::containers::content::ContentContainerElement;
                    let has_nested_list = nested_container
                        .content
                        .iter()
                        .any(|elem| matches!(elem, ContentContainerElement::List(_)));
                    assert!(has_nested_list, "Nested content should contain a list");
                }
            } else {
                panic!("Session 1.1 element 1 should be a List");
            }

            // Element 2: Paragraph (transition text)
            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[2] {
                let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
                println!("  [2] Paragraph: {}", para_text);
                assert!(para_text.contains("Following the list"));
            } else {
                panic!("Session 1.1 element 2 should be a Paragraph");
            }

            // Element 3: Definition
            if let SessionContainerElement::Definition(def) = &subsession.content.content[3] {
                println!("  [3] Definition:");
                let term_text: String = def.term.content.iter().map(|t| t.text_content()).collect();
                println!("      Term: {}", term_text);
                assert!(term_text.contains("Nested Definition"));

                assert_eq!(
                    def.content.content.len(),
                    1,
                    "Definition should have 1 paragraph"
                );
                if let ContentContainerElement::Paragraph(para) = &def.content.content[0] {
                    let content_text: String =
                        para.content.iter().map(|t| t.text_content()).collect();
                    println!("      Content: {}", content_text);
                    assert!(content_text.contains("within a subsection"));
                } else {
                    panic!("Definition content should be a paragraph");
                }
            } else {
                panic!("Session 1.1 element 3 should be a Definition");
            }
        } else {
            panic!("Element 1 should be a Session (1.1)");
        }

        // Element 2: Session "1.2. Second Major Subsection"
        if let SessionContainerElement::Session(subsession) = &session.content.content[2] {
            println!("\n[2] Session 1.2:");
            let sub_title: String = subsession
                .title
                .content
                .iter()
                .map(|t| t.text_content())
                .collect();
            println!("  Title: {}", sub_title);
            assert!(sub_title.contains("Second Major Subsection"));

            // Should have: paragraph + nested session
            assert_eq!(
                subsession.content.content.len(),
                2,
                "Session 1.2 should have 2 elements"
            );

            // Element 0: Paragraph
            if let SessionContainerElement::Paragraph(para) = &subsession.content.content[0] {
                let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
                println!("  [0] Paragraph: {}", para_text);
                assert!(para_text.contains("even deeper nesting"));
            } else {
                panic!("Session 1.2 element 0 should be a Paragraph");
            }

            // Element 1: Session "1.2.1. Third Level Section"
            if let SessionContainerElement::Session(third_level) = &subsession.content.content[1] {
                println!("  [1] Session 1.2.1:");
                let third_title: String = third_level
                    .title
                    .content
                    .iter()
                    .map(|t| t.text_content())
                    .collect();
                println!("      Title: {}", third_title);
                assert!(third_title.contains("Third Level Section"));

                // Should have: 2 paragraphs + list
                assert_eq!(
                    third_level.content.content.len(),
                    3,
                    "Session 1.2.1 should have 3 elements"
                );

                // Element 0: Paragraph
                if let SessionContainerElement::Paragraph(para) = &third_level.content.content[0] {
                    let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
                    println!("      [0] Paragraph: {}", para_text);
                    assert!(para_text.contains("third level of nesting"));
                } else {
                    panic!("Session 1.2.1 element 0 should be a Paragraph");
                }

                // Element 1: Paragraph ("Key concepts:")
                if let SessionContainerElement::Paragraph(para) = &third_level.content.content[1] {
                    let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
                    println!("      [1] Paragraph: {}", para_text);
                    assert!(para_text.contains("Key concepts"));
                } else {
                    panic!("Session 1.2.1 element 1 should be a Paragraph");
                }

                // Element 2: List
                if let SessionContainerElement::List(list) = &third_level.content.content[2] {
                    println!("      [2] List:");
                    println!("          Items: {}", list.items.len());
                    assert_eq!(list.items.len(), 3, "List should have 3 items");

                    let first_item_text: String = list.items[0]
                        .content
                        .iter()
                        .map(|t| t.text_content())
                        .collect();
                    println!("          Item 1: {}", first_item_text);
                    assert!(first_item_text.contains("Depth is indicated"));
                } else {
                    panic!("Session 1.2.1 element 1 should be a List");
                }
            } else {
                panic!("Session 1.2 element 1 should be a Session (1.2.1)");
            }
        } else {
            panic!("Element 2 should be a Session (1.2)");
        }
    } else {
        panic!("Top-level element 0 should be a Session");
    }

    // ========== Session 2: "2. Conclusion" ==========
    if let SessionContainerElement::Session(session) = &document.content.content[1] {
        println!("\n=== Session 2: '2. Conclusion' ===");

        let title_text: String = session
            .title
            .content
            .iter()
            .map(|t| t.text_content())
            .collect();
        println!("Title: {}", title_text);
        assert!(
            title_text.contains("Conclusion"),
            "Second session title should be 'Conclusion'"
        );

        // Should have 1 paragraph
        assert_eq!(
            session.content.content.len(),
            1,
            "Session 2 should have 1 paragraph"
        );

        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
            println!("[0] Paragraph: {}", para_text);
            assert!(para_text.contains("wraps up the complex document"));
        } else {
            panic!("Session 2 element 0 should be a Paragraph");
        }
    } else {
        panic!("Top-level element 1 should be a Session");
    }

    println!("\n✅ Ensemble 09 test passed!");
}
