/// # Illustrated Parser Test Example
///
/// This file demonstrates best practices for writing parser tests using the
/// corpora system, AST assertions, and tree visualization tools.
///
/// ## Purpose
///
/// - Show complete test patterns from simple to complex
/// - Demonstrate error handling and validation
/// - Illustrate AST inspection techniques
/// - Provide copy-paste templates for new tests
///
/// ## Structure
///
/// Tests are organized by complexity and purpose:
/// 1. Basic element parsing (single elements)
/// 2. Container parsing (nested structures)
/// 3. Full document parsing (integration)
/// 4. Error case handling (invalid input)
/// 5. AST validation helpers (reusable utilities)
#[path = "corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;

// ============================================================================
// PART 1: Basic Element Parsing
// ============================================================================

/// # Pattern: Parse Single Element (Valid)
///
/// The simplest test: load a corpus, parse it, verify success.
///
/// ## Template
///
/// ```rust
/// let corpus = TxxtCorpora::load("txxt.core.spec.ELEMENT.valid.CASE").unwrap();
/// let result = parse_ELEMENT(&corpus.source_text);
/// assert!(result.is_ok());
/// ```
///
/// ## What to Check
///
/// - Parse succeeds (no errors)
/// - Element type is correct
/// - Basic structure is valid
#[test]
fn illustrated_parse_simple_paragraph() {
    // Step 1: Load the test corpus
    // This gets the exact sample from docs/specs/elements/paragraph/paragraph.txxt
    let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")
        .expect("Corpus should exist in specs");

    // Step 2: Verify we got the right content
    // Good practice: check corpus before parsing
    assert!(!corpus.source_text.is_empty(), "Corpus should have content");
    assert!(
        corpus.source_text.contains("paragraph"),
        "Should be about paragraphs"
    );

    // Step 3: Parse the content
    // TODO: Replace with actual parser when implemented
    // let result = parse_paragraph(&corpus.source_text);
    // assert!(result.is_ok(), "Simple paragraph should parse successfully");

    // Step 4: Validate the AST (when parser is ready)
    // let paragraph = result.unwrap();
    // assert_eq!(paragraph.element_type(), ElementType::Block);
    // assert!(!paragraph.content.is_empty());
}

/// # Pattern: Parse Element with Structure Validation
///
/// Beyond just checking parse success, validate the actual AST structure.
///
/// ## What to Check
///
/// - Element has expected children count
/// - Content matches expected values
/// - Tokens are preserved correctly
/// - Annotations and parameters are present
#[test]
fn illustrated_parse_paragraph_with_validation() {
    let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.multiline")
        .expect("Multiline paragraph corpus should exist");

    // Verify test data expectations
    let line_count = corpus.source_text.lines().count();
    assert!(
        line_count >= 2,
        "Multiline paragraph should have multiple lines"
    );

    // TODO: Parse and validate structure
    // let paragraph = parse_paragraph(&corpus.source_text).unwrap();
    //
    // // Validate element type
    // assert_eq!(paragraph.element_type(), ElementType::Block);
    //
    // // Validate content (should merge into single paragraph)
    // let text = extract_text_content(&paragraph);
    // assert!(text.contains("paragraph begins"));
    // assert!(text.contains("continues on"));
    //
    // // Validate tokens are preserved
    // assert!(!paragraph.tokens().tokens.is_empty());
    //
    // // Should have no annotations (plain paragraph)
    // assert!(paragraph.annotations().is_empty());
}

// ============================================================================
// PART 2: Container and Nesting
// ============================================================================

/// # Pattern: Parse Nested Structure
///
/// Test parsing of elements with nested content containers.
///
/// ## What to Check
///
/// - Parent element parsed correctly
/// - Container created with proper type
/// - Child elements recognized
/// - Indentation preserved
#[test]
fn illustrated_parse_definition_with_nested_content() {
    let _corpus = TxxtCorpora::load("txxt.core.spec.definition.valid.nested-definitions")
        .expect("Nested definition corpus should exist");

    // This corpus has a definition containing other definitions
    // Structure:
    //   Programming ::
    //       (paragraph about programming)
    //       Algorithm ::
    //           (algorithm definition)
    //       Data Structure ::
    //           (data structure definition)

    // TODO: Parse and validate nesting
    // let definition = parse_definition(&corpus.source_text).unwrap();
    //
    // // Validate parent definition
    // assert_eq!(definition.term_text(), "Programming");
    //
    // // Validate content container
    // assert!(!definition.content.is_empty());
    //
    // // Should have 2 nested definitions + 1 paragraph
    // let nested_defs = count_elements_by_type(&definition.content, ElementType::Block);
    // assert_eq!(nested_defs, 3);
}

