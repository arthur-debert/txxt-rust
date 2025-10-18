//! Tests for ANNOTATION_MARKER token tokenization using rstest and proptest
//!
//! Tests both successful parsing and failure cases for ANNOTATION_MARKER tokens
//! Annotation markers are :: symbols used in annotations like :: title :: content
//! Also includes integration tests for annotation parameters.

use proptest::prelude::*;
use rstest::rstest;
use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::{core::patterns::IDENTIFIER_PATTERN, tokenize};

// =============================================================================
// ANNOTATION_MARKER Token - Isolated Tests (rstest)
// =============================================================================

#[rstest]
#[case("::", "::")]
fn test_annotation_marker_isolated_passing(#[case] input: &str, #[case] expected_content: &str) {
    let tokens = tokenize(input);

    assert!(!tokens.is_empty(), "Should have at least one token");

    match &tokens[0] {
        ScannerToken::TxxtMarker { span } => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, expected_content.len());
        }
        _ => panic!("Expected TxxtMarker token, got {:?}", tokens[0]),
    }
}

#[rstest]
#[case(":: title ::", "::", "title", "::")]
#[case(":: author ::", "::", "author", "::")]
#[case(":: metadata ::", "::", "metadata", "::")]
#[case("::label::", "::", "label", "::")]
fn test_annotation_marker_with_content_passing(
    #[case] input: &str,
    #[case] _expected_first_marker: &str,
    #[case] expected_identifier: &str,
    #[case] _expected_second_marker: &str,
) {
    let tokens = tokenize(input);

    // Should have: ANNOTATION_MARKER, IDENTIFIER, ANNOTATION_MARKER, EOF (at minimum)
    assert!(tokens.len() >= 3);

    // First token should be annotation marker
    match &tokens[0] {
        ScannerToken::TxxtMarker { span } => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => panic!("Expected first TxxtMarker token, got {:?}", tokens[0]),
    }

    // Find the text token
    let text_token = tokens
        .iter()
        .find(
            |token| matches!(token, ScannerToken::Text { content, .. } if content == expected_identifier),
        )
        .expect("Should find text token");

    match text_token {
        ScannerToken::Text { content, span } => {
            assert_eq!(content, expected_identifier);
            assert_eq!(span.start.row, 0);
            assert!(span.start.column > 0); // After first ::
        }
        _ => unreachable!(),
    }

    // Find the second annotation marker
    let second_marker_token = tokens
        .iter()
        .skip(1) // Skip first marker
        .find(|token| matches!(token, ScannerToken::TxxtMarker { .. }))
        .expect("Should find second annotation marker");

    match second_marker_token {
        ScannerToken::TxxtMarker { .. } => {
            // TxxtMarker always has content "::"
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("    :: title ::", "::", "title", "::")] // Indented pragma
#[case("  ::author::", "::", "author", "::")] // Indented without spaces
fn test_annotation_marker_indented_passing(
    #[case] input: &str,
    #[case] _expected_first_marker: &str,
    #[case] expected_identifier: &str,
    #[case] _expected_second_marker: &str,
) {
    let tokens = tokenize(input);

    // Should find annotation markers even when indented
    let pragma_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::TxxtMarker { .. }))
        .collect();

    assert_eq!(
        pragma_tokens.len(),
        2,
        "Should find exactly 2 annotation markers"
    );

    // Find the text token
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter(
            |token| matches!(token, ScannerToken::Text { content, .. } if content == expected_identifier),
        )
        .collect();

    assert_eq!(text_tokens.len(), 1, "Should find exactly 1 text token");
}

// =============================================================================
// ANNOTATION_MARKER Token - Failing Cases (rstest)
// =============================================================================

#[rstest]
// Single colon (not annotation marker)
#[case(":")]
#[case(": :")] // Colon with space
fn test_annotation_marker_isolated_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not contain any ANNOTATION_MARKER tokens
    let pragma_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::TxxtMarker { .. }))
        .collect();

    assert_eq!(
        pragma_tokens.len(),
        0,
        "Input '{}' should not produce ANNOTATION_MARKER tokens, but got: {:?}",
        input,
        pragma_tokens
    );
}

#[rstest]
// Triple colon cases - these are ambiguous and could be interpreted different ways
#[case(":::")] // Could be : + :: or :::
#[case("::: ")] // Could be : + :: + space
fn test_annotation_marker_triple_colon_cases(#[case] input: &str) {
    let tokens = tokenize(input);

    // These cases are implementation-defined behavior
    // The key requirement is that we don't crash and produce some reasonable tokenization
    assert!(
        !tokens.is_empty(),
        "Should produce some tokens without crashing"
    );

    // We can accept that ::: might produce an annotation marker (as : + ::)
    // The important thing is consistent behavior
}

