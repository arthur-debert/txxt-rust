//! Test to demonstrate missing paragraph issue with two-paragraphs document

mod corpora {
    include!("../../../infrastructure/corpora.rs");
}

use corpora::TxxtCorpora;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::transform::run_all;

/// Test that demonstrates the missing second paragraph issue
#[test]
#[ignore = "No parsing is really working"]
fn test_two_paragraphs_missing_second() {
    // Load the two paragraphs ensemble document
    let corpus = TxxtCorpora::load_document("01-two-paragraphs")
        .expect("Failed to load two-paragraphs document");

    println!("=== Source Text ===");
    println!("{}", corpus.source_text);
    println!("==================");

    // Parse through the full pipeline
    let document = run_all(
        &corpus.source_text,
        Some("01-two-paragraphs.txxt".to_string()),
    )
    .expect("Failed to parse document");

    println!("=== Parsed Document ===");
    println!("Element count: {}", document.content.content.len());

    // Walk through all elements and identify paragraphs
    let mut paragraph_count = 0;
    let mut paragraph_texts = Vec::new();

    for (i, element) in document.content.content.iter().enumerate() {
        println!("Element {}: {:?}", i, element_type_name(element));

        if let SessionContainerElement::Paragraph(paragraph) = element {
            paragraph_count += 1;

            // Extract text content from the paragraph
            let text_content = extract_paragraph_text(paragraph);
            paragraph_texts.push(text_content.clone());

            println!("  Paragraph {}: \"{}\"", paragraph_count, text_content);
            println!("  Content transforms: {}", paragraph.content.len());
        }
    }

    println!("=======================");
    println!("Total paragraphs found: {}", paragraph_count);

    // According to the specification and the source text, we should have 2 paragraphs
    let expected_paragraph_count = 2;
    let expected_texts = ["This is the first paragraph of a simple document. It contains plain text without any special formatting or structure.",
        "This is the second paragraph. It is separated from the first paragraph by a blank line, which is the standard way to separate paragraphs in txxt."];

    // Print what we expect vs what we got
    println!("\n=== Expected vs Actual ===");
    println!("Expected paragraphs: {}", expected_paragraph_count);
    println!("Actual paragraphs: {}", paragraph_count);

    for (i, expected_text) in expected_texts.iter().enumerate() {
        println!("\nExpected paragraph {}: \"{}\"", i + 1, expected_text);
        if let Some(actual_text) = paragraph_texts.get(i) {
            println!("Actual paragraph {}: \"{}\"", i + 1, actual_text);

            // Check if content matches (allowing for whitespace differences)
            let expected_normalized = expected_text.replace('\n', " ").trim().to_string();
            let actual_normalized = actual_text.replace('\n', " ").trim().to_string();

            if expected_normalized == actual_normalized {
                println!("✅ Content matches!");
            } else {
                println!("❌ Content differs!");
                println!("   Expected: \"{}\"", expected_normalized);
                println!("   Actual:   \"{}\"", actual_normalized);
            }
        } else {
            println!("❌ Missing paragraph {}!", i + 1);
        }
    }

    // This assertion will fail, demonstrating the bug
    assert_eq!(
        paragraph_count, expected_paragraph_count,
        "Expected {} paragraphs but found {}. Second paragraph is missing!",
        expected_paragraph_count, paragraph_count
    );
}

/// Helper function to extract element type name for debugging
fn element_type_name(element: &SessionContainerElement) -> &'static str {
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

/// Helper function to extract text content from a paragraph
fn extract_paragraph_text(
    paragraph: &txxt::ast::elements::paragraph::block::ParagraphBlock,
) -> String {
    paragraph
        .content
        .iter()
        .map(|transform| transform.text_content())
        .collect::<Vec<_>>()
        .join("")
}

/// Test that shows the current behavior (only finds first paragraph)
#[test]
#[ignore] // Disabled: Parser being reimplemented with regex-based grammar engine
fn test_current_behavior_one_paragraph() {
    let corpus = TxxtCorpora::load_document("01-two-paragraphs").expect("Failed to load document");

    let document = run_all(&corpus.source_text, Some("test".to_string())).expect("Failed to parse");

    // This currently passes but shouldn't - it demonstrates the bug
    let paragraph_count = document
        .content
        .content
        .iter()
        .filter(|e| matches!(e, SessionContainerElement::Paragraph(_)))
        .count();

    // This assertion currently passes (paragraph_count == 1) but should fail
    // because we should have 2 paragraphs, not 1
    println!(
        "Current implementation finds {} paragraph(s)",
        paragraph_count
    );
    assert!(paragraph_count >= 1, "Should find at least one paragraph");

    // TODO: Change this assertion once the parser is fixed to handle multiple paragraphs
    // assert_eq!(paragraph_count, 2, "Should find exactly 2 paragraphs");
}
