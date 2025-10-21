//! Verbatim 01: Simple Verbatim Block
//!
//! Tests basic verbatim block parsing with in-flow mode.

use txxt::transform::run_all;

#[test]
fn test_verbatim_01_simple() {
    // Simple verbatim test - read directly from the example file
    let source = std::fs::read_to_string(
        "docs/specs/elements/verbatim/examples/01-simple-nosession-single-verbatim.txxt",
    )
    .expect("Failed to read verbatim example 01");

    println!("=== Verbatim 01: Simple Verbatim Block ===");
    println!("Source document:\n{}\n", source);

    // Parse through full pipeline
    let document = run_all(&source, Some("verbatim-01.txxt".to_string()))
        .expect("Failed to parse verbatim 01");

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    // Should have 1 top-level element (the verbatim block)
    assert_eq!(
        document.content.content.len(),
        1,
        "Expected 1 top-level element"
    );

    // Verify it's a verbatim block
    let element = &document.content.content[0];
    match element {
        txxt::ast::elements::session::session_container::SessionContainerElement::Verbatim(
            verbatim,
        ) => {
            println!("\n=== Verbatim Block ===");
            println!("Title: {}", verbatim.title_text());
            println!("Label: {}", verbatim.label());
            println!("Content: {}", verbatim.content_text());

            // Verify title
            assert_eq!(verbatim.title_text(), "Code example");

            // Verify label
            assert_eq!(verbatim.label(), "python");

            // Verify content contains the Python code
            let content = verbatim.content_text();
            assert!(content.contains("def hello():"));
            assert!(content.contains("print(\"Hello, world!\")"));

            // Verify it's in-flow mode
            assert!(verbatim.is_in_flow());
            assert!(!verbatim.is_stretched());
        }
        _ => panic!("Expected verbatim block, got: {:?}", element),
    }

    println!("\n✅ Verbatim 01 test passed!");
}
