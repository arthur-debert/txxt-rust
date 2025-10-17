//! Test case for issue #26: Indented sequence markers not tokenized correctly
//!
//! When a dash list item is indented (e.g., "    - Item"), the dash is not
//! recognized as a SequenceMarker token. Instead, it becomes part of the Text token.
//! This breaks nested list parsing and round-trip detokenization.

use txxt::ast::tokens::Token;
use txxt::lexer::tokenize;

#[test]
fn test_indented_dash_not_recognized_as_sequence_marker() {
    // Minimal test case: indented dash list
    let input = "    - Item";
    let tokens = tokenize(input);

    // Debug output to show the issue
    eprintln!("\nTokens for '{}':", input);
    for (i, token) in tokens.iter().enumerate() {
        eprintln!("{}: {:?}", i, token);
    }

    // The dash should be tokenized as a SequenceMarker
    let has_seq_marker = tokens
        .iter()
        .any(|t| matches!(t, Token::SequenceMarker { .. }));

    assert!(
        has_seq_marker,
        "Indented dash should be recognized as SequenceMarker token"
    );
}

#[test]
fn test_nested_list_structure() {
    // More complete test showing the impact on nested lists
    let input = "1. First\n    - Sub item\n    - Another sub";
    let tokens = tokenize(input);

    // Count sequence markers
    let seq_markers = tokens
        .iter()
        .filter(|t| matches!(t, Token::SequenceMarker { .. }))
        .count();

    assert_eq!(
        seq_markers, 3,
        "Should have 3 sequence markers: one for '1.' and two for dashes"
    );
}

#[test]
fn test_deeply_nested_lists() {
    let input = "1. Level 1\n    a. Level 2\n        - Level 3";
    let tokens = tokenize(input);

    let seq_markers = tokens
        .iter()
        .filter(|t| matches!(t, Token::SequenceMarker { .. }))
        .count();

    assert_eq!(
        seq_markers, 3,
        "All three list markers should be recognized regardless of indentation"
    );
}
