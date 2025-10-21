/// Test list parsing with simple list documents
///
/// This test verifies that list parsing works correctly using:
/// - list-01-simple-single.txxt: Single list with 3 plain items
/// - list-02-simple-nosession-multiple-list.txxt: Three separate lists
#[path = "../infrastructure/corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::api::Stage;
use txxt::cst::HighLevelToken;
use txxt::semantic::ast_construction::AstConstructor;

#[test]
fn test_list_01_simple_single() {
    // Load the corpus with high-level tokens
    let corpus =
        TxxtCorpora::load_document_with_processing("list-01-simple-single", Stage::HighLevelTokens)
            .expect("Failed to load list-01-simple-single");

    // Get the output and extract high-level tokens
    let output = corpus.output().expect("No output from processing");
    let high_level_tokens = match output {
        txxt::api::Output::HighLevelTokens(tokens) => tokens,
        _ => panic!("Expected HighLevelTokens output"),
    };

    // Verify we have 3 SequenceTextLine tokens (simple list)
    assert_eq!(
        high_level_tokens.tokens.len(),
        3,
        "Should have 3 SequenceTextLine tokens for simple list"
    );

    // Verify all tokens are SequenceTextLine
    for (i, token) in high_level_tokens.tokens.iter().enumerate() {
        assert!(
            matches!(token, HighLevelToken::SequenceTextLine { .. }),
            "Token {} should be SequenceTextLine, got {:?}",
            i,
            token
        );
    }

    // Parse to AST
    let mut constructor = AstConstructor::new();
    let ast_nodes = constructor
        .parse(high_level_tokens)
        .expect("Failed to parse list");

    // Should have exactly 1 list node
    assert_eq!(
        ast_nodes.len(),
        1,
        "Should have exactly 1 list node, got {}",
        ast_nodes.len()
    );

    // Verify it's a list
    match &ast_nodes[0] {
        txxt::semantic::ast_construction::AstNode::List(list_block) => {
            assert_eq!(list_block.items.len(), 3, "List should have 3 items");

            // Verify item markers (plain dash, no space)
            assert_eq!(list_block.items[0].marker, "-");
            assert_eq!(list_block.items[1].marker, "-");
            assert_eq!(list_block.items[2].marker, "-");
        }
        _ => panic!("Expected List node, got {:?}", ast_nodes[0]),
    }
}

#[test]
fn test_list_02_multiple_lists() {
    // Load the corpus with high-level tokens
    let corpus = TxxtCorpora::load_document_with_processing(
        "list-02-simple-nosession-multiple-list",
        Stage::HighLevelTokens,
    )
    .expect("Failed to load list-02-simple-nosession-multiple-list");

    // Get the output and extract high-level tokens
    let output = corpus.output().expect("No output from processing");
    let high_level_tokens = match output {
        txxt::api::Output::HighLevelTokens(tokens) => tokens,
        _ => panic!("Expected HighLevelTokens output"),
    };

    // Parse to AST
    let mut constructor = AstConstructor::new();
    let ast_nodes = constructor
        .parse(high_level_tokens)
        .expect("Failed to parse lists");

    // Should have exactly 3 list nodes (separated by blank lines)
    assert_eq!(
        ast_nodes.len(),
        3,
        "Should have exactly 3 list nodes, got {}",
        ast_nodes.len()
    );

    // Verify first list (plain markers)
    match &ast_nodes[0] {
        txxt::semantic::ast_construction::AstNode::List(list_block) => {
            assert_eq!(list_block.items.len(), 3, "First list should have 3 items");
            assert_eq!(list_block.items[0].marker, "-");
            assert_eq!(list_block.items[1].marker, "-");
            assert_eq!(list_block.items[2].marker, "-");
        }
        _ => panic!("Expected first node to be List"),
    }

    // Verify second list (numbered markers)
    match &ast_nodes[1] {
        txxt::semantic::ast_construction::AstNode::List(list_block) => {
            assert_eq!(list_block.items.len(), 3, "Second list should have 3 items");
            // Numbered list markers (1., 2., 3.)
            assert_eq!(list_block.items[0].marker, "1.");
            assert_eq!(list_block.items[1].marker, "2.");
            assert_eq!(list_block.items[2].marker, "3.");
        }
        _ => panic!("Expected second node to be List"),
    }

    // Verify third list (alphabetical markers)
    match &ast_nodes[2] {
        txxt::semantic::ast_construction::AstNode::List(list_block) => {
            assert_eq!(list_block.items.len(), 3, "Third list should have 3 items");
            // Alphabetical list markers (a., b., c.)
            assert_eq!(list_block.items[0].marker, "a.");
            assert_eq!(list_block.items[1].marker, "b.");
            assert_eq!(list_block.items[2].marker, "c.");
        }
        _ => panic!("Expected third node to be List"),
    }
}
