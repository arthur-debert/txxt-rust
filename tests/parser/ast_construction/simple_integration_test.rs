//! Simple AST construction integration test
//!
//! This test verifies that AST construction is properly integrated into the
//! parsing pipeline with a minimal example.
//!
//! # Testing Pattern: Using ego-tree Traversal
//!
//! This test demonstrates the recommended pattern for testing document structure
//! using the ego-tree based traversal API rather than manual tree walking:
//!
//! 1. Parse source text to AST elements
//! 2. Assemble elements into a Document
//! 3. Wrap Document in TraversableDocument for querying
//! 4. Use XPath-style queries or visitor pattern for validation
//!
//! Benefits:
//! - O(1) parent/sibling access
//! - Type-safe element processing
//! - Cleaner test code
//! - Less brittleness (no manual indexing)

use txxt::assembler::DocumentAssembler;
use txxt::ast::traversal::TraversableDocument;
use txxt::lexer::tokenize;
use txxt::lexer::ScannerTokenTreeBuilder;
use txxt::process::process_parser;

/// Test that AST construction works with a simple paragraph
#[test]
fn test_simple_ast_construction_integration() {
    // Very simple test document - just a paragraph
    let source = "This is a simple paragraph.";

    // Phase 1: Lexer pipeline
    let tokens = tokenize(source);
    let token_tree = ScannerTokenTreeBuilder::new()
        .build_tree(tokens)
        .expect("Failed to build token tree");

    // Phase 2: Parser pipeline (including AST construction)
    let ast_elements = process_parser(token_tree).expect("Failed to parse to AST elements");

    // Verify that we got AST elements
    assert!(
        !ast_elements.is_empty(),
        "Should have parsed some AST elements"
    );

    // Phase 3: Assemble into Document
    let assembler = DocumentAssembler::new();
    let document = assembler
        .assemble_document(ast_elements, Some("test.txxt".to_string()))
        .expect("Failed to assemble document");

    // Wrap in TraversableDocument for ego-tree queries
    let traversable = TraversableDocument::from_document(&document);

    // Example 1: Query for block elements using XPath-style syntax
    let blocks = traversable
        .xpath("//Block")
        .expect("XPath query should succeed");
    assert!(
        !blocks.is_empty(),
        "Should have at least one block element (paragraph)"
    );

    // Example 2: Find container element
    let containers = traversable
        .xpath("//Container")
        .expect("XPath query should succeed");
    assert!(
        !containers.is_empty(),
        "Should have at least one container (SessionContainer)"
    );

    // Example 3: Navigate via root and children (demonstrates parent/child relationships)
    let root = traversable.root();
    let children: Vec<_> = root.children().collect();
    assert!(!children.is_empty(), "Root should have child elements");

    println!(
        "Successfully parsed and queried document structure: {} blocks, {} containers",
        blocks.len(),
        containers.len()
    );
}

/// Test ego-tree traversal with querying capabilities
#[test]
fn test_traversal_xpath_queries() {
    // Simple document for demonstrating XPath queries
    let source = "This is a test document.";

    // Full pipeline
    let tokens = tokenize(source);
    let token_tree = ScannerTokenTreeBuilder::new()
        .build_tree(tokens)
        .expect("Failed to build token tree");
    let ast_elements = process_parser(token_tree).expect("Failed to parse");

    let assembler = DocumentAssembler::new();
    let document = assembler
        .assemble_document(ast_elements, Some("test.txxt".to_string()))
        .expect("Failed to assemble");

    // Use traversal to query structure
    let traversable = TraversableDocument::from_document(&document);

    // Demonstrate different XPath query patterns

    // Pattern 1: Find all blocks
    let blocks = traversable
        .xpath("//Block")
        .expect("XPath query should succeed");
    assert!(!blocks.is_empty(), "Should find at least one block element");

    // Pattern 2: Find all containers
    let containers = traversable
        .xpath("//Container")
        .expect("XPath query should succeed");
    assert!(!containers.is_empty(), "Should find at least one container");

    // Pattern 3: Find all elements with wildcard
    let all_elements = traversable
        .xpath("//*")
        .expect("XPath query should succeed");
    assert!(
        all_elements.len() >= blocks.len(),
        "Should find more total elements than just blocks"
    );

    println!(
        "XPath queries: {} blocks, {} containers, {} total elements",
        blocks.len(),
        containers.len(),
        all_elements.len()
    );
}