/// # Pattern: Parse List with Nested Lists
///
/// Test multi-level nesting with different decoration styles.
///
/// ## What to Check
///
/// - Outer list parsed with correct style
/// - Nested lists detected and parsed
/// - Indentation levels correct
/// - Mixed styles handled properly
#[test]
fn illustrated_parse_nested_list() {
    let _corpus = TxxtCorpora::load("txxt.core.spec.list.valid.nested-mixed-styles")
        .expect("Nested list corpus should exist");

    // This has numerical list with nested plain lists with nested alphabetical
    // 1. Item
    //     - Nested item
    //         a. Deep nested
    //         b. Another deep nested

    // TODO: Parse and validate
    // let list = parse_list(&corpus.source_text).unwrap();
    //
    // // Outer list should be numerical
    // assert_eq!(list.decoration_type.style, NumberingStyle::Numerical);
    //
    // // Should have list items
    // assert!(!list.items.is_empty());
    //
    // // First item should have nested content
    // let first_item = &list.items[0];
    // assert!(first_item.nested.is_some());
    //
    // // Nested content should have a list
    // let nested_container = first_item.nested.as_ref().unwrap();
    // // ... validate nested structure
}

// ============================================================================
// PART 3: Full Document Parsing
// ============================================================================

/// # Pattern: Parse Complete Document (Progressive)
///
/// Test full document parsing using ensemble documents.
/// Start simple, progress to complex.
///
/// ## What to Check
///
/// - Document parses completely
/// - All elements recognized
/// - Hierarchy preserved
/// - Annotations attached correctly
#[test]
fn illustrated_parse_simple_document() {
    // Start with simplest ensemble
    let _corpus =
        TxxtCorpora::load_document("01-two-paragraphs").expect("Simplest ensemble should exist");

    // Document structure:
    //   Paragraph 1
    //   <blank line>
    //   Paragraph 2

    // TODO: Parse full document
    // let document = parse_document(&corpus.source_text).unwrap();
    //
    // // Validate document structure
    // assert_eq!(document.content.content.len(), 2, "Should have 2 paragraphs");
    //
    // // Both should be paragraphs
    // for element in &document.content.content {
    //     match element {
    //         SessionContainerElement::Paragraph(_) => {},
    //         _ => panic!("Expected paragraph"),
    //     }
    // }
}

/// # Pattern: Parse Complex Document with Validation
///
/// Test comprehensive document with multiple element types and nesting.
///
/// ## What to Check
///
/// - All element types present
/// - Deep nesting works
/// - Annotations attached
/// - Verbatim blocks preserved
#[test]
fn illustrated_parse_full_document() {
    let _corpus =
        TxxtCorpora::load_document("11-full-document").expect("Full document should exist");

    // This document has:
    // - Document-level annotations
    // - Multiple sessions with nesting
    // - Lists (flat and nested)
    // - Definitions
    // - Verbatim blocks
    // - Inline formatting throughout

    // TODO: Parse and validate comprehensively
    // let document = parse_document(&corpus.source_text).unwrap();
    //
    // // Check document-level annotations
    // let title_annotation = document.meta.get("title");
    // assert!(title_annotation.is_some());
    //
    // // Check element variety using traversal
    // let traversable = TraversableDocument::from_document(&document);
    // let stats = traversable.element_type_stats();
    //
    // assert!(stats.get(&ElementType::Block).unwrap() > &5);
    // assert!(stats.get(&ElementType::Container).unwrap() > &3);
    //
    // // Verify specific elements exist
    // let verbatim_blocks = traversable.query()
    //     .xpath("//Verbatim")
    //     .unwrap();
    // assert!(!verbatim_blocks.is_empty());
}

// ============================================================================
// PART 4: Error Handling
// ============================================================================

