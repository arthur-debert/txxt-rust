//! Tests for VerbatimBlockStart, VerbatimContentLine, and VerbatimBlockEnd token recognition
//!
//! Tests the integration between verbatim scanner and tokenizer to ensure
//! verbatim blocks are correctly identified and tokenized with the new token structure.

use rstest::rstest;
use txxt::cst::ScannerToken;
use txxt::syntax::tokenize;

// =============================================================================
// VerbatimBlockStart, VerbatimContentLine, and VerbatimBlockEnd Token Tests
// =============================================================================

#[rstest]
#[case(
    "simple title:\n    content line\n:: label\n",
    "simple title",
    vec!["    content line"]
)]
#[case(
    "test:\n    line 1\n    line 2\n:: block\n",
    "test",
    vec!["    line 1", "    line 2"]
)]
fn test_verbatim_block_basic(
    #[case] input: &str,
    #[case] expected_title: &str,
    #[case] expected_content_lines: Vec<&str>,
) {
    let tokens = tokenize(input);

    // Find VerbatimBlockStart token
    let verbatim_start = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::VerbatimBlockStart { .. }))
        .expect("Should find VerbatimBlockStart token");

    match verbatim_start {
        ScannerToken::VerbatimBlockStart { title, .. } => {
            assert_eq!(title, expected_title);
        }
        _ => unreachable!(),
    }

    // Find all VerbatimContentLine tokens
    let content_lines: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::VerbatimContentLine {
                content,
                indentation,
                ..
            } => Some(format!("{}{}", indentation, content)),
            _ => None,
        })
        .collect();

    assert_eq!(
        content_lines.len(),
        expected_content_lines.len(),
        "Should have {} content lines",
        expected_content_lines.len()
    );

    for (i, expected) in expected_content_lines.iter().enumerate() {
        assert_eq!(
            &content_lines[i], expected,
            "Content line {} should match",
            i
        );
    }

    // Find VerbatimBlockEnd token
    let verbatim_end = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
        .expect("Should find VerbatimBlockEnd token");

    match verbatim_end {
        ScannerToken::VerbatimBlockEnd { .. } => {
            // Token exists, test passes
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("empty block:\n:: empty\n")]
#[case("no content:\n:: label\n")]
fn test_verbatim_block_empty(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should have VerbatimBlockStart and VerbatimBlockEnd but no VerbatimContentLine
    let verbatim_starts: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::VerbatimBlockStart { .. }))
        .collect();

    let verbatim_contents: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::VerbatimContentLine { .. }))
        .collect();

    let verbatim_ends: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
        .collect();

    assert_eq!(
        verbatim_starts.len(),
        1,
        "Should have one VerbatimBlockStart token"
    );
    assert_eq!(
        verbatim_contents.len(),
        0,
        "Should have no VerbatimContentLine tokens for empty block"
    );
    assert_eq!(
        verbatim_ends.len(),
        1,
        "Should have one VerbatimBlockEnd token"
    );
}

#[rstest]
#[case(
    "stretched:\ncontent at column 0\n:: label\n",
    "stretched",
    vec!["content at column 0"]
)]
#[case(
    "stretched block:\nline 1\nline 2\n:: label\n",
    "stretched block",
    vec!["line 1", "line 2"]
)]
fn test_verbatim_block_stretched(
    #[case] input: &str,
    #[case] expected_title: &str,
    #[case] expected_content_lines: Vec<&str>,
) {
    let tokens = tokenize(input);

    // Find VerbatimBlockStart token
    let verbatim_start = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::VerbatimBlockStart { .. }))
        .expect("Should find VerbatimBlockStart token");

    match verbatim_start {
        ScannerToken::VerbatimBlockStart { title, .. } => {
            assert_eq!(title, expected_title);
        }
        _ => unreachable!(),
    }

    // Find all VerbatimContentLine tokens and reconstruct content
    let content_lines: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::VerbatimContentLine {
                content,
                indentation,
                ..
            } => Some(format!("{}{}", indentation, content)),
            _ => None,
        })
        .collect();

    assert_eq!(
        content_lines.len(),
        expected_content_lines.len(),
        "Should have {} content lines",
        expected_content_lines.len()
    );

    for (i, expected) in expected_content_lines.iter().enumerate() {
        assert_eq!(
            &content_lines[i], expected,
            "Content line {} should match",
            i
        );
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
        .filter(|token| matches!(token, ScannerToken::VerbatimBlockStart { .. }))
        .collect();

    let verbatim_contents: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::VerbatimContentLine { .. }))
        .collect();

    let verbatim_ends: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
        .collect();

    assert_eq!(
        verbatim_starts.len(),
        0,
        "Should not have VerbatimBlockStart tokens for non-verbatim content"
    );
    assert_eq!(
        verbatim_contents.len(),
        0,
        "Should not have VerbatimContentLine tokens for non-verbatim content"
    );
    assert_eq!(
        verbatim_ends.len(),
        0,
        "Should not have VerbatimBlockEnd tokens for non-verbatim content"
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
