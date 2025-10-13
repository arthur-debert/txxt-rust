//! Integration tests for indentation tokenization
//!
//! Tests the integration of IndentationTracker with the main lexer to ensure
//! Indent and Dedent tokens are properly generated for TXXT text.

use rstest::rstest;
use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

/// Test basic indentation tokenization
#[test]
fn test_basic_indentation_integration() {
    let input = "base level\n    indented content\nback to base";
    let tokens = tokenize(input);

    // Find Indent and Dedent tokens
    let indent_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Indent { .. }))
        .collect();

    let dedent_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Dedent { .. }))
        .collect();

    assert_eq!(indent_tokens.len(), 1, "Should have one Indent token");
    assert_eq!(dedent_tokens.len(), 1, "Should have one Dedent token");
}

/// Test multiple indentation levels
#[test]
fn test_multiple_indentation_levels() {
    let input = "level 0\n    level 1\n        level 2\n    back to level 1\nback to level 0";
    let tokens = tokenize(input);

    let indent_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Indent { .. }))
        .collect();

    let dedent_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Dedent { .. }))
        .collect();

    assert_eq!(indent_tokens.len(), 2, "Should have two Indent tokens");
    assert_eq!(
        dedent_tokens.len(),
        2,
        "Should have two Dedent tokens (one for level 2->1, one for level 1->0)"
    );
}

/// Test that empty lines don't affect indentation
#[test]
fn test_empty_lines_ignored() {
    let input = "base level\n\n    indented content\n\nback to base";
    let tokens = tokenize(input);

    let indent_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Indent { .. }))
        .collect();

    let dedent_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Dedent { .. }))
        .collect();

    assert_eq!(
        indent_tokens.len(),
        1,
        "Should have one Indent token despite empty lines"
    );
    assert_eq!(
        dedent_tokens.len(),
        1,
        "Should have one Dedent token despite empty lines"
    );
}

/// Test tab normalization in integrated tokenization
#[test]
fn test_tab_indentation_integration() {
    let input = "base level\n\tindented with tab\nback to base";
    let tokens = tokenize(input);

    let indent_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Indent { .. }))
        .collect();

    assert_eq!(indent_tokens.len(), 1, "Should handle tab indentation");

    // Check that the span reflects the normalized indentation (4 spaces)
    if let Token::Indent { span } = &indent_tokens[0] {
        assert_eq!(span.end.column, 4, "Tab should be normalized to 4 spaces");
    }
}

/// Test complex indentation patterns
#[rstest]
#[case(
    "base\n    level 1\n        level 2\n            level 3\nback to base",
    3,
    3
)]
#[case("base\n    level 1\n        level 2\n    back to level 1", 2, 2)] // Expect 2 dedents: one for level 2->1, one for finalize
#[case("base\n    only one level\nback to base", 1, 1)]
fn test_indentation_patterns(
    #[case] input: &str,
    #[case] expected_indents: usize,
    #[case] expected_dedents: usize,
) {
    let tokens = tokenize(input);

    let indent_count = tokens
        .iter()
        .filter(|token| matches!(token, Token::Indent { .. }))
        .count();

    let dedent_count = tokens
        .iter()
        .filter(|token| matches!(token, Token::Dedent { .. }))
        .count();

    assert_eq!(
        indent_count, expected_indents,
        "Incorrect number of Indent tokens"
    );
    assert_eq!(
        dedent_count, expected_dedents,
        "Incorrect number of Dedent tokens"
    );
}

/// Test that indentation tokens have correct spans
#[test]
fn test_indentation_token_spans() {
    let input = "base\n    indented\nback";
    let tokens = tokenize(input);

    let indent_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Indent { .. }))
        .expect("Should find Indent token");

    if let Token::Indent { span } = indent_token {
        assert_eq!(span.start.row, 1, "Indent should be on line 1 (0-indexed)");
        assert_eq!(span.start.column, 0, "Indent should start at column 0");
        assert_eq!(span.end.column, 4, "Indent should end at column 4");
    }

    let dedent_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Dedent { .. }))
        .expect("Should find Dedent token");

    if let Token::Dedent { span } = dedent_token {
        assert_eq!(span.start.row, 2, "Dedent should be on line 2 (0-indexed)");
        assert_eq!(span.start.column, 0, "Dedent should start at column 0");
    }
}

/// Test interaction with other tokens
#[test]
fn test_indentation_with_other_tokens() {
    let input = "paragraph content\n    - list item\n        nested content\n    another item";
    let tokens = tokenize(input);

    // Should have indentation tokens
    let indent_count = tokens
        .iter()
        .filter(|token| matches!(token, Token::Indent { .. }))
        .count();
    assert!(indent_count > 0, "Should have indentation tokens");

    // Should also have text and other tokens
    let text_count = tokens
        .iter()
        .filter(|token| matches!(token, Token::Text { .. }))
        .count();
    assert!(text_count > 0, "Should have text tokens");

    // Note: Sequence markers have specific parsing rules and may not be recognized
    // in this context. The important thing is that indentation processing works
    // alongside other tokenization without interfering.

    // Should have newline tokens
    let newline_count = tokens
        .iter()
        .filter(|token| matches!(token, Token::Newline { .. }))
        .count();
    assert!(newline_count > 0, "Should have newline tokens");
}
