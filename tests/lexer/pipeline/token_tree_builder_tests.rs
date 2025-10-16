//! Tests for Phase 1c: Token Tree Building
//!
//! These tests verify the token tree builder's ability to transform flat token
//! streams into hierarchical structures based on indentation.

use txxt::ast::tokens::{Position, SourceSpan, Token};
use txxt::lexer::pipeline::{TokenTree, TokenTreeBuilder};
use txxt::lexer::tokenize;

/// Test simple flat content (no indentation)
#[test]
fn test_token_tree_builder_flat_content() {
    let source = "This is a simple paragraph.\nNo indentation here.";
    let tokens = tokenize(source);

    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should have tokens at root level, no children
    assert!(!result.tokens.is_empty());
    assert!(result.children.is_empty());
}

/// Test single level of indentation
#[test]
fn test_token_tree_builder_single_indent() {
    let source = "Parent content\n    Indented content\n    More indented";
    let tokens = tokenize(source);

    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should have parent tokens and one child group
    assert!(!result.tokens.is_empty());
    assert_eq!(result.children.len(), 1);
    assert!(!result.children[0].tokens.is_empty());
}

/// Test multiple indent/dedent cycles
#[test]
fn test_token_tree_builder_multiple_indent_cycles() {
    let source = "First block\n    Indented\nSecond block\n    Another indent";
    let tokens = tokenize(source);

    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should have two child groups
    assert_eq!(result.children.len(), 2);
}

/// Test nested indentation
#[test]
fn test_token_tree_builder_nested_indentation() {
    let source = "Level 0\n    Level 1\n        Level 2\n    Back to 1";
    let tokens = tokenize(source);

    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should have nested structure
    assert_eq!(result.children.len(), 1);
    assert_eq!(result.children[0].children.len(), 1);
}

/// Test with walkthrough.txxt - first few lines
#[test]
fn test_token_tree_builder_walkthrough_simple() {
    // First paragraph from walkthrough - completely flat
    let first_paragraph = "TXXT :: A Radical Take on Minimal Structured Text";
    let tokens = tokenize(first_paragraph);

    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should be flat - no indentation
    assert!(!result.tokens.is_empty());
    assert!(result.children.is_empty());
}

/// Test walkthrough with session and indented content
#[test]
fn test_token_tree_builder_walkthrough_with_session() {
    // Create a simplified version with session structure
    let session_example = r#"1. What is TXXT?

    TXXT (pronounced "text") is a structured plain text format.
    
    It features:
    - Human readability
    - Machine parsability"#;

    let tokens = tokenize(session_example);
    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should have session title at root, indented content as child
    assert!(!result.tokens.is_empty());
    assert_eq!(result.children.len(), 1);

    // The child should contain the indented paragraph and list
    let child = &result.children[0];
    assert!(!child.tokens.is_empty());
}

/// Test complex nested structure from walkthrough
#[test]
fn test_token_tree_builder_walkthrough_complex_nesting() {
    // Complex example with multiple levels
    let complex_example = r#"1. Session Title

    Content in session
    
    1.1. Subsession
    
        Content in subsession
        
        - List item
            Nested content in list"#;

    let tokens = tokenize(complex_example);
    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Verify the hierarchical structure
    assert_eq!(result.children.len(), 1); // Session container
    let session_container = &result.children[0];
    assert_eq!(session_container.children.len(), 1); // Subsession container
    let subsession_container = &session_container.children[0];
    assert_eq!(subsession_container.children.len(), 1); // List item container
}

/// Helper function to debug print block group structure
#[cfg(test)]
#[allow(dead_code)]
fn debug_block_structure(tree: &TokenTree, indent: usize) {
    let prefix = " ".repeat(indent * 2);
    println!("{}TokenTree {{", prefix);
    println!("{}  tokens: {} items", prefix, tree.tokens.len());
    for (i, token) in tree.tokens.iter().enumerate() {
        println!("{}    [{}]: {:?}", prefix, i, token_summary(token));
    }
    println!("{}  children: {}", prefix, tree.children.len());
    for (i, child) in tree.children.iter().enumerate() {
        println!("{}  Child {}:", prefix, i);
        debug_block_structure(child, indent + 2);
    }
    println!("{}}}", prefix);
}

