//! Ensemble 07: Session with Definition
//!
//! Tests parsing of sessions containing definition elements.
//! Document structure:
//! - Session: "1. Terminology"
//!   - Paragraph
//!   - Definition: "Parser ::"
//!     - Paragraph
//!   - Definition: "Tokenizer ::"
//!     - Paragraph
//!   - Definition: "AST ::"
//!     - Paragraph

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::ast::elements::containers::simple::SimpleBlockElement;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::transform::run_all;

#[test]
fn test_ensemble_07_session_with_definition() {
    // Load ensemble document 07
    let corpus = TxxtCorpora::load_document("07-session-with-definition")
        .expect("Failed to load ensemble 07");

    let source = &corpus.source_text;

    println!("=== Ensemble 07: Session with Definition ===");
    println!("Source document:\n{}", source);
    println!();

    // Parse through full pipeline
    let document =
        run_all(source, Some("ensemble-07.txxt".to_string())).expect("Failed to parse ensemble 07");

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    // Should have 1 session at top level
    assert_eq!(
        document.content.content.len(),
        1,
        "Expected 1 top-level session"
    );

    // Verify the session
    if let SessionContainerElement::Session(session) = &document.content.content[0] {
        println!("\n=== Session: '1. Terminology' ===");

        // Check title
        let title_text: String = session
            .title
            .content
            .iter()
            .map(|t| t.text_content())
            .collect();
        println!("Title: {}", title_text);
        assert!(
            title_text.contains("Terminology"),
            "Session title should contain 'Terminology'"
        );

        // Check session content
        println!(
            "Session content elements: {}",
            session.content.content.len()
        );
        assert_eq!(
            session.content.content.len(),
            4,
            "Session should have 4 elements: paragraph + 3 definitions"
        );

        // Element 0: Paragraph (intro text)
        if let SessionContainerElement::Paragraph(para) = &session.content.content[0] {
            let para_text: String = para.content.iter().map(|t| t.text_content()).collect();
            println!("\n[0] Paragraph: {}", para_text);
            assert!(
                para_text.contains("defines important terms"),
                "First paragraph should introduce the definitions"
            );
        } else {
            panic!("Element 0 should be a Paragraph");
        }

        // Element 1: Definition "Parser ::"
        if let SessionContainerElement::Definition(def) = &session.content.content[1] {
            println!("\n[1] Definition:");
            let term_text: String = def.term.content.iter().map(|t| t.text_content()).collect();
            println!("  Term: {}", term_text);
            assert!(
                term_text.contains("Parser"),
                "First definition term should be 'Parser'"
            );

            // Check definition content
            assert_eq!(
                def.content.content.len(),
                1,
                "Parser definition should have 1 paragraph"
            );
            if let SimpleBlockElement::Paragraph(para) = &def.content.content[0] {
                let content_text: String = para.content.iter().map(|t| t.text_content()).collect();
                println!("  Content: {}", content_text);
                assert!(
                    content_text.contains("analyzes text according to formal grammar"),
                    "Parser definition content should describe parsing"
                );
            } else {
                panic!("Parser definition content should be a paragraph");
            }
        } else {
            panic!("Element 1 should be a Definition (Parser)");
        }

        // Element 2: Definition "Tokenizer ::"
        if let SessionContainerElement::Definition(def) = &session.content.content[2] {
            println!("\n[2] Definition:");
            let term_text: String = def.term.content.iter().map(|t| t.text_content()).collect();
            println!("  Term: {}", term_text);
            assert!(
                term_text.contains("Tokenizer"),
                "Second definition term should be 'Tokenizer'"
            );

            // Check definition content
            assert_eq!(
                def.content.content.len(),
                1,
                "Tokenizer definition should have 1 paragraph"
            );
            if let SimpleBlockElement::Paragraph(para) = &def.content.content[0] {
                let content_text: String = para.content.iter().map(|t| t.text_content()).collect();
                println!("  Content: {}", content_text);
                assert!(
                    content_text.contains("breaks input text into meaningful units"),
                    "Tokenizer definition content should describe tokenization"
                );
            } else {
                panic!("Tokenizer definition content should be a paragraph");
            }
        } else {
            panic!("Element 2 should be a Definition (Tokenizer)");
        }

        // Element 3: Definition "AST ::"
        if let SessionContainerElement::Definition(def) = &session.content.content[3] {
            println!("\n[3] Definition:");
            let term_text: String = def.term.content.iter().map(|t| t.text_content()).collect();
            println!("  Term: {}", term_text);
            assert!(
                term_text.contains("AST"),
                "Third definition term should be 'AST'"
            );

            // Check definition content
            assert_eq!(
                def.content.content.len(),
                1,
                "AST definition should have 1 paragraph"
            );
            if let SimpleBlockElement::Paragraph(para) = &def.content.content[0] {
                let content_text: String = para.content.iter().map(|t| t.text_content()).collect();
                println!("  Content: {}", content_text);
                assert!(
                    content_text.contains("Abstract Syntax Tree"),
                    "AST definition should expand the acronym"
                );
            } else {
                panic!("AST definition content should be a paragraph");
            }
        } else {
            panic!("Element 3 should be a Definition (AST)");
        }
    } else {
        panic!("Top-level element should be a Session");
    }

    println!("\n✅ Ensemble 07 test passed!");
}