/// # Pattern: Test Invalid Input
///
/// Verify parser rejects invalid syntax appropriately.
///
/// ## What to Check
///
/// - Parse returns error (not panic)
/// - Error type matches expected
/// - Error message is helpful
/// - Recovery is graceful
#[test]
fn illustrated_parse_invalid_definition() {
    // Load error case corpus
    let corpus = TxxtCorpora::load("txxt.core.spec.definition.invalid.empty-term")
        .expect("Invalid corpus should exist");

    // Verify it's marked as error case
    assert!(corpus.is_error_case(), "Should be error corpus");
    assert_eq!(
        corpus.expected_error(),
        Some("EmptyTerm"),
        "Should expect EmptyTerm error"
    );

    // TODO: Parse and expect error
    // let result = parse_definition(&corpus.source_text);
    //
    // assert!(result.is_err(), "Empty term should fail parsing");
    //
    // // Validate error details
    // if let Err(error) = result {
    //     let error_msg = error.to_string();
    //     assert!(
    //         error_msg.contains("term") || error_msg.contains("empty"),
    //         "Error should mention the problem: {}",
    //         error_msg
    //     );
    // }
}

/// # Pattern: Test Graceful Degradation
///
/// Verify parser falls back gracefully on ambiguous input.
///
/// ## What to Check
///
/// - Parser doesn't panic
/// - Falls back to paragraph when appropriate
/// - Content preserved even if structure misunderstood
#[test]
fn illustrated_parse_ambiguous_session_vs_paragraph() {
    // Single line with no indented content following
    // Could be session title OR could be paragraph
    // Spec says: treat as paragraph (no indented content = not session)

    let _test_input = "This could be a title\n\nBut has no indented content.";

    // TODO: Parse and verify fallback
    // let document = parse_document(test_input).unwrap();
    //
    // // Should parse as paragraphs, not a session
    // assert_eq!(document.content.content.len(), 2);
    // for element in &document.content.content {
    //     assert!(matches!(element, SessionContainerElement::Paragraph(_)));
    // }
}

// ============================================================================
// PART 5: AST Validation Helpers
// ============================================================================

/// # Helper: Extract Text Content from Elements
///
/// Recursively extract all text from an element for validation.
///
/// ## Usage
///
/// ```rust
/// let text = extract_all_text(&paragraph);
/// assert!(text.contains("expected content"));
/// ```
#[allow(dead_code)] // Will be used by parser tests
fn extract_all_text(_element: &str) -> String {
    // TODO: Implement when AST types are available
    // This would recursively walk through:
    // - TextTransform content
    // - Nested containers
    // - All child elements
    // And concatenate all text spans
    String::from("TODO: Implement with real AST")
}

/// # Helper: Count Elements by Type
///
/// Count how many elements of a specific type exist in a container.
///
/// ## Usage
///
/// ```rust
/// let para_count = count_elements_by_type(&container, ElementType::Block);
/// assert_eq!(para_count, 3);
/// ```
#[allow(dead_code)] // Will be used by parser tests
fn count_elements_by_type(_container: &str, _element_type: &str) -> usize {
    // TODO: Implement when AST types are available
    // This would:
    // 1. Iterate through container.content
    // 2. Check each element's type
    // 3. Count matches
    // 4. Optionally recurse into nested containers
    0
}

/// # Helper: Validate Indentation Structure
///
/// Check that indentation follows txxt rules (multiples of 4).
///
/// ## Usage
///
/// ```rust
/// validate_indentation(&corpus.source_text);
/// ```
#[allow(dead_code)] // Will be used by parser tests
fn validate_indentation(source: &str) {
    for (i, line) in source.lines().enumerate() {
        if !line.trim().is_empty() {
            let spaces = line.chars().take_while(|&c| c == ' ').count();
            assert!(
                spaces % 4 == 0,
                "Line {} has invalid indentation: {} spaces (not multiple of 4)",
                i + 1,
                spaces
            );
        }
    }
}

/// # Helper: Assert Element Count
///
/// Verify container has expected number of elements.
///
/// ## Usage
///
/// ```rust
/// assert_element_count(&session.content, 3, "session should have 3 children");
/// ```
#[allow(dead_code)] // Will be used by parser tests
fn assert_element_count(_container: &str, expected: usize, message: &str) {
    // TODO: Implement when AST types are available
    // let actual = container.content.len();
    // assert_eq!(actual, expected, "{}", message);
    let _ = (expected, message); // Suppress unused warnings
}

/// # Helper: Assert Has Element Type
///
/// Verify container has at least one element of a specific type.
///
/// ## Usage
///
/// ```rust
/// assert_has_element_type(&document.content, ElementType::Block);
/// ```
#[allow(dead_code)] // Will be used by parser tests
fn assert_has_element_type(_container: &str, _element_type: &str) {
    // TODO: Implement when AST types are available
    // let found = container.content.iter().any(|e| e.element_type() == element_type);
    // assert!(found, "Container should have {:?} element", element_type);
}

