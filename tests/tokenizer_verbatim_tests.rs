//! Tests for VerbatimStart and VerbatimContent token recognition
//!
//! Tests the integration between verbatim scanner and tokenizer to ensure
//! verbatim blocks are correctly identified and tokenized.

use rstest::rstest;
use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

// =============================================================================
// VerbatimStart and VerbatimContent Token Tests
// =============================================================================

#[rstest]
#[case(
    "simple title:\n    content line\n()\n",
    "simple title:",
    "    content line"
)]
#[case(
    "test:\n    line 1\n    line 2\n(block)\n",
    "test:",
    "    line 1\n    line 2"
)]
fn test_verbatim_block_basic(
    #[case] input: &str,
    #[case] expected_title: &str,
    #[case] expected_content: &str,
) {
    let tokens = tokenize(input);

    // Find VerbatimStart token
    let verbatim_start = tokens
        .iter()
        .find(|token| matches!(token, Token::VerbatimStart { .. }))
        .expect("Should find VerbatimStart token");

    match verbatim_start {
        Token::VerbatimStart { content, span } => {
            assert_eq!(content, expected_title);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => unreachable!(),
    }

    // Find VerbatimContent token
    let verbatim_content = tokens
        .iter()
        .find(|token| matches!(token, Token::VerbatimContent { .. }))
        .expect("Should find VerbatimContent token");

    match verbatim_content {
        Token::VerbatimContent { content, .. } => {
            assert_eq!(content, expected_content);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("empty block:\n()\n")]
#[case("no content:\n(label)\n")]
fn test_verbatim_block_empty(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should have VerbatimStart but no VerbatimContent
    let verbatim_starts: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::VerbatimStart { .. }))
        .collect();

    let verbatim_contents: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::VerbatimContent { .. }))
        .collect();

    assert_eq!(
        verbatim_starts.len(),
        1,
        "Should have one VerbatimStart token"
    );
    assert_eq!(
        verbatim_contents.len(),
        0,
        "Should have no VerbatimContent tokens for empty block"
    );
}

#[rstest]
#[case(
    "stretched:\ncontent at column 0\n()\n",
    "stretched:",
    "content at column 0"
)]
#[case(
    "stretched block:\nline 1\nline 2\n(label)\n",
    "stretched block:",
    "line 1\nline 2"
)]
fn test_verbatim_block_stretched(
    #[case] input: &str,
    #[case] expected_title: &str,
    #[case] expected_content: &str,
) {
    let tokens = tokenize(input);

    // Find VerbatimStart token
    let verbatim_start = tokens
        .iter()
        .find(|token| matches!(token, Token::VerbatimStart { .. }))
        .expect("Should find VerbatimStart token");

    match verbatim_start {
        Token::VerbatimStart { content, .. } => {
            assert_eq!(content, expected_title);
        }
        _ => unreachable!(),
    }

    // Find VerbatimContent token
    let verbatim_content = tokens
        .iter()
        .find(|token| matches!(token, Token::VerbatimContent { .. }))
        .expect("Should find VerbatimContent token");

    match verbatim_content {
        Token::VerbatimContent { content, .. } => {
            assert_eq!(content, expected_content);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("not verbatim line")]
#[case(":: annotation ::")]
#[case("definition::")]
#[case("line without colon")]
fn test_non_verbatim_content(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any verbatim tokens
    let verbatim_starts: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::VerbatimStart { .. }))
        .collect();

    let verbatim_contents: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::VerbatimContent { .. }))
        .collect();

    assert_eq!(
        verbatim_starts.len(),
        0,
        "Should not have VerbatimStart tokens for non-verbatim content"
    );
    assert_eq!(
        verbatim_contents.len(),
        0,
        "Should not have VerbatimContent tokens for non-verbatim content"
    );
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("title:\n    content\n()\n");
        assert!(!tokens.is_empty());
    }
}
