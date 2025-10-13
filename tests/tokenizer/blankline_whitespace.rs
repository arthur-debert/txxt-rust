//! Tests for BlankLine whitespace preservation (issue #30)

use txxt::parser::detokenizer::Detokenizer;
use txxt::tokenizer::{tokenize, Token};

/// Helper to verify round-trip tokenization for verification purposes
fn verify_blankline_round_trip(original: &str) {
    let tokens1 = tokenize(original);

    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer
        .detokenize_for_verification(&tokens1)
        .expect("Detokenization should succeed");

    assert_eq!(
        original, reconstructed,
        "Round-trip should preserve blank line whitespace"
    );
}

#[test]
fn test_blank_line_with_spaces() {
    let input = "Line one\n    \nLine two";
    verify_blankline_round_trip(input);
}

#[test]
fn test_blank_line_with_tabs() {
    let input = "Line one\n\t\t\nLine two";
    verify_blankline_round_trip(input);
}

#[test]
fn test_blank_line_mixed_whitespace() {
    let input = "Line one\n  \t  \nLine two";
    verify_blankline_round_trip(input);
}

#[test]
fn test_blank_line_no_whitespace() {
    let input = "Line one\n\nLine two";
    verify_blankline_round_trip(input);
}

#[test]
fn test_multiple_blank_lines_with_whitespace() {
    let input = "Line one\n    \n        \n\nLine two";
    verify_blankline_round_trip(input);
}

#[test]
fn test_blank_line_preservation_in_indented_block() {
    let input = "    Indented paragraph\n    \n    More indented text";
    verify_blankline_round_trip(input);
}

#[test]
fn test_blank_line_token_content() {
    let input = "Text\n    \nMore text";
    let tokens = tokenize(input);

    // Find the BlankLine token
    let blank_line = tokens
        .iter()
        .find_map(|t| match t {
            Token::BlankLine { whitespace, .. } => Some(whitespace),
            _ => None,
        })
        .expect("Should have a BlankLine token");

    assert_eq!(blank_line, "    ", "BlankLine should preserve whitespace");
}
