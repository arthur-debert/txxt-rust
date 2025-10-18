//! Tests for BLANKLINE token tokenization using rstest and proptest
//!
//! Tests both successful parsing and failure cases for BLANKLINE tokens
//! Blank lines are empty lines (possibly with whitespace) that separate content blocks

use proptest::prelude::*;
use rstest::rstest;
use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::tokenize;

// =============================================================================
// BLANKLINE Token - Isolated Tests (rstest)
// =============================================================================

#[rstest]
#[case("\n\n")] // Two newlines = blank line
#[case("\n \n")] // Newline + space + newline = blank line
#[case("\n\t\n")] // Newline + tab + newline = blank line
#[case("\n   \n")] // Newline + spaces + newline = blank line
fn test_blankline_isolated_passing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should contain at least one BlankLine token
    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
        .collect();

    assert_eq!(
        blankline_tokens.len(),
        1,
        "Should have exactly one BlankLine token"
    );

    match blankline_tokens[0] {
        ScannerToken::BlankLine { span, .. } => {
            assert_eq!(span.start.row, 1); // After first newline
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 2); // Before second newline
            assert_eq!(span.end.column, 0);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("text\n\nmore", "text", "more")]
#[case("first\n\nsecond", "first", "second")]
#[case("hello\n \nworld", "hello", "world")]
#[case("line1\n\t\nline2", "line1", "line2")]
fn test_blankline_with_content_passing(
    #[case] input: &str,
    #[case] expected_first_text: &str,
    #[case] expected_second_text: &str,
) {
    let tokens = tokenize(input);

    // Should have: TEXT, NEWLINE, BLANKLINE, NEWLINE, TEXT, EOF (at minimum)
    assert!(tokens.len() >= 5);

    // First token should be text
    match &tokens[0] {
        ScannerToken::Text { content, span } => {
            assert_eq!(content, expected_first_text);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => panic!("Expected first Text token, got {:?}", tokens[0]),
    }

    // Should have exactly one BlankLine token
    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
        .collect();

    assert_eq!(
        blankline_tokens.len(),
        1,
        "Should find exactly 1 BlankLine token"
    );

    // Find the second text token (after blank line)
    let second_text_token = tokens
        .iter()
        .skip_while(|token| !matches!(token, ScannerToken::BlankLine { .. }))
        .skip(1) // Skip the blank line
        .find(
            |token| matches!(token, ScannerToken::Text { content, .. } if content == expected_second_text),
        )
        .expect("Should find second text token");

    match second_text_token {
        ScannerToken::Text { content, span } => {
            assert_eq!(content, expected_second_text);
            assert!(span.start.row >= 2); // After blank line
            assert_eq!(span.start.column, 0);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("paragraph\n\n1. List")]
#[case(":: title ::\n\nContent")]
#[case("- Item\n\n2. Next")]
fn test_blankline_with_structured_content(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should contain exactly one BlankLine token separating structure
    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
        .collect();

    assert_eq!(
        blankline_tokens.len(),
        1,
        "Should find exactly 1 BlankLine token"
    );

    // BlankLine should be between structured elements
    match blankline_tokens[0] {
        ScannerToken::BlankLine { span, .. } => {
            assert_eq!(span.start.row, 1); // After first line
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 2); // Before second line
            assert_eq!(span.end.column, 0);
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// BLANKLINE Token - Multiple Blank Lines
// =============================================================================

#[rstest]
#[case("text\n\n\nmore", 2)] // Two consecutive blank lines -> 2 BlankLine tokens
#[case("text\n\n\n\nmore", 3)] // Three consecutive blank lines -> 3 BlankLine tokens
#[case("text\n \n \nmore", 2)] // Blank lines with spaces -> 2 BlankLine tokens
fn test_blankline_multiple_consecutive(#[case] input: &str, #[case] expected_count: usize) {
    let tokens = tokenize(input);

    // Each blank line should produce its own BlankLine token
    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
        .collect();

    assert_eq!(
        blankline_tokens.len(),
        expected_count,
        "Should produce {} BlankLine token(s) for input '{:?}'",
        expected_count,
        input
    );
}

#[rstest]
#[case("text\n\nfirst\n\nsecond", 2)] // Two separate blank lines
#[case("a\n\nb\n\nc\n\nd", 3)] // Three separate blank lines
fn test_blankline_multiple_separated(#[case] input: &str, #[case] expected_count: usize) {
    let tokens = tokenize(input);

    // Separated blank lines should each produce a BlankLine token
    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
        .collect();

    assert_eq!(
        blankline_tokens.len(),
        expected_count,
        "Should produce {} BlankLine tokens for input '{:?}'",
        expected_count,
        input
    );
}

// =============================================================================
// BLANKLINE Token - Failing Cases (rstest)
// =============================================================================

#[rstest]
#[case("")] // Empty input
#[case("\n")] // Single newline only
#[case("text")] // No newlines
#[case("text\nmore")] // Single newline between content
#[case(" ")] // Just spaces
#[case("\t")] // Just tabs
fn test_blankline_isolated_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any BLANKLINE tokens
    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
        .collect();

    assert_eq!(
        blankline_tokens.len(),
        0,
        "Input '{}' should not produce BLANKLINE tokens, but got: {:?}",
        input,
        blankline_tokens
    );
}

#[rstest]
#[case("\n\n\n")] // Only blank lines
#[case("  \n\n  ")] // Whitespace around blank lines
#[case("\n  \n\n")] // Mixed whitespace patterns
fn test_blankline_edge_cases(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should handle edge cases gracefully without crashing
    assert!(
        !tokens.is_empty(),
        "Should produce some tokens without crashing"
    );

    // Should produce at least one BlankLine token for valid blank line patterns
    let blankline_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
        .collect();

    assert!(
        !blankline_tokens.is_empty(),
        "Should produce at least 1 BlankLine token for blank line input '{:?}'",
        input
    );
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_blankline_basic_properties(text in "[a-zA-Z0-9]+") {
        // Test text with blank line separation
        let input = format!("{}\n\n{}", text, text);
        let tokens = tokenize(&input);

        // Should have exactly 1 BLANKLINE token
        let blankline_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
            .collect();

        prop_assert_eq!(blankline_tokens.len(), 1, "Should produce exactly 1 BLANKLINE token");

        // Should have exactly 2 TEXT tokens
        let text_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::Text { content, .. } if content == &text))
            .collect();

        prop_assert_eq!(text_tokens.len(), 2, "Should produce exactly 2 TEXT tokens");
    }

    #[test]
    fn test_blankline_span_consistency(
        whitespace in r"[ \t]*"
    ) {
        // Test blank line with various whitespace patterns
        let input = format!("\n{}\n", whitespace);
        let tokens = tokenize(&input);

        for token in &tokens {
            if let ScannerToken::BlankLine { span, .. } = token {
                // Blank lines should span from after first newline to before next content
                prop_assert_eq!(span.start.row, 1,
                    "BlankLine should start on row 1 (after first newline)");

                prop_assert_eq!(span.start.column, 0,
                    "BlankLine should start at column 0");

                // End should be at start of next row
                prop_assert_eq!(span.end.column, 0,
                    "BlankLine should end at column 0");

                // Start should come before end
                prop_assert!(span.start.row <= span.end.row);
            }
        }
    }

    #[test]
    fn test_blankline_whitespace_variations(
        spaces in 0usize..=5,
        tabs in 0usize..=3
    ) {
        // Test blank lines with different amounts of whitespace
        let whitespace = " ".repeat(spaces) + &"\t".repeat(tabs);
        let input = format!("text\n{}\nmore", whitespace);
        let tokens = tokenize(&input);

        // Should always produce exactly 1 BlankLine token regardless of whitespace
        let blankline_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
            .collect();

        prop_assert_eq!(blankline_tokens.len(), 1,
            "Should produce 1 BlankLine token regardless of whitespace amount");
    }

    #[test]
    fn test_multiple_blank_line_groups(
        text_segments in prop::collection::vec("[a-zA-Z0-9]+", 2..=4)
    ) {
        // Test multiple text segments separated by blank lines
        let input = text_segments.join("\n\n");
        let tokens = tokenize(&input);

        // Should have exactly text_segments.len() - 1 BLANKLINE tokens
        let blankline_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::BlankLine { .. }))
            .collect();

        prop_assert_eq!(blankline_tokens.len(), text_segments.len() - 1,
            "Should produce {} BlankLine tokens for {} text segments",
            text_segments.len() - 1, text_segments.len());

        // Should have exactly text_segments.len() TEXT tokens
        let text_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::Text { .. }))
            .collect();

        prop_assert_eq!(text_tokens.len(), text_segments.len(),
            "Should produce {} Text tokens for {} text segments",
            text_segments.len(), text_segments.len());
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("hello\n\nworld");
        assert!(!tokens.is_empty());
    }
}
