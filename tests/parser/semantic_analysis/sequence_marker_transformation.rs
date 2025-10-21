//! Tests for Sequence Marker semantic token transformation
//!
//! This module tests the transformation of SequenceMarker scanner tokens into
//! SequenceMarker semantic tokens as specified in Issue #84.

use txxt::cst::high_level_tokens::{
    HighLevelNumberingForm, HighLevelNumberingStyle, HighLevelToken, HighLevelTokenBuilder,
    HighLevelTokenSpan,
};
use txxt::cst::{Position, ScannerToken, SequenceMarkerType, SourceSpan};
use txxt::syntax::semantic_analysis::SemanticAnalyzer;

#[test]
fn test_sequence_marker_plain_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let marker_type = SequenceMarkerType::Plain("-".to_string());
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 1 },
    };

    let result = analyzer.transform_sequence_marker(marker_type, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span: token_span,
        } => {
            assert_eq!(style, HighLevelNumberingStyle::Plain);
            assert_eq!(form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "-");
            assert_eq!(token_span, span);
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_marker_numerical_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let marker_type = SequenceMarkerType::Numerical(1, "1.".to_string());
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let result = analyzer.transform_sequence_marker(marker_type, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span: token_span,
        } => {
            assert_eq!(style, HighLevelNumberingStyle::Numeric);
            assert_eq!(form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "1.");
            assert_eq!(token_span, span);
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_marker_alphabetical_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let marker_type = SequenceMarkerType::Alphabetical('a', "a.".to_string());
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let result = analyzer.transform_sequence_marker(marker_type, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span: token_span,
        } => {
            assert_eq!(style, HighLevelNumberingStyle::Alphabetic);
            assert_eq!(form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "a.");
            assert_eq!(token_span, span);
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_marker_roman_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let marker_type = SequenceMarkerType::Roman(1, "i.".to_string());
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let result = analyzer.transform_sequence_marker(marker_type, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span: token_span,
        } => {
            assert_eq!(style, HighLevelNumberingStyle::Roman);
            assert_eq!(form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "i.");
            assert_eq!(token_span, span);
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_marker_different_markers() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        (
            SequenceMarkerType::Plain("*".to_string()),
            HighLevelNumberingStyle::Plain,
        ),
        (
            SequenceMarkerType::Numerical(42, "42)".to_string()),
            HighLevelNumberingStyle::Numeric,
        ),
        (
            SequenceMarkerType::Alphabetical('Z', "Z)".to_string()),
            HighLevelNumberingStyle::Alphabetic,
        ),
        (
            SequenceMarkerType::Roman(5, "v)".to_string()),
            HighLevelNumberingStyle::Roman,
        ),
    ];

    for (marker_type, expected_style) in test_cases.iter() {
        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 3 },
        };

        let result = analyzer.transform_sequence_marker(marker_type.clone(), span.clone());
        assert!(result.is_ok());

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::SequenceMarker {
                style,
                form,
                marker,
                span: token_span,
            } => {
                assert_eq!(style, *expected_style);
                assert_eq!(form, HighLevelNumberingForm::Regular);
                assert_eq!(marker, marker_type.content());
                assert_eq!(token_span, span);
            }
            _ => panic!(
                "Expected SequenceMarker semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_sequence_marker_different_positions() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        (
            Position { row: 1, column: 0 },
            Position { row: 1, column: 2 },
        ),
        (
            Position { row: 2, column: 4 },
            Position { row: 2, column: 6 },
        ),
        (
            Position { row: 5, column: 8 },
            Position { row: 5, column: 10 },
        ),
    ];

    for (start, end) in test_cases.iter() {
        let marker_type = SequenceMarkerType::Numerical(1, "1.".to_string());
        let span = SourceSpan {
            start: *start,
            end: *end,
        };

        let result = analyzer.transform_sequence_marker(marker_type, span.clone());
        assert!(result.is_ok());

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::SequenceMarker {
                span: token_span, ..
            } => {
                assert_eq!(token_span, span);
            }
            _ => panic!(
                "Expected SequenceMarker semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_sequence_marker_in_semantic_analysis() {
    let analyzer = SemanticAnalyzer::new();

    let scanner_tokens = vec![
        ScannerToken::SequenceMarker {
            marker_type: SequenceMarkerType::Numerical(1, "1.".to_string()),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::BlankLine {
            whitespace: "".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::SequenceMarker {
            marker_type: SequenceMarkerType::Plain("-".to_string()),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 3, column: 1 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 3);

    // Check first sequence marker
    match &semantic_tokens.tokens[0] {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span,
        } => {
            assert_eq!(*style, HighLevelNumberingStyle::Numeric);
            assert_eq!(*form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "1.");
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 2);
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_tokens.tokens[0]
        ),
    }

    // Check blank line
    match &semantic_tokens.tokens[1] {
        HighLevelToken::BlankLine { span } => {
            assert_eq!(span.start.row, 2);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 2);
            assert_eq!(span.end.column, 0);
        }
        _ => panic!(
            "Expected BlankLine semantic token, got {:?}",
            semantic_tokens.tokens[1]
        ),
    }

    // Check second sequence marker
    match &semantic_tokens.tokens[2] {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span,
        } => {
            assert_eq!(*style, HighLevelNumberingStyle::Plain);
            assert_eq!(*form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "-");
            assert_eq!(span.start.row, 3);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 3);
            assert_eq!(span.end.column, 1);
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_tokens.tokens[2]
        ),
    }
}

