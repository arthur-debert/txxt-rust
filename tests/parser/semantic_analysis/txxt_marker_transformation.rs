#![allow(deprecated)]
//! Tests for TxxtMarker semantic token transformation
//!
//! This module tests the TxxtMarker transformation as implemented in Issue #81.
//! It verifies that TxxtMarker scanner tokens are correctly transformed into
//! TxxtMarker semantic tokens while preserving source span information.

use txxt::cst::high_level_tokens::{HighLevelToken, HighLevelTokenBuilder, HighLevelTokenSpan};
use txxt::cst::{Position, ScannerToken, SourceSpan};
use txxt::syntax::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

/// Test basic TxxtMarker transformation
#[test]
fn test_txxt_marker_basic_transformation() {
    let analyzer = SemanticAnalyzer::new();

    // Create a TxxtMarker scanner token
    let txxt_marker_token = ScannerToken::TxxtMarker {
        span: SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    };

    // Transform the token
    let result = analyzer.transform_txxt_marker(&txxt_marker_token);

    // Verify the transformation
    assert!(result.is_ok());
    let semantic_token = result.unwrap();

    match semantic_token {
        HighLevelToken::TxxtMarker { span, .. } => {
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 2);
        }
        _ => panic!(
            "Expected TxxtMarker semantic token, got {:?}",
            semantic_token
        ),
    }
}

/// Test TxxtMarker transformation with different source positions
#[test]
fn test_txxt_marker_different_positions() {
    let analyzer = SemanticAnalyzer::new();

    // Test TxxtMarker at different positions
    let test_cases = vec![
        (
            Position { row: 0, column: 0 },
            Position { row: 0, column: 2 },
        ),
        (
            Position { row: 5, column: 10 },
            Position { row: 5, column: 12 },
        ),
        (
            Position {
                row: 100,
                column: 50,
            },
            Position {
                row: 100,
                column: 52,
            },
        ),
    ];

    for (start, end) in test_cases {
        let txxt_marker_token = ScannerToken::TxxtMarker {
            span: SourceSpan { start, end },
        };

        let result = analyzer.transform_txxt_marker(&txxt_marker_token);
        assert!(result.is_ok());

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::TxxtMarker { span, .. } => {
                assert_eq!(span.start, start);
                assert_eq!(span.end, end);
            }
            _ => panic!("Expected TxxtMarker semantic token"),
        }
    }
}

/// Test TxxtMarker transformation with invalid token type
#[test]
fn test_txxt_marker_invalid_token_type() {
    let analyzer = SemanticAnalyzer::new();

    // Try to transform a non-TxxtMarker token
    let text_token = ScannerToken::Text {
        content: "hello".to_string(),
        span: SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 5 },
        },
    };

    let result = analyzer.transform_txxt_marker(&text_token);

    // Verify error handling
    assert!(result.is_err());
    match result.unwrap_err() {
        SemanticAnalysisError::InvalidTokenType { expected, actual } => {
            assert_eq!(expected, "TxxtMarker");
            assert!(actual.contains("Text"));
        }
        _ => panic!("Expected InvalidTokenType error"),
    }
}

/// Test TxxtMarker transformation in full semantic analysis
#[test]
fn test_txxt_marker_in_semantic_analysis() {
    let analyzer = SemanticAnalyzer::new();

    // Create a mix of tokens including TxxtMarker
    let scanner_tokens = vec![
        ScannerToken::Text {
            content: "Hello".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 6 },
                end: Position { row: 1, column: 8 },
            },
        },
        ScannerToken::Text {
            content: "world".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 9 },
                end: Position { row: 1, column: 14 },
            },
        },
        ScannerToken::BlankLine {
            whitespace: "".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
    ];

    // Run semantic analysis
    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 4);

    // Verify the TxxtMarker was transformed correctly
    let txxt_marker_token = &semantic_tokens.tokens[1];
    match txxt_marker_token {
        HighLevelToken::TxxtMarker { span, .. } => {
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 6);
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 8);
        }
        _ => panic!(
            "Expected TxxtMarker semantic token at position 1, got {:?}",
            txxt_marker_token
        ),
    }
}

/// Test TxxtMarker builder method
#[test]
fn test_txxt_marker_builder() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let txxt_marker = HighLevelTokenBuilder::txxt_marker(span.clone());

    match txxt_marker {
        HighLevelToken::TxxtMarker {
            span: marker_span, ..
        } => {
            assert_eq!(marker_span, span);
        }
        _ => panic!("Expected TxxtMarker semantic token"),
    }
}

/// Test TxxtMarker span trait implementation
#[test]
fn test_txxt_marker_span_trait() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let txxt_marker = HighLevelTokenBuilder::txxt_marker(span.clone());
    let retrieved_span = txxt_marker.span();

    assert_eq!(retrieved_span, &span);
}

/// Test multiple TxxtMarkers in sequence
#[test]
fn test_multiple_txxt_markers() {
    let analyzer = SemanticAnalyzer::new();

    // Create multiple TxxtMarker tokens
    let scanner_tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 3 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 2 },
            },
        },
    ];

    // Run semantic analysis
    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 3);

    // Verify all tokens are TxxtMarkers
    for (i, token) in semantic_tokens.tokens.iter().enumerate() {
        match token {
            HighLevelToken::TxxtMarker { span, .. } => {
                // Verify each marker has correct position
                match i {
                    0 => {
                        assert_eq!(span.start.row, 1);
                        assert_eq!(span.start.column, 0);
                        assert_eq!(span.end.column, 2);
                    }
                    1 => {
                        assert_eq!(span.start.row, 1);
                        assert_eq!(span.start.column, 3);
                        assert_eq!(span.end.column, 5);
                    }
                    2 => {
                        assert_eq!(span.start.row, 2);
                        assert_eq!(span.start.column, 0);
                        assert_eq!(span.end.column, 2);
                    }
                    _ => panic!("Unexpected token index"),
                }
            }
            _ => panic!(
                "Expected TxxtMarker semantic token at position {}, got {:?}",
                i, token
            ),
        }
    }
}

/// Test TxxtMarker with structural tokens
#[test]
fn test_txxt_marker_with_structural_tokens() {
    let analyzer = SemanticAnalyzer::new();

    // Create tokens including TxxtMarker and structural tokens
    let scanner_tokens = vec![
        ScannerToken::Indent {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 4 },
                end: Position { row: 1, column: 6 },
            },
        },
        ScannerToken::Dedent {
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
    ];

    // Run semantic analysis
    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 3);

    // Verify structural tokens are passed through unchanged
    match &semantic_tokens.tokens[0] {
        HighLevelToken::Indent { span, .. } => {
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 4);
        }
        _ => panic!("Expected Indent semantic token"),
    }

    // Verify TxxtMarker is transformed
    match &semantic_tokens.tokens[1] {
        HighLevelToken::TxxtMarker { span, .. } => {
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 4);
            assert_eq!(span.end.column, 6);
        }
        _ => panic!("Expected TxxtMarker semantic token"),
    }

    // Verify structural tokens are passed through unchanged
    match &semantic_tokens.tokens[2] {
        HighLevelToken::Dedent { span, .. } => {
            assert_eq!(span.start.row, 2);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 0);
        }
        _ => panic!("Expected Dedent semantic token"),
    }
}
