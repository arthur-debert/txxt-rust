//! Tests for TEXT token tokenization using rstest and proptest
//!
//! Tests both successful parsing and failure cases for TEXT tokens
//! with precise SourceSpan positioning.

use proptest::prelude::*;
use rstest::rstest;
use txxt::ast::tokens::Token;
use txxt::tokenizer::{infrastructure::patterns::TEXT_PATTERN, tokenize};

// =============================================================================
// TEXT Token - Isolated Tests (rstest)
// =============================================================================

#[rstest]
#[case("hello", "hello")]
#[case("world", "world")]
#[case("simple", "simple")]
#[case("word123", "word123")]
#[case("underscore_text", "underscore_text")]
fn test_text_token_isolated_passing(#[case] input: &str, #[case] expected_content: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // TEXT + EOF

    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, expected_content);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, expected_content.len());
        }
        _ => panic!("Expected Text token, got {:?}", tokens[0]),
    }

    // Should end with EOF
    match &tokens[1] {
        Token::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[1]),
    }
}

#[rstest]
#[case("hello world", "hello", "world")]
#[case("first second", "first", "second")]
#[case("one two", "one", "two")]
fn test_text_token_with_more_content_passing(
    #[case] input: &str,
    #[case] first_word: &str,
    #[case] second_word: &str,
) {
    let tokens = tokenize(input);

    // Should have: TEXT, WHITESPACE, TEXT, EOF
    assert!(tokens.len() >= 3);

    // First text token
    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, first_word);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, first_word.len());
        }
        _ => panic!("Expected first Text token, got {:?}", tokens[0]),
    }

    // Find second text token (skipping whitespace)
    let second_text_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Text { content, .. } if content == second_word))
        .expect("Should find second text token");

    match second_text_token {
        Token::Text { content, span } => {
            assert_eq!(content, second_word);
            assert_eq!(span.start.row, 0);
            assert!(span.start.column > first_word.len()); // After first word + space
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// TEXT Token - Failing Cases (rstest)
// =============================================================================

#[rstest]
#[case("")] // Empty input should not produce TEXT token
#[case(" ")] // Only whitespace should not produce TEXT token
#[case("\n")] // Only newline should not produce TEXT token
#[case("\t")] // Only tab should not produce TEXT token
fn test_text_token_isolated_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any TEXT tokens
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Text { .. }))
        .collect();

    assert_eq!(
        text_tokens.len(),
        0,
        "Input '{}' should not produce TEXT tokens, but got: {:?}",
        input.escape_debug(),
        text_tokens
    );
}

#[rstest]
#[case("  hello", "hello")] // Leading whitespace - should still parse text
#[case("hello  ", "hello")] // Trailing whitespace - should still parse text
#[case("  hello  ", "hello")] // Both - should still parse text
fn test_text_token_with_whitespace_failing_edge_cases(
    #[case] input: &str,
    #[case] expected_text: &str,
) {
    let tokens = tokenize(input);

    // Find the text token
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // Should have exactly one text token with expected content
    assert_eq!(text_tokens.len(), 1);
    assert_eq!(text_tokens[0], expected_text);
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_text_token_properties(text in TEXT_PATTERN) {
        // Only test valid text characters to ensure we get TEXT tokens
        prop_assume!(!text.is_empty());

        let tokens = tokenize(&text);

        // Should have at least one TEXT token
        let text_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, Token::Text { .. }))
            .collect();

        prop_assert!(!text_tokens.is_empty(), "Should produce at least one TEXT token");

        // First token should be TEXT if input starts with valid text chars
        // Note: Standalone _ is an italic delimiter, not text
        if text.chars().next().unwrap().is_alphanumeric() {
            match &tokens[0] {
                Token::Text { content, span } => {
                    prop_assert_eq!(span.start.row, 0);
                    prop_assert_eq!(span.start.column, 0);
                    prop_assert!(span.end.column > 0);
                    prop_assert!(!content.is_empty());
                }
                _ => prop_assert!(false, "Expected first token to be Text"),
            }
        }
    }

    #[test]
    fn test_text_token_span_consistency(text in "[a-zA-Z]+") {
        prop_assume!(!text.is_empty());

        let tokens = tokenize(&text);

        for token in &tokens {
            if let Token::Text { content, span } = token {
                // Span should be consistent with content length
                prop_assert_eq!(
                    span.end.column - span.start.column,
                    content.len(),
                    "Span length should match content length"
                );

                // Start should come before end
                prop_assert!(span.start.column <= span.end.column);
                prop_assert!(span.start.row <= span.end.row);
            }
        }
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("test");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_underscore_produces_italic_delimiter() {
        // Verify that standalone underscore produces ItalicDelimiter, not Text
        let tokens = tokenize("_");
        assert_eq!(tokens.len(), 2); // ItalicDelimiter + EOF

        match &tokens[0] {
            Token::ItalicDelimiter { span } => {
                assert_eq!(span.start.row, 0);
                assert_eq!(span.start.column, 0);
                assert_eq!(span.end.row, 0);
                assert_eq!(span.end.column, 1);
            }
            _ => panic!("Expected ItalicDelimiter, got {:?}", tokens[0]),
        }
    }
}
