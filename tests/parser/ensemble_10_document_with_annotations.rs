//! Ensemble 10: Document with Annotations
//!
//! Tests document-level annotations and annotations within sessions.
//! Document structure:
//! - Document-level annotations (4): title, author, version, date
//! - Session: "1. Introduction"
//!   - Paragraph
//!   - Annotation: "note"
//!   - Paragraph
//! - Session: "2. Content Section"
//!   - Annotation: "status"
//!   - Paragraph
//!   - List

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::transform::run_all;

#[test]
fn test_ensemble_10_document_with_annotations() {
    // Load ensemble document 10
    let corpus = TxxtCorpora::load_document("10-document-with-annotations")
        .expect("Failed to load ensemble 10");

    let source = &corpus.source_text;

    println!("=== Ensemble 10: Document with Annotations ===");
    println!("Source document:\n{}", source);
    println!();

    // Parse through full pipeline
    let document =
        run_all(source, Some("ensemble-10.txxt".to_string())).expect("Failed to parse ensemble 10");

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    // Document should have 2 sessions at top level (Introduction, Content Section)
    // The 4 annotations at the start should be document-level (not in content)
    // For now, we'll count all top-level elements including annotations

    // Let's see what we actually got
    for (i, element) in document.content.content.iter().enumerate() {
        let type_name = match element {
            SessionContainerElement::Paragraph(_) => "Paragraph",
            SessionContainerElement::Session(_) => "Session",
            SessionContainerElement::List(_) => "List",
            SessionContainerElement::Definition(_) => "Definition",
            SessionContainerElement::Annotation(_) => "Annotation",
            SessionContainerElement::BlankLine(_) => "BlankLine",
            _ => "Other",
        };
        println!("[{}] {}", i, type_name);
    }

    // Count annotations and sessions separately
    let mut annotation_count = 0;
    let mut session_count = 0;

    for element in &document.content.content {
        match element {
            SessionContainerElement::Annotation(_) => annotation_count += 1,
            SessionContainerElement::Session(_) => session_count += 1,
            _ => {}
        }
    }

    println!("\nDocument-level annotations: {}", annotation_count);
    println!("Top-level sessions: {}", session_count);

    // Should have 4 document-level annotations
    assert_eq!(
        annotation_count, 4,
        "Expected 4 document-level annotations (title, author, version, date)"
    );

    // Should have 2 sessions
    assert_eq!(session_count, 2, "Expected 2 top-level sessions");

    // Find and validate the annotations
    let mut found_title = false;
    let mut found_author = false;
    let mut found_version = false;
    let mut found_date = false;

    for element in &document.content.content {
        if let SessionContainerElement::Annotation(ann) = element {
            println!(
                "\nAnnotation: label='{}', content={:?}",
                ann.label, ann.content
            );
            match ann.label.as_str() {
                "title" => found_title = true,
                "author" => found_author = true,
                "version" => found_version = true,
                "date" => found_date = true,
                _ => {}
            }
        }
    }

    assert!(found_title, "Should have 'title' annotation");
    assert!(found_author, "Should have 'author' annotation");
    assert!(found_version, "Should have 'version' annotation");
    assert!(found_date, "Should have 'date' annotation");

    // Validate sessions contain annotations
    let mut session_1_index = None;
    let mut session_2_index = None;

    for (i, element) in document.content.content.iter().enumerate() {
        if let SessionContainerElement::Session(session) = element {
            let title_text: String = session
                .title
                .content
                .iter()
                .map(|t| t.text_content())
                .collect();

            if title_text.contains("Introduction") {
                session_1_index = Some(i);
            } else if title_text.contains("Content Section") {
                session_2_index = Some(i);
            }
        }
    }

    // Validate Session 1: Introduction
    if let Some(idx) = session_1_index {
        if let SessionContainerElement::Session(session) = &document.content.content[idx] {
            println!("\n=== Session 1: Introduction ===");
            println!("Content elements: {}", session.content.content.len());

            // Should have: paragraph + annotation + paragraph
            let mut has_note_annotation = false;

            for element in &session.content.content {
                if let SessionContainerElement::Annotation(ann) = element {
                    if ann.label == "note" {
                        has_note_annotation = true;
                        println!("Found 'note' annotation in session 1");
                    }
                }
            }

            assert!(
                has_note_annotation,
                "Session 1 should contain 'note' annotation"
            );
        }
    }

    // Validate Session 2: Content Section
    if let Some(idx) = session_2_index {
        if let SessionContainerElement::Session(session) = &document.content.content[idx] {
            println!("\n=== Session 2: Content Section ===");
            println!("Content elements: {}", session.content.content.len());

            // Should have: annotation + paragraph + list
            let mut has_status_annotation = false;

            for element in &session.content.content {
                if let SessionContainerElement::Annotation(ann) = element {
                    if ann.label == "status" {
                        has_status_annotation = true;
                        println!("Found 'status' annotation in session 2");
                    }
                }
            }

            assert!(
                has_status_annotation,
                "Session 2 should contain 'status' annotation"
            );
        }
    }

    println!("\n✅ Ensemble 10 test passed!");
}
