//! Tests for NEWLINE token tokenization using rstest and proptest
//!
//! Tests both successful parsing and failure cases for NEWLINE tokens
//! Newlines are fundamental structural tokens that mark line boundaries

use proptest::prelude::*;
use rstest::rstest;
use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

// =============================================================================
// NEWLINE Token - Isolated Tests (rstest)
// =============================================================================

#[rstest]
#[case("\n")]
#[case("\r\n")] // CRLF should be normalized to LF
fn test_newline_isolated_passing(#[case] input: &str) {
    let tokens = tokenize(input);

    assert!(!tokens.is_empty(), "Should have at least one token");

    match &tokens[0] {
        Token::Newline { span } => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            // Both \n and \r\n should advance to next row
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 0);
        }
        _ => panic!("Expected Newline token, got {:?}", tokens[0]),
    }
}

#[rstest]
#[case("hello\nworld", "hello", "world")]
#[case("first\nsecond\nthird", "first", "second")]
#[case("line1\nline2", "line1", "line2")]
fn test_newline_with_content_passing(
    #[case] input: &str,
    #[case] expected_first_text: &str,
    #[case] expected_second_text: &str,
) {
    let tokens = tokenize(input);

    // Should have: TEXT, NEWLINE, TEXT, EOF (at minimum)
    assert!(tokens.len() >= 3);

    // First token should be text
    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, expected_first_text);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => panic!("Expected first Text token, got {:?}", tokens[0]),
    }

    // Find the newline token
    let newline_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Newline { .. }))
        .expect("Should find newline token");

    match newline_token {
        Token::Newline { span } => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, expected_first_text.len());
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 0);
        }
        _ => unreachable!(),
    }

    // Find the second text token
    let second_text_token = tokens
        .iter()
        .skip_while(|token| !matches!(token, Token::Newline { .. }))
        .skip(1) // Skip the newline
        .find(
            |token| matches!(token, Token::Text { content, .. } if content == expected_second_text),
        )
        .expect("Should find second text token");

    match second_text_token {
        Token::Text { content, span } => {
            assert_eq!(content, expected_second_text);
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 0);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("1. Item\n2. Next")]
#[case("- First\n- Second")]
#[case(":: title ::\nContent here")]
fn test_newline_with_structured_content(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should contain at least one newline
    let newline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Newline { .. }))
        .collect();

    assert_eq!(
        newline_tokens.len(),
        1,
        "Should find exactly 1 newline token"
    );

    // Newline should separate the two structural elements
    match newline_tokens[0] {
        Token::Newline { span } => {
            assert_eq!(span.start.row, 0);
            assert!(span.start.column > 0); // After first line content
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 0);
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// NEWLINE Token - Edge Cases (rstest)
// =============================================================================

#[rstest]
#[case("")] // Empty input
#[case(" ")] // Just space
#[case("\t")] // Just tab
fn test_newline_isolated_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any NEWLINE tokens
    let newline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Newline { .. }))
        .collect();

    assert_eq!(
        newline_tokens.len(),
        0,
        "Input '{}' should not produce NEWLINE tokens, but got: {:?}",
        input,
        newline_tokens
    );
}

#[rstest]
#[case("text\n", 1)] // Trailing newline
#[case("\ntext", 1)] // Leading newline
fn test_newline_edge_cases(#[case] input: &str, #[case] expected_newlines: usize) {
    let tokens = tokenize(input);

    // Should handle edge cases gracefully
    let newline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Newline { .. }))
        .collect();

    assert_eq!(
        newline_tokens.len(),
        expected_newlines,
        "Should produce {} newline tokens for input '{:?}'",
        expected_newlines,
        input
    );
}

#[test]
fn test_double_newline_produces_newline_and_blankline() {
    // Special case: "\n\n" should produce 1 Newline + 1 BlankLine
    let tokens = tokenize("\n\n");

    let newline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::Newline { .. }))
        .collect();

    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::BlankLine { .. }))
        .collect();

    assert_eq!(newline_tokens.len(), 1, "Should produce 1 Newline token");
    assert_eq!(
        blankline_tokens.len(),
        1,
        "Should produce 1 BlankLine token"
    );
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_newline_basic_properties(text in "[a-zA-Z0-9]+") {
        // Test text with single newline
        let input = format!("{}\n", text);
        let tokens = tokenize(&input);

        // Should have exactly 1 NEWLINE token
        let newline_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, Token::Newline { .. }))
            .collect();

        prop_assert_eq!(newline_tokens.len(), 1, "Should produce exactly 1 NEWLINE token");

        // Should have exactly 1 TEXT token
        let text_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, Token::Text { content, .. } if content == &text))
            .collect();

        prop_assert_eq!(text_tokens.len(), 1, "Should produce exactly 1 TEXT token");
    }

    #[test]
    fn test_newline_span_consistency(
        input in r"\n"
    ) {
        let tokens = tokenize(&input);

        for token in &tokens {
            if let Token::Newline { span } = token {
                // Newlines should advance to next row
                prop_assert_eq!(span.end.row, span.start.row + 1,
                    "Newline should advance row by 1");

                // End column should be 0 (start of next line)
                prop_assert_eq!(span.end.column, 0,
                    "Newline end should be at column 0");

                // Start should come before end
                prop_assert!(span.start.row <= span.end.row);
            }
        }
    }

    #[test]
    fn test_multiple_newlines(
        lines in prop::collection::vec("[a-zA-Z0-9]+", 1..=5)
    ) {
        // Test multiple lines separated by newlines
        let input = lines.join("\n");
        let tokens = tokenize(&input);

        // Should have exactly lines.len() - 1 NEWLINE tokens
        let newline_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, Token::Newline { .. }))
            .collect();

        prop_assert_eq!(newline_tokens.len(), lines.len() - 1,
            "Should produce {} newline tokens for {} lines", lines.len() - 1, lines.len());

        // Should have exactly lines.len() TEXT tokens
        let text_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, Token::Text { .. }))
            .collect();

        prop_assert_eq!(text_tokens.len(), lines.len(),
            "Should produce {} text tokens for {} lines", lines.len(), lines.len());
    }

    #[test]
    fn test_crlf_normalization(text in "[a-zA-Z0-9]+") {
        // Test that CRLF is handled properly
        let crlf_input = format!("{}\r\n{}", text, text);
        let lf_input = format!("{}\n{}", text, text);

        let crlf_tokens = tokenize(&crlf_input);
        let lf_tokens = tokenize(&lf_input);

        // Should produce same number of tokens regardless of line ending style
        prop_assert_eq!(crlf_tokens.len(), lf_tokens.len(),
            "CRLF and LF should produce same token count");

        // Both should have exactly 1 newline
        let crlf_newlines: Vec<_> = crlf_tokens.iter()
            .filter(|token| matches!(token, Token::Newline { .. }))
            .collect();
        let lf_newlines: Vec<_> = lf_tokens.iter()
            .filter(|token| matches!(token, Token::Newline { .. }))
            .collect();

        prop_assert_eq!(crlf_newlines.len(), 1, "CRLF should produce 1 newline");
        prop_assert_eq!(lf_newlines.len(), 1, "LF should produce 1 newline");
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("hello\nworld");
        assert!(!tokens.is_empty());
    }
}
