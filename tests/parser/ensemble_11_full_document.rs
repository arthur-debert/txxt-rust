//! Ensemble 11: Full Document
//!
//! Comprehensive test showcasing all major txxt features working together.
//! Document structure:
//! - Document-level annotations (4): title, author, date, version
//! - Session: "1. Introduction" - 2 paragraphs
//! - Session: "2. Basic Elements"
//!   - Session: "2.1. Paragraphs and Text Flow" - 2 paragraphs
//!   - Session: "2.2. Lists and Enumeration" - paragraph + 2 lists
//! - Session: "3. Advanced Features"
//!   - Session: "3.1. Definitions" - paragraph + 3 definitions
//!   - Session: "3.2. Code Examples" - paragraph + verbatim + paragraph
//!   - Session: "3.3. Nested Structure" - paragraph + nested list
//! - Session: "4. Inline Elements"
//!   - Session: "4.1. Formatting Options" - paragraph
//!   - Session: "4.2. Mathematical Expressions" - paragraph
//! - Session: "5. Best Practices" - paragraph + list + annotation
//! - Session: "6. Conclusion" - 2 paragraphs
//! - Document-level annotation: "note"

#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::ast::elements::session::session_container::SessionContainerElement;
use txxt::transform::run_all;

#[test]
fn test_ensemble_11_full_document() {
    // Load ensemble document 11
    let corpus =
        TxxtCorpora::load_document("11-full-document").expect("Failed to load ensemble 11");

    let source = &corpus.source_text;

    println!("=== Ensemble 11: Full Document ===");
    println!("Source length: {} bytes", source.len());
    println!();

    // Parse through full pipeline
    let document =
        run_all(source, Some("ensemble-11.txxt".to_string())).expect("Failed to parse ensemble 11");

    println!("=== Document Structure ===");
    println!("Top-level elements: {}", document.content.content.len());

    // Count element types
    let mut annotation_count = 0;
    let mut session_count = 0;

    for element in &document.content.content {
        match element {
            SessionContainerElement::Annotation(_) => annotation_count += 1,
            SessionContainerElement::Session(_) => session_count += 1,
            _ => {}
        }
    }

    println!("Document-level annotations: {}", annotation_count);
    println!("Top-level sessions: {}", session_count);

    // Should have 5 document-level annotations (4 at start + 1 at end)
    assert_eq!(annotation_count, 5, "Expected 5 document-level annotations");

    // Should have 6 main sessions
    assert_eq!(session_count, 6, "Expected 6 top-level sessions");

    // Validate annotations exist
    let annotation_labels: Vec<String> = document
        .content
        .content
        .iter()
        .filter_map(|e| {
            if let SessionContainerElement::Annotation(ann) = e {
                Some(ann.label.clone())
            } else {
                None
            }
        })
        .collect();

    println!("\nAnnotation labels found: {:?}", annotation_labels);

    assert!(
        annotation_labels.contains(&"title".to_string()),
        "Should have 'title' annotation"
    );
    assert!(
        annotation_labels.contains(&"author".to_string()),
        "Should have 'author' annotation"
    );
    assert!(
        annotation_labels.contains(&"date".to_string()),
        "Should have 'date' annotation"
    );
    assert!(
        annotation_labels.contains(&"version".to_string()),
        "Should have 'version' annotation"
    );
    assert!(
        annotation_labels.contains(&"note".to_string()),
        "Should have 'note' annotation at end"
    );

    // Find and validate key sessions
    let sessions: Vec<&txxt::ast::elements::session::SessionBlock> = document
        .content
        .content
        .iter()
        .filter_map(|e| {
            if let SessionContainerElement::Session(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .collect();

    // Session 1: Introduction
    let intro_session = &sessions[0];
    let intro_title: String = intro_session
        .title
        .content
        .iter()
        .map(|t| t.text_content())
        .collect();
    println!("\n=== Session 1 ===");
    println!("Title: {}", intro_title);
    assert!(intro_title.contains("Introduction"));
    assert_eq!(
        intro_session.content.content.len(),
        2,
        "Introduction should have 2 paragraphs"
    );

    // Session 2: Basic Elements (should have nested sessions)
    let basic_elements_session = &sessions[1];
    let basic_title: String = basic_elements_session
        .title
        .content
        .iter()
        .map(|t| t.text_content())
        .collect();
    println!("\n=== Session 2 ===");
    println!("Title: {}", basic_title);
    assert!(basic_title.contains("Basic Elements"));

    // Count nested sessions
    let nested_session_count = basic_elements_session
        .content
        .content
        .iter()
        .filter(|e| matches!(e, SessionContainerElement::Session(_)))
        .count();

    println!("Nested sessions: {}", nested_session_count);
    assert_eq!(
        nested_session_count, 2,
        "Basic Elements should have 2 nested sessions"
    );

    // Session 3: Advanced Features (should have nested sessions with definitions)
    let advanced_session = &sessions[2];
    let advanced_title: String = advanced_session
        .title
        .content
        .iter()
        .map(|t| t.text_content())
        .collect();
    println!("\n=== Session 3 ===");
    println!("Title: {}", advanced_title);
    assert!(advanced_title.contains("Advanced Features"));

    // Check for definitions subsession
    let has_definitions_section = advanced_session.content.content.iter().any(|e| {
        if let SessionContainerElement::Session(s) = e {
            let title: String = s.title.content.iter().map(|t| t.text_content()).collect();
            title.contains("Definitions")
        } else {
            false
        }
    });

    assert!(
        has_definitions_section,
        "Advanced Features should have Definitions subsection"
    );

    // Session 5: Best Practices (should have annotation)
    let best_practices_session = &sessions[4];
    let bp_title: String = best_practices_session
        .title
        .content
        .iter()
        .map(|t| t.text_content())
        .collect();
    println!("\n=== Session 5 ===");
    println!("Title: {}", bp_title);
    assert!(bp_title.contains("Best Practices"));

    // Check for tip annotation
    let has_tip_annotation = best_practices_session.content.content.iter().any(|e| {
        if let SessionContainerElement::Annotation(ann) = e {
            ann.label == "tip"
        } else {
            false
        }
    });

    assert!(
        has_tip_annotation,
        "Best Practices should have 'tip' annotation"
    );

    // Session 6: Conclusion
    let conclusion_session = &sessions[5];
    let conclusion_title: String = conclusion_session
        .title
        .content
        .iter()
        .map(|t| t.text_content())
        .collect();
    println!("\n=== Session 6 ===");
    println!("Title: {}", conclusion_title);
    assert!(conclusion_title.contains("Conclusion"));

    println!("\n✅ Ensemble 11 test passed!");
    println!("✅ Successfully parsed comprehensive document with all major features!");
}
