//! Test for simple definition parsing

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use txxt::ast::elements::containers::simple::SimpleBlockElement;
use txxt::transform::run_all;

#[test]
fn test_definition_simple() {
    // Simple definition from spec (new syntax: single colon)
    let source = r#"Parser:
    A program that analyzes text according to formal grammar rules to create a structured representation like an Abstract Syntax Tree.
"#;

    println!("=== Simple Definition Test ===");
    println!("Source:\n{}", source);
    println!();

    // Parse through full pipeline
    let document = run_all(source, Some("definition-simple.txxt".to_string()))
        .expect("Failed to parse simple definition");

    use txxt::ast::elements::session::session_container::SessionContainerElement;

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    for (i, element) in document.content.content.iter().enumerate() {
        println!("  {}: {:?}", i, element_type_name(element));
    }

    // Should have exactly 1 definition
    assert_eq!(
        document.content.content.len(),
        1,
        "Expected 1 definition, got {}",
        document.content.content.len()
    );

    // Verify it's a definition
    if let SessionContainerElement::Definition(def) = &document.content.content[0] {
        println!("\n=== Definition ===");
        let term_text: String = def.term.content.iter().map(|t| t.text_content()).collect();
        println!("Term: {}", term_text);
        assert!(term_text.contains("Parser"), "Term should be 'Parser'");

        println!("Content elements: {}", def.content.content.len());
        assert_eq!(
            def.content.content.len(),
            1,
            "Should have 1 paragraph of content"
        );

        // Check content is a paragraph
        if let SimpleBlockElement::Paragraph(para) = &def.content.content[0] {
            let content_text: String = para.content.iter().map(|t| t.text_content()).collect();
            println!("Content: {}", content_text);
            assert!(content_text.contains("program that analyzes text"));
        } else {
            panic!("Definition content should be a paragraph");
        }
    } else {
        panic!("Element should be a Definition");
    }

    println!("\n✅ Simple definition test passed!");
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
