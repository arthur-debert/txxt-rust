//! Tests for RefMarker token recognition using rstest and proptest
//!
//! Tests both successful parsing and failure cases for reference markers:
//! [file], [#section], [1], [url], etc.
//! NOTE: Citation references ([@...]) are now handled by CitationRef tokens.

use proptest::prelude::*;
use rstest::rstest;
use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

// =============================================================================
// CitationRef Token Tests (rstest) - Citations produce CitationRef, not RefMarker
// =============================================================================

#[rstest]
#[case("[@smith2023]", "smith2023")]
#[case("[@doe_2024]", "doe_2024")]
#[case("[@jones-2025]", "jones-2025")]
#[case("[@author123]", "author123")]
fn test_ref_marker_citation_passing(#[case] input: &str, #[case] expected_content: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // CITATION_REF + EOF

    match &tokens[0] {
        Token::CitationRef { content, span } => {
            assert_eq!(content, expected_content);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected CitationRef token, got {:?}", tokens[0]),
    }
}

// =============================================================================
// RefMarker Token - Section Tests (rstest)
// =============================================================================

#[rstest]
#[case("[#1]", "1")]
#[case("[#2.1]", "2.1")]
#[case("[#3.2.1]", "3.2.1")]
#[case("[#-1]", "-1")] // Last section
#[case("[#-1.2]", "-1.2")] // Second subsection of last section
fn test_ref_marker_section_passing(#[case] input: &str, #[case] expected_content: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // SESSION_REF + EOF

    match &tokens[0] {
        Token::SessionRef { content, span } => {
            assert_eq!(content, expected_content);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected SessionRef token, got {:?}", tokens[0]),
    }
}

// =============================================================================
// FootnoteRef Token - Footnote Tests (rstest)
// =============================================================================

#[rstest]
#[case("[1]", 1)]
#[case("[2]", 2)]
#[case("[42]", 42)]
#[case("[123]", 123)]
fn test_ref_marker_footnote_passing(#[case] input: &str, #[case] expected_number: u32) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // FOOTNOTE_REF + EOF

    match &tokens[0] {
        Token::FootnoteRef {
            footnote_type,
            span,
        } => {
            use txxt::tokenizer::inline::references::footnote_ref::FootnoteType;
            assert_eq!(footnote_type, &FootnoteType::Naked(expected_number));
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected FootnoteRef token, got {:?}", tokens[0]),
    }
}

// =============================================================================
// RefMarker Token - URL Tests (rstest)
// =============================================================================

#[rstest]
#[case("[https://example.com]", "https://example.com")]
#[case("[http://site.org/path]", "http://site.org/path")]
#[case("[www.example.com]", "www.example.com")]
#[case("[user@domain.com]", "user@domain.com")]
fn test_ref_marker_url_passing(#[case] input: &str, #[case] expected_content: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // REF_MARKER + EOF

    match &tokens[0] {
        Token::RefMarker { content, span } => {
            assert_eq!(content, expected_content);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected RefMarker token, got {:?}", tokens[0]),
    }
}

// =============================================================================
// RefMarker Token - File Path Tests (rstest)
// =============================================================================

#[rstest]
#[case("[./file.txxt]", "./file.txxt")]
#[case("[../other/file.md]", "../other/file.md")]
#[case("[/full/path/to/file.txt]", "/full/path/to/file.txt")]
#[case("[document.pdf]", "document.pdf")]
#[case("[image.png]", "image.png")]
fn test_ref_marker_file_path_passing(#[case] input: &str, #[case] expected_content: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // REF_MARKER + EOF

    match &tokens[0] {
        Token::RefMarker { content, span } => {
            assert_eq!(content, expected_content);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected RefMarker token, got {:?}", tokens[0]),
    }
}

// =============================================================================
// RefMarker Token - Plain Anchor Tests (rstest)
// =============================================================================

#[rstest]
#[case("[anchor-name]", "anchor-name")]
#[case("[section_title]", "section_title")]
#[case("[item123]", "item123")]
#[case("[config.setting]", "config.setting")]
fn test_ref_marker_anchor_passing(#[case] input: &str, #[case] expected_content: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // REF_MARKER + EOF

    match &tokens[0] {
        Token::RefMarker { content, span } => {
            assert_eq!(content, expected_content);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected RefMarker token, got {:?}", tokens[0]),
    }
}

// =============================================================================
// RefMarker Token - Context Tests (rstest)
// =============================================================================

#[rstest]
#[case("See [file.txt] for details", "See", "file.txt", "for")]
#[case("Check [some-reference] now", "Check", "some-reference", "now")]
#[case("Read [document.pdf] first", "Read", "document.pdf", "first")]
fn test_ref_marker_in_context_passing(
    #[case] input: &str,
    #[case] prefix_text: &str,
    #[case] expected_ref: &str,
    #[case] suffix_text: &str,
) {
    let tokens = tokenize(input);

    // Should have: TEXT, REF_MARKER, TEXT, EOF (at minimum)
    assert!(tokens.len() >= 4);

    // Find prefix text
    let prefix_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Text { content, .. } if content == prefix_text))
        .expect("Should find prefix text");

    match prefix_token {
        Token::Text { content, span } => {
            assert_eq!(content, prefix_text);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => unreachable!(),
    }

    // Find reference token (could be RefMarker or CitationRef)
    if let Some(citation_content) = expected_ref.strip_prefix('@') {
        // Citation reference - should be CitationRef token
        // Remove @ prefix
        let ref_token = tokens
            .iter()
            .find(|token| matches!(token, Token::CitationRef { content, .. } if content == citation_content))
            .expect("Should find citation reference");

        match ref_token {
            Token::CitationRef { content, .. } => {
                assert_eq!(content, citation_content);
            }
            _ => unreachable!(),
        }
    } else {
        // Other reference types - should be RefMarker token
        let ref_token = tokens
            .iter()
            .find(|token| matches!(token, Token::RefMarker { content, .. } if content == expected_ref))
            .expect("Should find reference marker");

        match ref_token {
            Token::RefMarker { content, .. } => {
                assert_eq!(content, expected_ref);
            }
            _ => unreachable!(),
        }
    }

    // Find suffix text
    let suffix_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Text { content, .. } if content == suffix_text))
        .expect("Should find suffix text");

    match suffix_token {
        Token::Text { content, .. } => {
            assert_eq!(content, suffix_text);
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// RefMarker Token - Failing Cases (rstest)
// =============================================================================

#[rstest]
#[case("[")] // Unclosed bracket
#[case("]")] // No opening bracket
#[case("[]")] // Empty content
#[case("[  ]")]
// Only whitespace
// #[case("[invalid content with spaces]")] // Now accepted by tokenizer, classified during parsing
#[case("[@]")] // Citation without identifier
#[case("[#]")] // Section without number
               // #[case("[#abc]")] // Now accepted by tokenizer, classified during parsing
fn test_ref_marker_isolated_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any REF_MARKER tokens
    let ref_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::RefMarker { .. }))
        .collect();

    assert_eq!(
        ref_tokens.len(),
        0,
        "Input '{}' should not produce REF_MARKER tokens, but got: {:?}",
        input,
        ref_tokens
    );
}

#[rstest]
#[case("[text\nmore]")] // Cannot span lines
#[case("[text\rmore]")] // Cannot span lines (CRLF)
fn test_ref_marker_multiline_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any REF_MARKER tokens (cannot span lines)
    let ref_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::RefMarker { .. }))
        .collect();

    assert_eq!(
        ref_tokens.len(),
        0,
        "Input '{}' should not produce REF_MARKER tokens when spanning lines, but got: {:?}",
        input,
        ref_tokens
    );
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_ref_marker_citation_properties(identifier in "[a-zA-Z][a-zA-Z0-9_-]{2,10}") {
        let input = format!("[@{}]", identifier);
        let tokens = tokenize(&input);

        // Should have exactly 1 CITATION_REF token + EOF
        prop_assert_eq!(tokens.len(), 2);

        match &tokens[0] {
            Token::CitationRef { content, span } => {
                prop_assert_eq!(content, &identifier); // Just the identifier, no @ prefix
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, input.len());
            }
            _ => prop_assert!(false, "Expected CitationRef token"),
        }
    }

    #[test]
    fn test_ref_marker_footnote_properties(number in 1u32..=999u32) {
        let input = format!("[{}]", number);
        let tokens = tokenize(&input);

        // Should have exactly 1 FOOTNOTE_REF token + EOF
        prop_assert_eq!(tokens.len(), 2);

        match &tokens[0] {
            Token::FootnoteRef { footnote_type, span } => {
                use txxt::tokenizer::inline::references::footnote_ref::FootnoteType;
                prop_assert_eq!(footnote_type, &FootnoteType::Naked(number));
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, input.len());
            }
            _ => prop_assert!(false, "Expected FootnoteRef token"),
        }
    }

    #[test]
    fn test_ref_marker_section_properties(
        major in 1u32..=99u32,
        minor in 1u32..=99u32
    ) {
        let input = format!("[#{}.{}]", major, minor);
        let tokens = tokenize(&input);

        // Should have exactly 1 SESSION_REF token + EOF
        prop_assert_eq!(tokens.len(), 2);

        match &tokens[0] {
            Token::SessionRef { content, span } => {
                let expected_content = format!("{}.{}", major, minor);
                prop_assert_eq!(content, &expected_content);
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, input.len());
            }
            _ => prop_assert!(false, "Expected SessionRef token"),
        }
    }

    #[test]
    fn test_ref_marker_span_consistency(
        ref_type in r"(#[1-9]|@[a-z]+|[a-z]+\.txt)"
    ) {
        let input = format!("[{}]", ref_type);
        let tokens = tokenize(&input);

        for token in &tokens {
            if let Token::RefMarker { content, span } = token {
                // Span should be consistent with input length
                prop_assert_eq!(
                    span.end.column - span.start.column,
                    input.len(),
                    "Span length should match input length"
                );

                // Content should match the reference type
                prop_assert_eq!(content, &ref_type);

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
        let tokens = tokenize("[#1]");
        assert!(!tokens.is_empty());
    }
}
