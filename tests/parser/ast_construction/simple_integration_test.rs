//! Simple AST construction integration test
//!
//! This test verifies that AST construction is properly integrated into the
//! parsing pipeline with a minimal example.

use txxt::lexer::pipeline::ScannerTokenTreeBuilder;
use txxt::lexer::tokenize;
use txxt::pipeline::parser_pipeline;

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
    let ast_elements = parser_pipeline(token_tree).expect("Failed to parse to AST elements");

    // Verify that we got AST elements
    assert!(
        !ast_elements.is_empty(),
        "Should have parsed some AST elements"
    );

    println!("Successfully parsed {} AST elements", ast_elements.len());
}