/// Create a summary of a token for debugging
#[cfg(test)]
#[allow(dead_code)]
fn token_summary(token: &Token) -> String {
    match token {
        Token::Text { .. } => "Text".to_string(),
        Token::Newline { .. } => "Newline".to_string(),
        Token::BlankLine { .. } => "BlankLine".to_string(),
        Token::Indent { .. } => "Indent".to_string(),
        Token::Dedent { .. } => "Dedent".to_string(),
        Token::SequenceMarker { marker_type, .. } => format!("SeqMarker({:?})", marker_type),
        _ => format!("{:?}", std::mem::discriminant(token)),
    }
}

/// Test error handling - unmatched dedent
#[test]
fn test_token_tree_builder_error_unmatched_dedent() {
    // Create tokens with unmatched dedent
    let tokens = vec![
        Token::Text {
            content: "test".to_string(),
            span: create_test_span(),
        },
        Token::Dedent {
            span: create_test_span(),
        }, // Dedent without indent
    ];

    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens);

    assert!(result.is_err());
}

/// Test progressive walkthrough examples
#[test]
fn test_token_tree_builder_walkthrough_progressive() {
    // Test 1: Just title
    let tokens = tokenize("TXXT :: A Radical Take on Minimal Structured Text");
    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();
    assert!(result.children.is_empty());

    // Test 2: Title + paragraph
    let tokens = tokenize("TXXT :: A Radical Take on Minimal Structured Text\n\nA new format.");
    let result = builder.build_tree(tokens).unwrap();
    assert!(result.children.is_empty()); // Still flat

    // Test 3: Add a session with content
    let tokens = tokenize(
        r#"TXXT :: A Radical Take on Minimal Structured Text

1. Introduction

    This is indented content."#,
    );
    let result = builder.build_tree(tokens).unwrap();
    assert_eq!(result.children.len(), 1); // One indented block

    // Test 4: Multiple sessions
    let tokens = tokenize(
        r#"TXXT :: A Radical Take on Minimal Structured Text

1. First Section

    First content.

2. Second Section

    Second content."#,
    );
    let result = builder.build_tree(tokens).unwrap();
    assert_eq!(result.children.len(), 2); // Two indented blocks
}

/// Test verbatim block handling
#[test]
fn test_token_tree_builder_verbatim_blocks() {
    let verbatim_example = r#"Here is code:

code::
    fn main() {
        println!("Hello");
    }
::"#;

    let tokens = tokenize(verbatim_example);
    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Verbatim content should be grouped
    assert!(!result.tokens.is_empty());
    assert_eq!(result.children.len(), 1); // Verbatim block is indented
}

/// Test empty input
#[test]
fn test_token_tree_builder_empty_input() {
    let tokens = tokenize("");
    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should have only EOF token or be empty
    assert!(result.tokens.len() <= 1); // May have EOF token
    assert!(result.children.is_empty());
}

/// Test multi-level dedent
#[test]
fn test_token_tree_builder_multi_level_dedent() {
    let source = "Level 0\n    Level 1\n        Level 2\nBack to 0";
    let tokens = tokenize(source);

    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Should handle multi-level dedent correctly
    assert_eq!(result.children.len(), 1);
    assert_eq!(result.children[0].children.len(), 1);
}

/// Debug test to visualize structure
#[test]
fn test_token_tree_builder_debug_structure() {
    let source = r#"Root level
    First indent
        Second indent
    Back to first
Another root"#;

    let tokens = tokenize(source);
    let builder = TokenTreeBuilder::new();
    let result = builder.build_tree(tokens).unwrap();

    // Uncomment to debug:
    // debug_block_structure(&result, 0);

    // Verify structure
    assert_eq!(result.children.len(), 1);
    assert_eq!(result.children[0].children.len(), 1);
}

/// Helper to create test spans
#[cfg(test)]
fn create_test_span() -> SourceSpan {
    SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 0, column: 1 },
    }
}
