//! Tests for Dash token recognition using rstest and proptest
//!
//! Tests both successful parsing and failure cases for standalone dash tokens.
//! Dash tokens are single '-' characters that are not part of sequence markers.

use rstest::rstest;
use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::tokenize;

// =============================================================================
// Dash Token - Isolated Tests (rstest)
// =============================================================================

#[rstest]
#[case("-")]
fn test_dash_token_isolated_passing(#[case] input: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // DASH + EOF

    match &tokens[0] {
        ScannerToken::Dash { span } => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 1);
        }
        _ => panic!("Expected Dash token, got {:?}", tokens[0]),
    }

    // Should end with EOF
    match &tokens[1] {
        ScannerToken::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[1]),
    }
}

#[rstest]
#[case("hello-world")]
#[case("text-more-text")]
fn test_dash_token_with_content_passing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should have TEXT tokens separated by DASH tokens
    let mut dash_count = 0;
    let mut text_count = 0;

    for token in &tokens {
        match token {
            ScannerToken::Dash { .. } => {
                dash_count += 1;
            }
            ScannerToken::Text { .. } => {
                text_count += 1;
            }
            ScannerToken::Eof { .. } => {
                // Expected at the end
            }
            _ => {
                // Other tokens are fine, just ignore for this test
            }
        }
    }

    // Should have at least one dash and multiple text parts
    assert!(dash_count > 0, "Should have at least one dash token");
    assert!(text_count > 1, "Should have multiple text tokens");
}

#[rstest]
#[case("before-after", "before", "after")]
#[case("word-another", "word", "another")]
fn test_dash_token_mixed_content(
    #[case] input: &str,
    #[case] first_text: &str,
    #[case] second_text: &str,
) {
    let tokens = tokenize(input);

    // Find first text token
    let first_text_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { content, .. } if content == first_text))
        .expect("Should find first text token");

    match first_text_token {
        ScannerToken::Text { content, .. } => {
            assert_eq!(content, first_text);
        }
        _ => unreachable!(),
    }

    // Find dash token
    let dash_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Dash { .. }))
        .expect("Should find dash token");

    match dash_token {
        ScannerToken::Dash { span } => {
            assert!(span.start.column > 0); // Should not be at start of line
        }
        _ => unreachable!(),
    }

    // Find second text token
    let second_text_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { content, .. } if content == second_text))
        .expect("Should find second text token");

    match second_text_token {
        ScannerToken::Text { content, .. } => {
            assert_eq!(content, second_text);
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// Dash Token - Failing Cases (rstest)
// =============================================================================

#[rstest]
#[case("- ")] // Sequence marker, not standalone dash
#[case("-  ")] // Sequence marker with extra space
fn test_dash_token_failing_sequence_markers(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any DASH tokens (these should be SEQUENCE_MARKER)
    let dash_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::Dash { .. }))
        .collect();

    assert_eq!(
        dash_tokens.len(),
        0,
        "Input '{}' should not produce DASH tokens, but got: {:?}",
        input,
        dash_tokens
    );

    // Should contain sequence marker instead
    let sequence_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::SequenceMarker { .. }))
        .collect();

    assert_eq!(
        sequence_tokens.len(),
        1,
        "Input '{}' should produce 1 SEQUENCE_MARKER token, but got: {:?}",
        input,
        sequence_tokens
    );
}

#[rstest]
#[case("")]
#[case(" ")]
#[case("text")]
fn test_dash_token_failing_no_dash(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any DASH tokens
    let dash_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::Dash { .. }))
        .collect();

    assert_eq!(
        dash_tokens.len(),
        0,
        "Input '{}' should not produce DASH tokens, but got: {:?}",
        input,
        dash_tokens
    );
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("-");
        assert!(!tokens.is_empty());
    }
}
