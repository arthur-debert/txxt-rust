//! Tests for DefinitionMarker token recognition using rstest and proptest
//!
//! Tests both successful parsing and failure cases for definition markers.
//! Definition markers are :: at the end of lines (term ::) as opposed to
//! annotation markers which are :: label :: patterns.

use proptest::prelude::*;
use rstest::rstest;
use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::tokenize;

// =============================================================================
// DefinitionMarker Token - Isolated Tests (rstest)
// =============================================================================

#[rstest]
#[case("term ::")]
#[case("definition ::")]
#[case("concept ::")]
fn test_definition_marker_isolated_passing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should have: TEXT, DEFINITION_MARKER, EOF
    assert!(tokens.len() >= 3, "Should have at least 3 tokens");

    // Find the definition marker
    let definition_marker = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
        .expect("Should find DefinitionMarker token");

    match definition_marker {
        ScannerToken::DefinitionMarker { content, span } => {
            assert_eq!(content, "::");
            assert!(span.start.column > 0); // Should not be at start of line
        }
        _ => unreachable!(),
    }

    // Should also have text before the definition marker
    let text_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { .. }))
        .expect("Should find Text token");

    assert!(matches!(text_token, ScannerToken::Text { .. }));
}

#[rstest]
#[case("term::\n", "term")]
#[case("definition::\n", "definition")]
#[case("my concept::\n", "my")]
fn test_definition_marker_with_newline(#[case] input: &str, #[case] expected_text: &str) {
    let tokens = tokenize(input);

    // Find the text token
    let text_token = tokens
        .iter()
        .find(
            |token| matches!(token, ScannerToken::Text { content, .. } if content == expected_text),
        )
        .expect("Should find expected text token");

    match text_token {
        ScannerToken::Text { content, .. } => {
            assert_eq!(content, expected_text);
        }
        _ => unreachable!(),
    }

    // Find the definition marker
    let definition_marker = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
        .expect("Should find DefinitionMarker token");

    match definition_marker {
        ScannerToken::DefinitionMarker { content, .. } => {
            assert_eq!(content, "::");
        }
        _ => unreachable!(),
    }

    // Should also have a newline token
    let newline_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Newline { .. }))
        .expect("Should find Newline token");

    assert!(matches!(newline_token, ScannerToken::Newline { .. }));
}

#[rstest]
#[case("term :: content after", "term", "content")]
#[case("definition :: more text", "definition", "more")]
fn test_definition_marker_with_content_after(
    #[case] input: &str,
    #[case] first_text: &str,
    #[case] second_text: &str,
) {
    let tokens = tokenize(input);

    // Find first text token
    let first_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { content, .. } if content == first_text))
        .expect("Should find first text token");

    assert!(matches!(first_token, ScannerToken::Text { .. }));

    // Find definition marker
    let definition_marker = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
        .expect("Should find DefinitionMarker token");

    assert!(matches!(
        definition_marker,
        ScannerToken::DefinitionMarker { .. }
    ));

    // Find second text token
    let second_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { content, .. } if content == second_text))
        .expect("Should find second text token");

    assert!(matches!(second_token, ScannerToken::Text { .. }));
}

// =============================================================================
// DefinitionMarker Token - Failing Cases (rstest)
// =============================================================================

#[rstest]
#[case(":: label ::")] // Annotation marker, not definition
#[case(":: title ::")] // Annotation marker, not definition
#[case(":: content ::")] // Annotation marker, not definition
fn test_definition_marker_failing_annotation_patterns(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should NOT contain any DEFINITION_MARKER tokens
    let definition_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
        .collect();

    assert_eq!(
        definition_tokens.len(),
        0,
        "Input '{}' should not produce DEFINITION_MARKER tokens, but got: {:?}",
        input,
        definition_tokens
    );

    // Should contain annotation markers instead
    let annotation_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::AnnotationMarker { .. }))
        .collect();

    assert!(
        !annotation_tokens.is_empty(),
        "Input '{}' should produce ANNOTATION_MARKER tokens",
        input
    );
}

#[rstest]
#[case("::")] // Just :: alone without term
#[case(":::")] // Triple colon
#[case("::::")] // Quad colon
fn test_definition_marker_failing_invalid_patterns(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should NOT contain any DEFINITION_MARKER tokens
    let definition_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
        .collect();

    assert_eq!(
        definition_tokens.len(),
        0,
        "Input '{}' should not produce DEFINITION_MARKER tokens, but got: {:?}",
        input,
        definition_tokens
    );
}

#[rstest]
#[case("text")] // No double colon
#[case("term:")] // Single colon
#[case("normal text")] // Regular text
fn test_definition_marker_failing_no_marker(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should NOT contain any DEFINITION_MARKER tokens
    let definition_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
        .collect();

    assert_eq!(
        definition_tokens.len(),
        0,
        "Input '{}' should not produce DEFINITION_MARKER tokens",
        input
    );
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_definition_marker_properties(term in "[a-zA-Z][a-zA-Z0-9 ]{1,20}") {
        let input = format!("{} ::", term.trim());
        let tokens = tokenize(&input);

        // Should have at least one DEFINITION_MARKER token
        let definition_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
            .collect();

        prop_assert_eq!(definition_tokens.len(), 1, "Should produce exactly 1 DEFINITION_MARKER token");

        // Should have at least one TEXT token for the term
        let text_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::Text { .. }))
            .collect();

        prop_assert!(!text_tokens.is_empty(), "Should produce at least 1 TEXT token");
    }

    #[test]
    fn test_definition_marker_span_consistency(term in "[a-zA-Z]+") {
        let input = format!("{} ::", term);
        let tokens = tokenize(&input);

        for token in &tokens {
            if let ScannerToken::DefinitionMarker { content, span } = token {
                // Span should be consistent with content length
                prop_assert_eq!(
                    span.end.column - span.start.column,
                    content.len(),
                    "Span length should match content length"
                );

                // Start should come before end
                prop_assert!(span.start.column <= span.end.column);
                prop_assert!(span.start.row <= span.end.row);

                // Content should always be "::"
                prop_assert_eq!(content, "::");
            }
        }
    }

    #[test]
    fn test_definition_vs_annotation_distinction(label in "[a-zA-Z]+") {
        // Test definition pattern
        let def_input = format!("{} ::", label);
        let def_tokens = tokenize(&def_input);

        let def_markers: Vec<_> = def_tokens.iter()
            .filter(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
            .collect();

        prop_assert_eq!(def_markers.len(), 1, "Definition pattern should produce DEFINITION_MARKER");

        // Test annotation pattern
        let ann_input = format!(":: {} ::", label);
        let ann_tokens = tokenize(&ann_input);

        let ann_markers: Vec<_> = ann_tokens.iter()
            .filter(|token| matches!(token, ScannerToken::AnnotationMarker { .. }))
            .collect();

        prop_assert!(ann_markers.len() >= 2, "Annotation pattern should produce ANNOTATION_MARKERs");

        let ann_def_markers: Vec<_> = ann_tokens.iter()
            .filter(|token| matches!(token, ScannerToken::DefinitionMarker { .. }))
            .collect();

        prop_assert_eq!(ann_def_markers.len(), 0, "Annotation pattern should not produce DEFINITION_MARKERs");
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("term ::");
        assert!(!tokens.is_empty());
    }
}
