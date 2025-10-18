//! Tests for VerbatimTitle and IgnoreTextSpan token recognition
//!
//! Tests the integration between verbatim scanner and tokenizer to ensure
//! verbatim blocks are correctly identified and tokenized.

use rstest::rstest;
use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::tokenize;

// =============================================================================
// VerbatimTitle and IgnoreTextSpan Token Tests
// =============================================================================

#[rstest]
#[case(
    "simple title:\n    content line\n:: label\n",
    "simple title",
    "content line"
)]
#[case("test:\n    line 1\n    line 2\n:: block\n", "test", "line 1\nline 2")]
fn test_verbatim_block_basic(
    #[case] input: &str,
    #[case] expected_title: &str,
    #[case] expected_content: &str,
) {
    let tokens = tokenize(input);

    // Find VerbatimTitle token
    let verbatim_start = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::VerbatimTitle { .. }))
        .expect("Should find VerbatimTitle token");

    match verbatim_start {
        ScannerToken::VerbatimTitle { content, span } => {
            assert_eq!(content, expected_title);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => unreachable!(),
    }

    // Find IgnoreTextSpan token
    let verbatim_content = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::IgnoreTextSpan { .. }))
        .expect("Should find IgnoreTextSpan token");

    match verbatim_content {
        ScannerToken::IgnoreTextSpan { content, .. } => {
            assert_eq!(content, expected_content);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("empty block:\n:: empty\n")]
#[case("no content:\n:: label\n")]
fn test_verbatim_block_empty(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should have VerbatimTitle but no IgnoreTextSpan
    let verbatim_starts: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::VerbatimTitle { .. }))
        .collect();

    let verbatim_contents: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::IgnoreTextSpan { .. }))
        .collect();

    assert_eq!(
        verbatim_starts.len(),
        1,
        "Should have one VerbatimTitle token"
    );
    assert_eq!(
        verbatim_contents.len(),
        0,
        "Should have no IgnoreTextSpan tokens for empty block"
    );
}

#[rstest]
#[case(
    "stretched:\ncontent at column 0\n:: label\n",
    "stretched",
    "content at column 0"
)]
#[case(
    "stretched block:\nline 1\nline 2\n:: label\n",
    "stretched block",
    "line 1\nline 2"
)]
fn test_verbatim_block_stretched(
    #[case] input: &str,
    #[case] expected_title: &str,
    #[case] expected_content: &str,
) {
    let tokens = tokenize(input);

    // Find VerbatimTitle token
    let verbatim_start = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::VerbatimTitle { .. }))
        .expect("Should find VerbatimTitle token");

    match verbatim_start {
        ScannerToken::VerbatimTitle { content, .. } => {
            assert_eq!(content, expected_title);
        }
        _ => unreachable!(),
    }

    // Find IgnoreTextSpan token
    let verbatim_content = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::IgnoreTextSpan { .. }))
        .expect("Should find IgnoreTextSpan token");

    match verbatim_content {
        ScannerToken::IgnoreTextSpan { content, .. } => {
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
        .filter(|token| matches!(token, ScannerToken::VerbatimTitle { .. }))
        .collect();

    let verbatim_contents: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::IgnoreTextSpan { .. }))
        .collect();

    assert_eq!(
        verbatim_starts.len(),
        0,
        "Should not have VerbatimTitle tokens for non-verbatim content"
    );
    assert_eq!(
        verbatim_contents.len(),
        0,
        "Should not have IgnoreTextSpan tokens for non-verbatim content"
    );
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("title:\n    content\n:: label\n");
        assert!(!tokens.is_empty());
    }
}