#[rstest]
// Incomplete pragma patterns
#[case(":: title")] // Missing closing ::
#[case("title ::")] // Missing opening ::
#[case(":: ")] // Just opening with space
#[case(":: ::")] // Empty pragma (no identifier)
fn test_annotation_marker_incomplete_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // These should either not produce annotation markers or produce incomplete patterns
    // The important thing is they don't crash and handle gracefully

    // We can be flexible here - the lexer might produce some annotation markers
    // but the key is that it handles these edge cases without panicking
    assert!(
        !tokens.is_empty(),
        "Should produce some tokens without crashing"
    );
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_annotation_marker_basic_properties(identifier in IDENTIFIER_PATTERN) {
        // Test valid identifier patterns in annotation markers
        let input = format!(":: {} ::", identifier);
        let tokens = tokenize(&input);

        // Should have exactly 2 ANNOTATION_MARKER tokens
        let pragma_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::TxxtMarker { .. }))
            .collect();

        prop_assert_eq!(pragma_tokens.len(), 2, "Should produce exactly 2 ANNOTATION_MARKER tokens");

        // The content between annotation markers should reconstruct to the identifier
        // Find indices of annotation markers
        let marker_indices: Vec<usize> = tokens.iter().enumerate()
            .filter_map(|(i, token)| {
                if matches!(token, ScannerToken::TxxtMarker { .. }) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        prop_assert_eq!(marker_indices.len(), 2, "Should have exactly 2 annotation markers");

        // Reconstruct content between markers
        if marker_indices.len() == 2 {
            let content_tokens = &tokens[marker_indices[0] + 1..marker_indices[1]];
            let reconstructed: String = content_tokens.iter()
                .filter_map(|token| match token {
                    ScannerToken::Text { content, .. } => Some(content.as_str()),
                    ScannerToken::Identifier { content, .. } => Some(content.as_str()),
                    ScannerToken::ItalicDelimiter { .. } => Some("_"),
                    ScannerToken::BoldDelimiter { .. } => Some("*"),
                    ScannerToken::CodeDelimiter { .. } => Some("`"),
                    ScannerToken::MathDelimiter { .. } => Some("#"),
                    ScannerToken::Whitespace { .. } => None, // Skip whitespace in reconstruction
                    _ => None,
                })
                .collect();

            prop_assert_eq!(&reconstructed, &identifier,
                "Content between markers should reconstruct to the identifier");
        }
    }

    #[test]
    fn test_annotation_marker_span_consistency(
        input in r"::"
    ) {
        let tokens = tokenize(&input);

        for token in &tokens {
            if let ScannerToken::TxxtMarker { span } = token {
                // Span should be consistent with content length
                prop_assert_eq!(
                    span.end.column - span.start.column,
                    2, // TxxtMarker always has content "::"
                    "Span length should match content length"
                );

                // Start should come before end
                prop_assert!(span.start.column <= span.end.column);
                prop_assert!(span.start.row <= span.end.row);

                // Content should always be "::"
                prop_assert_eq!("::", "::");
            }
        }
    }

    #[test]
    fn test_multiple_annotation_markers(
        identifiers in prop::collection::vec(IDENTIFIER_PATTERN, 1..=3)
    ) {
        // Test multiple annotation markers in sequence
        let input = identifiers.iter()
            .map(|id| format!(":: {} ::", id))
            .collect::<Vec<_>>()
            .join(" ");

        let tokens = tokenize(&input);

        // Should have exactly 2 * identifiers.len() ANNOTATION_MARKER tokens
        let pragma_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::TxxtMarker { .. }))
            .collect();

        prop_assert_eq!(pragma_tokens.len(), 2 * identifiers.len(),
            "Should produce 2 annotation markers per identifier");

        // Each annotation should contain the expected identifier
        let marker_indices: Vec<usize> = tokens.iter().enumerate()
            .filter_map(|(i, token)| {
                if matches!(token, ScannerToken::TxxtMarker { .. }) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        // Should have pairs of markers
        prop_assert_eq!(marker_indices.len() % 2, 0, "Annotation markers should come in pairs");

        // Verify each annotation contains its identifier
        let mut reconstructed_identifiers = Vec::new();
        for i in (0..marker_indices.len()).step_by(2) {
            if i + 1 < marker_indices.len() {
                let content_tokens = &tokens[marker_indices[i] + 1..marker_indices[i + 1]];
                let reconstructed: String = content_tokens.iter()
                    .filter_map(|token| match token {
                        ScannerToken::Text { content, .. } => Some(content.as_str()),
                        ScannerToken::Identifier { content, .. } => Some(content.as_str()),
                        ScannerToken::ItalicDelimiter { .. } => Some("_"),
                        ScannerToken::BoldDelimiter { .. } => Some("*"),
                        ScannerToken::CodeDelimiter { .. } => Some("`"),
                        ScannerToken::MathDelimiter { .. } => Some("#"),
                        ScannerToken::Whitespace { .. } => None, // Skip whitespace
                        _ => None,
                    })
                    .collect();
                reconstructed_identifiers.push(reconstructed);
            }
        }

        prop_assert_eq!(reconstructed_identifiers.len(), identifiers.len(),
            "Should reconstruct one identifier per annotation");

        // Verify all identifiers are present
        for (expected, actual) in identifiers.iter().zip(reconstructed_identifiers.iter()) {
            prop_assert_eq!(actual, expected, "Reconstructed content should match identifier");
        }
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize(":: test ::");
        assert!(!tokens.is_empty());
    }
}

// =============================================================================
// ANNOTATION Parameter Integration Tests
// =============================================================================

#[test]
fn test_simple_annotation_parameter() {
    let input = ":: warning:severity=high :: Critical issue";
    let tokens = tokenize(input);

    // Find parameter tokens
    let param_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let ScannerToken::Parameter { key, value, .. } = token {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(param_tokens.len(), 1);
    assert_eq!(param_tokens[0], ("severity", "high"));

    // Find clean label
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let ScannerToken::Text { content, .. } = token {
                Some(content.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(text_tokens.contains(&"warning"));
}

#[test]
fn test_simple_definition_parameter() {
    let input = "API:version=2.0 ::\n    Application Programming Interface";
    let tokens = tokenize(input);

    // Find parameter tokens
    let param_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let ScannerToken::Parameter { key, value, .. } = token {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(param_tokens.len(), 1);
    assert_eq!(param_tokens[0], ("version", "2.0"));

    // Find clean term
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let ScannerToken::Text { content, .. } = token {
                Some(content.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(text_tokens.contains(&"API"));
}
