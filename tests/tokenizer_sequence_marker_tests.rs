//! Tests for SEQUENCE_MARKER token tokenization using rstest and proptest
//!
//! Tests both successful parsing and failure cases for SEQUENCE_MARKER tokens
//! based on the NumberingStyle enum from src/ast/structure.rs

use proptest::prelude::*;
use rstest::rstest;
use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

// =============================================================================
// SEQUENCE_MARKER Token - Isolated Tests (rstest)
// =============================================================================

#[rstest]
// Plain style (dash)
#[case("- ", "-")]
// Numerical style
#[case("1. ", "1.")]
#[case("2. ", "2.")]
#[case("42. ", "42.")]
// Alphabetical lowercase
#[case("a. ", "a.")]
#[case("b. ", "b.")]
#[case("a) ", "a)")]
#[case("z) ", "z)")]
// Alphabetical uppercase
#[case("A. ", "A.")]
#[case("B. ", "B.")]
#[case("A) ", "A)")]
#[case("Z) ", "Z)")]
// Roman numerals lowercase
#[case("i. ", "i.")]
#[case("ii. ", "ii.")]
#[case("iii. ", "iii.")]
#[case("iv. ", "iv.")]
#[case("v. ", "v.")]
#[case("i) ", "i)")]
#[case("ii) ", "ii)")]
// Roman numerals uppercase
#[case("I. ", "I.")]
#[case("II. ", "II.")]
#[case("III. ", "III.")]
#[case("IV. ", "IV.")]
#[case("V. ", "V.")]
#[case("I) ", "I)")]
#[case("II) ", "II)")]
fn test_sequence_marker_isolated_passing(#[case] input: &str, #[case] expected_marker: &str) {
    let tokens = tokenize(input);

    assert!(!tokens.is_empty(), "Should have at least one token");

    match &tokens[0] {
        Token::SequenceMarker { content, span } => {
            assert_eq!(content, expected_marker);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, expected_marker.len());
        }
        _ => panic!("Expected SequenceMarker token, got {:?}", tokens[0]),
    }
}

#[rstest]
#[case("1. First item", "1.", "First")]
#[case("- Bullet item", "-", "Bullet")]
#[case("a) Alpha item", "a)", "Alpha")]
#[case("I. Roman item", "I.", "Roman")]
#[case("42. Numbered item", "42.", "Numbered")]
fn test_sequence_marker_with_content_passing(
    #[case] input: &str,
    #[case] expected_marker: &str,
    #[case] expected_text: &str,
) {
    let tokens = tokenize(input);

    // Should have: SEQUENCE_MARKER, TEXT, EOF (at minimum)
    assert!(tokens.len() >= 2);

    // First token should be sequence marker
    match &tokens[0] {
        Token::SequenceMarker { content, span } => {
            assert_eq!(content, expected_marker);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => panic!("Expected SequenceMarker token, got {:?}", tokens[0]),
    }

    // Find the text token
    let text_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Text { content, .. } if content == expected_text))
        .expect("Should find text token");

    match text_token {
        Token::Text { content, span } => {
            assert_eq!(content, expected_text);
            assert_eq!(span.start.row, 0);
            assert!(span.start.column > expected_marker.len()); // After marker + space
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// SEQUENCE_MARKER Token - Failing Cases (rstest)
// =============================================================================

#[rstest]
// Missing space after marker
#[case("1.")]
#[case("a)")]
#[case("-")]
// Invalid marker patterns
#[case("1 ")] // Missing punctuation
#[case("a ")] // Missing punctuation
#[case("1.. ")] // Double punctuation
#[case("(a) ")] // Wrong punctuation style
#[case("aa. ")] // Multi-letter (invalid for alphabet)
#[case("1a. ")] // Mixed alphanumeric
// Numbers with wrong punctuation
#[case("1) text")] // Numbers should use . not )
// Invalid roman numerals
#[case("iiii. ")] // Should be iv
#[case("iiiii. ")] // Should be v
fn test_sequence_marker_isolated_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any SEQUENCE_MARKER tokens
    let sequence_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::SequenceMarker { .. }))
        .collect();

    assert_eq!(
        sequence_tokens.len(),
        0,
        "Input '{}' should not produce SEQUENCE_MARKER tokens, but got: {:?}",
        input,
        sequence_tokens
    );
}

#[rstest]
// Sequence markers must be at start of line
#[case("  1. item")] // Indented
#[case("text 1. item")] // After text
#[case("before - item")] // After text
fn test_sequence_marker_position_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any SEQUENCE_MARKER tokens (they're not at line start)
    let sequence_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::SequenceMarker { .. }))
        .collect();

    assert_eq!(sequence_tokens.len(), 0,
        "Input '{}' should not produce SEQUENCE_MARKER tokens when not at line start, but got: {:?}",
        input, sequence_tokens);
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_numerical_sequence_markers(num in 1u32..1000u32) {
        let input = format!("{}. ", num);
        let tokens = tokenize(&input);

        // Should have at least one SEQUENCE_MARKER token
        let sequence_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, Token::SequenceMarker { .. }))
            .collect();

        prop_assert_eq!(sequence_tokens.len(), 1, "Should produce exactly one SEQUENCE_MARKER token");

        match &tokens[0] {
            Token::SequenceMarker { content, span } => {
                let expected = format!("{}.", num);
                prop_assert_eq!(content, &expected);
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, expected.len());
            }
            _ => prop_assert!(false, "Expected SequenceMarker token"),
        }
    }

    #[test]
    fn test_alphabetical_sequence_markers(letter_code in 97u8..=122u8) {
        let letter = letter_code as char;
        let input = format!("{}. ", letter);
        let tokens = tokenize(&input);

        // Should have exactly one SEQUENCE_MARKER token
        let sequence_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, Token::SequenceMarker { .. }))
            .collect();

        prop_assert_eq!(sequence_tokens.len(), 1);

        match &tokens[0] {
            Token::SequenceMarker { content, span } => {
                let expected = format!("{}.", letter);
                prop_assert_eq!(content, &expected);
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, expected.len());
            }
            _ => prop_assert!(false, "Expected SequenceMarker token"),
        }
    }

    #[test]
    fn test_sequence_marker_span_consistency(
        marker in r"(1\.|2\.|a\.|b\.|A\.|B\.|i\.|ii\.|I\.|II\.|-)"
    ) {
        let input = format!("{} ", marker);
        let tokens = tokenize(&input);

        for token in &tokens {
            if let Token::SequenceMarker { content, span } = token {
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
        let tokens = tokenize("1. test");
        assert!(!tokens.is_empty());
    }
}