// ============================================================================
// PART 6: Progressive Testing Pattern
// ============================================================================

/// # Pattern: Progressive Document Validation
///
/// Test documents in order from simple to complex to isolate capabilities.
///
/// ## Strategy
///
/// - If doc N fails, parser doesn't support that complexity level yet
/// - If doc N passes but N+1 fails, implement missing features from doc N+1
/// - Each document builds on previous capabilities
///
/// ## Benefits
///
/// - Clear failure isolation
/// - Incremental implementation
/// - Validates progressive complexity
#[test]
fn illustrated_progressive_parsing() {
    // Level 1: Just paragraphs (simplest possible)
    let _doc_01 = TxxtCorpora::load_document("01-two-paragraphs").expect("Doc 01 should exist");

    // TODO: When parser ready
    // let result = parse_document(&_doc_01.source_text);
    // assert!(result.is_ok(), "Level 1: Basic paragraphs should work");

    // Level 2: Add sessions
    let _doc_02 =
        TxxtCorpora::load_document("02-session-one-paragraph").expect("Doc 02 should exist");

    // TODO: When parser ready
    // let result = parse_document(&_doc_02.source_text);
    // assert!(result.is_ok(), "Level 2: Sessions should work");

    // Level 3: Add session nesting
    let _doc_05 =
        TxxtCorpora::load_document("05-nested-sessions-basic").expect("Doc 05 should exist");

    // TODO: When parser ready
    // let result = parse_document(&doc_05.source_text);
    // assert!(result.is_ok(), "Level 3: Nested sessions should work");

    // Continue through all 11 documents...
    // Each level adds complexity

    // Verify all documents exist for progressive testing
    let all_docs = TxxtCorpora::load_all_documents().expect("Should load all ensembles");
    assert!(all_docs.len() >= 11, "Should have all ensemble documents");
}

// ============================================================================
// PART 7: Using Tree Visualization for Debugging
// ============================================================================

/// # Pattern: Visualize AST for Debugging
///
/// When tests fail, use tree visualization to see what was parsed.
///
/// ## Benefits
///
/// - See actual structure vs expected
/// - Identify parsing errors visually
/// - Understand nesting issues
/// - Debug container boundaries
#[test]
#[ignore] // Remove ignore when parser is implemented
fn illustrated_debug_with_tree_visualization() {
    let _corpus =
        TxxtCorpora::load_document("09-nested-complex").expect("Complex doc should exist");

    // TODO: When parser ready, uncomment:
    /*
    let document = parse_document(&corpus.source_text).unwrap();

    // Use tree visualizer for debugging
    use txxt::ast::debug::AstTreeVisualizer;

    let viz = AstTreeVisualizer::new();
    let tree_view = viz.visualize(&document);

    // Print during development (remove for final tests)
    println!("\n=== AST Tree View ===\n{}", tree_view);

    // Or save to file for review
    std::fs::write("debug_ast_tree.txt", tree_view).unwrap();

    // Can also use compact view
    let compact_viz = AstTreeVisualizer::new_compact();
    println!("\n=== Compact View ===\n{}", compact_viz.visualize(&document));
    */
}

/// # Pattern: Use Traversal for Validation
///
/// Leverage tree traversal API to validate complex structures.
///
/// ## Benefits
///
/// - Query elements by type
/// - Navigate parent/child relationships
/// - Collect statistics
/// - Use XPath selectors
#[test]
#[ignore] // Remove ignore when parser is implemented
fn illustrated_validate_with_traversal() {
    let _corpus = TxxtCorpora::load_document("11-full-document").expect("Full doc should exist");

    // TODO: When parser ready, uncomment:
    /*
    let document = parse_document(&corpus.source_text).unwrap();
    let traversable = TraversableDocument::from_document(&document);

    // Get statistics
    let stats = traversable.element_type_stats();
    println!("Element counts: {:?}", stats);

    // Find all paragraphs
    let paragraphs = traversable.query()
        .find_by_type(ElementType::Block)
        .collect::<Vec<_>>();

    assert!(paragraphs.len() > 5, "Full doc should have many paragraphs");

    // Find verbatim blocks using XPath
    let verbatim = traversable.xpath("//Verbatim").unwrap();
    assert!(!verbatim.is_empty(), "Full doc has verbatim blocks");

    // Find sessions with specific text
    let intro_sessions = traversable.query()
        .find_by_type(ElementType::Block)
        .text_contains("Introduction")
        .collect::<Vec<_>>();

    assert!(!intro_sessions.is_empty());

    // Check tree depth
    let depth = traversable.tree_depth();
    assert!(depth >= 3, "Full doc should have deep nesting");
    */
}

