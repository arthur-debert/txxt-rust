//! Test for Ensemble 12: Complex Sessions
//!
//! This is the most complex parsing test, working through the document in quarters.
//! The document tests ambiguous cases of sessions, lists, and paragraphs.

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use txxt::api::run_all_unified;

/// Test first quarter (lines 1-40): Top-level paragraphs + Session 1 + nested session
#[test]
fn test_complex_sessions_first_quarter() {
    // Create a trimmed version of the first 40 lines
    let source = r#"The most complex part of txxt to parse is the structure being derived from the indentation.
The indentation itself is not the problem, but the lack of explicit syntax for most elements is, which generates various ambiguities.

This document aggregates most of these issues, both for reference and tests. Do not add cases to it.

1. Sources

    There are three elements that cannot be identified by themselves, that is, by only looking at the line they are in. These are:

    - Session titles , hence sessions
    - List items
    - Paragraphs

    All these can have similar forms. While list items do require a list marker (- or 1.), session titles and paragraphs do not and can contain them. Hence, for some variants you can write list items off these three as consistently intertwined.

2. General idea

    The general idea is that sessions create a new , deeper level, thus +1 indented.
    Like here, the "2. General Idea" line itself could be any one of the three. How do you tell them apart?
    The key is that session and lists require other features in the text, if you can't find them, by exclusion the element will be the paragraph.

    1. Sessions

        A session's title has to be preceded by a blank line, it's content is +1 Indented and Sessions cannot be empty. Hence:
            Foo

            Bar

        Foo and Bar can only be paragraphs.

        Any valid session child would be +1 indented.
        Here is the edge Case:
"#;

    let document = run_all_unified(source, Some("12-complex-sessions-q1.txxt".to_string()))
        .expect("Failed to parse first quarter");

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    for (i, element) in document.content.content.iter().enumerate() {
        println!("  {}: {:?}", i, element_type_name(element));
    }

    // Expected structure:
    // - 2 top-level paragraphs (lines 1-2 combined, lines 4-5 combined)
    // - Session "1. Sources" with paragraph + list + paragraph
    // - Session "2. General idea" with paragraph + nested session "1. Sessions"
    //
    // Note: Consecutive PlainTextLines are combined into a single paragraph until
    // a blank line or other element terminates them

    use txxt::ast::elements::session::session_container::SessionContainerElement;

    // Should have 4 top-level elements: 2 paragraphs + 2 sessions
    assert_eq!(
        document.content.content.len(),
        4,
        "Expected 4 top-level elements (2 paragraphs + 2 sessions), got {}",
        document.content.content.len()
    );

    // Element 0: First paragraph (lines 1-2 combined)
    assert!(
        matches!(
            document.content.content[0],
            SessionContainerElement::Paragraph(_)
        ),
        "Element 0 should be paragraph"
    );

    // Element 1: Second paragraph (lines 4-5 combined)
    assert!(
        matches!(
            document.content.content[1],
            SessionContainerElement::Paragraph(_)
        ),
        "Element 1 should be paragraph"
    );

    // Element 2: Session "1. Sources"
    if let SessionContainerElement::Session(session) = &document.content.content[2] {
        assert!(
            session.title_text().contains("Sources"),
            "Session 2 should be 'Sources'"
        );

        println!("\n=== Session 1: Sources ===");
        println!("Children: {}", session.content.content.len());
        for (i, child) in session.content.content.iter().enumerate() {
            println!("  Child {}: {:?}", i, element_type_name(child));
        }

        // Session 1 should have: paragraph, list (3 items), paragraph
        assert_eq!(
            session.content.content.len(),
            3,
            "Session 1 should have 3 children (para + list + para)"
        );

        // Child 0: Paragraph
        assert!(
            matches!(
                session.content.content[0],
                SessionContainerElement::Paragraph(_)
            ),
            "Session 1, child 0 should be paragraph"
        );

        // Child 1: List with 3 items
        if let SessionContainerElement::List(list) = &session.content.content[1] {
            assert_eq!(list.items.len(), 3, "List should have 3 items");
        } else {
            panic!("Session 1, child 1 should be list");
        }

        // Child 2: Paragraph
        assert!(
            matches!(
                session.content.content[2],
                SessionContainerElement::Paragraph(_)
            ),
            "Session 1, child 2 should be paragraph"
        );
    } else {
        panic!("Element 2 should be Session");
    }

    // Element 3: Session "2. General idea"
    if let SessionContainerElement::Session(session) = &document.content.content[3] {
        assert!(
            session.title_text().contains("General idea"),
            "Session 3 should be 'General idea'"
        );

        println!("\n=== Session 2: General idea ===");
        println!("Children: {}", session.content.content.len());
        for (i, child) in session.content.content.iter().enumerate() {
            println!("  Child {}: {:?}", i, element_type_name(child));
        }

        // Session 2 should have: 1 paragraph (lines 19-21 combined) + nested session
        // Content after the nested session should be inside it
        println!("Session 2 child count: {}", session.content.content.len());

        // Find the nested session (should be one of the children)
        let nested_session_index = session
            .content
            .content
            .iter()
            .position(|child| matches!(child, SessionContainerElement::Session(_)))
            .expect("Should have nested session");

        println!("Nested session at index: {}", nested_session_index);

        // Get the nested session
        if let SessionContainerElement::Session(nested) =
            &session.content.content[nested_session_index]
        {
            assert!(
                nested.title_text().contains("Sessions"),
                "Nested session should be 'Sessions'"
            );

            println!("\n=== Nested Session: 1. Sessions ===");
            println!("Children: {}", nested.content.content.len());
            for (i, child) in nested.content.content.iter().enumerate() {
                println!("  Child {}: {:?}", i, element_type_name(child));
            }

            // Nested session should have paragraphs
            assert!(
                !nested.content.content.is_empty(),
                "Nested session should have children"
            );
        } else {
            panic!("Nested session child should be Session");
        }
    } else {
        panic!("Element 3 should be Session");
    }

    println!("\n✅ First quarter parsed successfully!");
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