#[test]
fn test_sequence_marker_builder() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let semantic_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        span.clone(),
    );

    match semantic_token {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span: token_span,
        } => {
            assert_eq!(style, HighLevelNumberingStyle::Numeric);
            assert_eq!(form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "1.");
            assert_eq!(token_span, span);
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_marker_span_trait() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let semantic_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        span.clone(),
    );
    let token_span = semantic_token.span();

    assert_eq!(token_span, &span);
}

#[test]
fn test_sequence_marker_with_structural_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let scanner_tokens = vec![
        ScannerToken::Indent {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::SequenceMarker {
            marker_type: SequenceMarkerType::Numerical(1, "1.".to_string()),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::Dedent {
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 3);

    // Check that structural tokens are preserved
    match &semantic_tokens.tokens[0] {
        HighLevelToken::Indent { .. } => {} // OK
        _ => panic!(
            "Expected Indent semantic token, got {:?}",
            semantic_tokens.tokens[0]
        ),
    }

    // Check that sequence marker is created
    match &semantic_tokens.tokens[1] {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            ..
        } => {
            assert_eq!(*style, HighLevelNumberingStyle::Numeric);
            assert_eq!(*form, HighLevelNumberingForm::Regular);
            assert_eq!(marker, "1.");
        }
        _ => panic!(
            "Expected SequenceMarker semantic token, got {:?}",
            semantic_tokens.tokens[1]
        ),
    }

    // Check that dedent is preserved
    match &semantic_tokens.tokens[2] {
        HighLevelToken::Dedent { .. } => {} // OK
        _ => panic!(
            "Expected Dedent semantic token, got {:?}",
            semantic_tokens.tokens[2]
        ),
    }
}

#[test]
fn test_sequence_marker_classification_helper() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        (
            SequenceMarkerType::Plain("-".to_string()),
            (
                HighLevelNumberingStyle::Plain,
                HighLevelNumberingForm::Regular,
            ),
        ),
        (
            SequenceMarkerType::Numerical(1, "1.".to_string()),
            (
                HighLevelNumberingStyle::Numeric,
                HighLevelNumberingForm::Regular,
            ),
        ),
        (
            SequenceMarkerType::Alphabetical('a', "a.".to_string()),
            (
                HighLevelNumberingStyle::Alphabetic,
                HighLevelNumberingForm::Regular,
            ),
        ),
        (
            SequenceMarkerType::Roman(1, "i.".to_string()),
            (
                HighLevelNumberingStyle::Roman,
                HighLevelNumberingForm::Regular,
            ),
        ),
    ];

    for (marker_type, expected) in test_cases.iter() {
        let result = analyzer.classify_sequence_marker(marker_type);
        assert_eq!(result, *expected);
    }
}