// ============================================================================
// PART 8: Snapshot Testing
// ============================================================================

/// # Pattern: Snapshot Testing for Regression Prevention
///
/// Use insta crate to save expected AST structure.
///
/// ## Benefits
///
/// - Catch unintended AST changes
/// - Visual review of structure
/// - Easy to update when changes are intentional
/// - No manual assertion writing
#[test]
#[ignore] // Remove ignore when parser is implemented
fn illustrated_snapshot_test() {
    let _corpus = TxxtCorpora::load("txxt.core.spec.session.valid.hierarchical")
        .expect("Hierarchical session corpus");

    // TODO: When parser ready, uncomment:
    /*
    let session = parse_session(&corpus.source_text).unwrap();

    // First time: cargo test -- --ignored
    // This creates snapshot file
    insta::assert_yaml_snapshot!(session);

    // Subsequent runs: compares against saved snapshot
    // If AST changes:
    // - cargo insta review (to review changes)
    // - cargo insta accept (to accept changes)
    // - cargo insta reject (to reject changes)
    */
}

// ============================================================================
// PART 9: Error Messages and Recovery
// ============================================================================

/// # Pattern: Validate Error Messages are Helpful
///
/// Ensure error messages guide users to fix issues.
///
/// ## What to Check
///
/// - Error indicates what's wrong
/// - Error shows where (line/column)
/// - Error suggests how to fix
/// - Error includes context
#[test]
#[ignore] // Remove ignore when parser is implemented
fn illustrated_helpful_error_messages() {
    let _corpus = TxxtCorpora::load("txxt.core.spec.verbatim.invalid.missing-label")
        .expect("Invalid verbatim corpus");

    // TODO: When parser ready, uncomment:
    /*
    let result = parse_verbatim(&corpus.source_text);
    assert!(result.is_err());

    if let Err(error) = result {
        let msg = error.to_string();

        // Should mention what's wrong
        assert!(
            msg.contains("label") || msg.contains("missing"),
            "Error should indicate missing label"
        );

        // Should show position (if available)
        // assert!(msg.contains("line"), "Should show line number");

        // Should be specific
        assert!(
            msg.len() > 10,
            "Error message should be descriptive, not generic"
        );
    }
    */
}

// ============================================================================
// PART 10: Testing Checklist Template
// ============================================================================

// # Testing Checklist for Each Element
//
// Copy this checklist when implementing a new element parser.
//
// ## Valid Input Tests
//
// - [ ] Simple form parses successfully
// - [ ] Complex form with formatting
// - [ ] With parameters (if applicable)
// - [ ] With annotations (if applicable)
// - [ ] Nested content (if applicable)
// - [ ] Edge cases from spec
//
// ## Invalid Input Tests
//
// - [ ] Missing required parts
// - [ ] Malformed syntax
// - [ ] Invalid indentation
// - [ ] Empty/whitespace only
//
// ## Integration Tests
//
// - [ ] In content container
// - [ ] In session container
// - [ ] With other elements at same level
// - [ ] In ensemble documents
//
// ## AST Validation
//
// - [ ] Correct element type
// - [ ] Content structure matches spec
// - [ ] Tokens preserved
// - [ ] Annotations attached properly
// - [ ] Parameters parsed correctly
//
// ## Documentation
//
// - [ ] Test names are descriptive
// - [ ] Complex tests have inline comments
// - [ ] Helper functions documented
// - [ ] Edge cases explained

// ============================================================================
// Quick Test Template - Copy & Modify
// ============================================================================

/*
#[test]
fn test_ELEMENT_CASE() {
    // Load corpus
    let corpus = TxxtCorpora::load("txxt.core.spec.ELEMENT.valid.CASE")
        .expect("Corpus should exist");

    // Parse
    let result = parse_ELEMENT(&corpus.source_text);
    assert!(result.is_ok(), "Should parse successfully");

    // Validate
    let element = result.unwrap();
    assert_eq!(element.element_type(), ElementType::Block);
    // ... more assertions
}
*/

// NOTE: This illustrated test file serves as a comprehensive template and guide
// for writing parser tests. Copy patterns and adapt them to your specific testing needs.
