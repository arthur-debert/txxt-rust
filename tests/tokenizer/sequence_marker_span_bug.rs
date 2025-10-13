//! Tests demonstrating sequence marker span calculation bugs
//!
//! These tests show that sequence markers calculate spans incorrectly
//! by adding string length to column position, which fails for multi-byte characters.

use txxt::ast::tokens::{SequenceMarkerType, Token};
use txxt::tokenizer::Lexer;

#[test]
fn test_sequence_marker_span_with_unicode() {
    // According to txxt spec, sequence markers MUST be at line start.
    // "🎉- item" does NOT produce a sequence marker, only a dash token.
    let input = "🎉- item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Should NOT find sequence marker
    let has_marker = tokens
        .iter()
        .any(|t| matches!(t, Token::SequenceMarker { .. }));

    assert!(
        !has_marker,
        "Should NOT find sequence marker when dash is not at column 0"
    );

    // Should find dash token instead
    let dash = tokens
        .iter()
        .find(|t| matches!(t, Token::Dash { .. }))
        .expect("Should find dash token");

    match dash {
        Token::Dash { span } => {
            // The emoji takes 1 character position
            assert_eq!(
                span.start.column, 1,
                "Dash should start at column 1 after emoji"
            );
            assert_eq!(span.end.column, 2, "Dash should end at column 2");
        }
        _ => unreachable!(),
    }

    // Test actual sequence marker at line start with Unicode content
    let valid_input = "- 🎉item";
    let mut lexer = Lexer::new(valid_input);
    let tokens = lexer.tokenize();

    let marker = tokens
        .iter()
        .find(|t| matches!(t, Token::SequenceMarker { .. }))
        .expect("Should find sequence marker at column 0");

    match marker {
        Token::SequenceMarker { span, .. } => {
            assert_eq!(span.start.column, 0, "Sequence marker at line start");
            assert_eq!(span.end.column, 1, "Plain dash marker is 1 char");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_sequence_marker_span_calculation() {
    // Test various sequence markers
    let test_cases = vec![
        ("- item", 0, 1, "plain marker"),
        ("1. item", 0, 2, "single digit numerical"),
        ("42. item", 0, 3, "double digit numerical"),
        ("a. item", 0, 2, "alphabetical"),
        ("iv. item", 0, 3, "roman numeral"),
    ];

    for (input, expected_start, expected_end, description) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let marker = tokens
            .iter()
            .find(|t| matches!(t, Token::SequenceMarker { .. }))
            .unwrap_or_else(|| panic!("Should find sequence marker for {}", description));

        match marker {
            Token::SequenceMarker { span, .. } => {
                assert_eq!(
                    span.start.column, expected_start,
                    "{}: incorrect start column",
                    description
                );
                assert_eq!(
                    span.end.column, expected_end,
                    "{}: incorrect end column (calculated as {})",
                    description, span.end.column
                );
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_sequence_marker_correct_type_detection() {
    let test_cases = vec![
        ("- ", SequenceMarkerType::Plain("-".to_string())),
        ("1. ", SequenceMarkerType::Numerical(1, "1.".to_string())),
        (
            "a. ",
            SequenceMarkerType::Alphabetical('a', "a.".to_string()),
        ),
        ("i. ", SequenceMarkerType::Roman(1, "i.".to_string())),
    ];

    for (input, expected_type) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let marker = tokens
            .iter()
            .find(|t| matches!(t, Token::SequenceMarker { .. }))
            .unwrap_or_else(|| panic!("Should find sequence marker in '{}'", input));

        match marker {
            Token::SequenceMarker { marker_type, .. } => match (marker_type, &expected_type) {
                (SequenceMarkerType::Plain(a), SequenceMarkerType::Plain(b)) => {
                    assert_eq!(a, b, "Plain marker mismatch");
                }
                (SequenceMarkerType::Numerical(n1, s1), SequenceMarkerType::Numerical(n2, s2)) => {
                    assert_eq!(n1, n2, "Numerical value mismatch");
                    assert_eq!(s1, s2, "Numerical string mismatch");
                }
                (
                    SequenceMarkerType::Alphabetical(c1, s1),
                    SequenceMarkerType::Alphabetical(c2, s2),
                ) => {
                    assert_eq!(c1, c2, "Alphabetical char mismatch");
                    assert_eq!(s1, s2, "Alphabetical string mismatch");
                }
                (SequenceMarkerType::Roman(n1, s1), SequenceMarkerType::Roman(n2, s2)) => {
                    assert_eq!(n1, n2, "Roman value mismatch");
                    assert_eq!(s1, s2, "Roman string mismatch");
                }
                _ => panic!(
                    "Type mismatch: got {:?}, expected {:?}",
                    marker_type, expected_type
                ),
            },
            _ => unreachable!(),
        }
    }
}
