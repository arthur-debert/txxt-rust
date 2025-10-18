//! Tests for hierarchical numerical sequence markers
//!
//! These tests verify that the lexer correctly tokenizes hierarchical numerical
//! sequence markers like "1.1.", "1.2.3.", etc. as defined in the extended form
//! specification.

use txxt::ast::scanner_tokens::{SequenceMarkerType, ScannerToken};
use txxt::lexer::Lexer;

#[test]
fn test_hierarchical_numerical_sequence_markers() {
    let test_cases = vec![
        ("1.1. Item", 1, "1.1."),
        ("1.2. Item", 1, "1.2."),
        ("1.2.3. Item", 1, "1.2.3."),
        ("2.1. Item", 2, "2.1."),
        ("10.5. Item", 10, "10.5."),
        ("1.1.1.1. Item", 1, "1.1.1.1."),
    ];

    for (input, expected_number, expected_marker) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Find the sequence marker token
        let sequence_marker = tokens
            .iter()
            .find(|t| matches!(t, ScannerToken::SequenceMarker { .. }))
            .unwrap_or_else(|| panic!("Should find sequence marker for input: {}", input));

        match sequence_marker {
            ScannerToken::SequenceMarker { marker_type, .. } => match marker_type {
                SequenceMarkerType::Numerical(number, marker) => {
                    assert_eq!(
                        *number, expected_number,
                        "Expected number {} for input: {}",
                        expected_number, input
                    );
                    assert_eq!(
                        marker, expected_marker,
                        "Expected marker '{}' for input: {}",
                        expected_marker, input
                    );
                }
                _ => panic!("Expected Numerical marker type for input: {}", input),
            },
            _ => panic!("Expected SequenceMarker token for input: {}", input),
        }
    }
}

#[test]
fn test_hierarchical_vs_regular_numerical_markers() {
    // Test that regular numerical markers still work
    let regular_input = "1. Item";
    let mut lexer = Lexer::new(regular_input);
    let tokens = lexer.tokenize();

    let sequence_marker = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::SequenceMarker { .. }))
        .expect("Should find sequence marker for regular input");

    match sequence_marker {
        ScannerToken::SequenceMarker { marker_type, .. } => match marker_type {
            SequenceMarkerType::Numerical(number, marker) => {
                assert_eq!(*number, 1, "Expected number 1 for regular marker");
                assert_eq!(marker, "1.", "Expected marker '1.' for regular marker");
            }
            _ => panic!("Expected Numerical marker type for regular input"),
        },
        _ => panic!("Expected SequenceMarker token for regular input"),
    }
}

#[test]
fn test_hierarchical_marker_without_space_is_not_sequence_marker() {
    // "1.1.Item" should not be a sequence marker (no space after period)
    let input = "1.1.Item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Should NOT find sequence marker
    let has_marker = tokens
        .iter()
        .any(|t| matches!(t, ScannerToken::SequenceMarker { .. }));

    assert!(
        !has_marker,
        "Should NOT find sequence marker when there's no space after period"
    );

    // Should find text tokens instead
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t, ScannerToken::Text { .. }))
        .collect();

    assert!(!text_tokens.is_empty(), "Should find text tokens");
}

#[test]
fn test_hierarchical_marker_with_parenthesis_format() {
    // Test that hierarchical markers work with parenthesis format too
    let input = "1.1) Item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Should find sequence marker (parenthesis format IS supported for hierarchical)
    let sequence_marker = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::SequenceMarker { .. }))
        .expect("Should find sequence marker for hierarchical parenthesis format");

    match sequence_marker {
        ScannerToken::SequenceMarker { marker_type, .. } => match marker_type {
            SequenceMarkerType::Numerical(number, marker) => {
                assert_eq!(
                    *number, 1,
                    "Expected first number 1 for hierarchical parenthesis marker"
                );
                assert_eq!(marker, "1.1)", "Expected hierarchical parenthesis marker");
            }
            _ => panic!("Expected Numerical marker type for hierarchical parenthesis input"),
        },
        _ => panic!("Expected SequenceMarker token for hierarchical parenthesis input"),
    }
}

#[test]
fn test_complex_hierarchical_numbering() {
    // Test a complex hierarchical numbering pattern
    let input = "1.2.3.4.5. Item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let sequence_marker = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::SequenceMarker { .. }))
        .expect("Should find sequence marker for complex hierarchical input");

    match sequence_marker {
        ScannerToken::SequenceMarker { marker_type, .. } => match marker_type {
            SequenceMarkerType::Numerical(number, marker) => {
                assert_eq!(
                    *number, 1,
                    "Expected first number 1 for complex hierarchical marker"
                );
                assert_eq!(marker, "1.2.3.4.5.", "Expected full hierarchical marker");
            }
            _ => panic!("Expected Numerical marker type for complex hierarchical input"),
        },
        _ => panic!("Expected SequenceMarker token for complex hierarchical input"),
    }
}
