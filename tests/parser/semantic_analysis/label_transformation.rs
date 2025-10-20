//! Tests for Label semantic token transformation
//!
//! This module tests the Label transformation as implemented in Issue #82.
//! It verifies that Identifier scanner tokens are correctly transformed into
//! Label semantic tokens while preserving source span information and validating
//! label content according to the specification.

use txxt::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};
use txxt::ast::tokens::semantic::{SemanticToken, SemanticTokenBuilder, SemanticTokenSpan};
use txxt::parser::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

/// Test basic Label transformation
#[test]
fn test_label_basic_transformation() {
    let analyzer = SemanticAnalyzer::new();

    // Create an Identifier scanner token
    let identifier_token = ScannerToken::Identifier {
        content: "python".to_string(),
        span: SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 6 },
        },
    };

    // Transform the token
    let result = analyzer.transform_label("python".to_string(), identifier_token.span().clone());

    // Verify the transformation
    assert!(result.is_ok());
    let semantic_token = result.unwrap();

    match semantic_token {
        SemanticToken::Label { text, span } => {
            assert_eq!(text, "python");
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 6);
        }
        _ => panic!("Expected Label semantic token, got {:?}", semantic_token),
    }
}

/// Test Label transformation with namespaced identifiers
#[test]
fn test_label_namespaced_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        "org.example.custom",
        "com.company.product",
        "io.github.project",
        "a.b.c.d.e",
    ];

    for label_text in test_cases.iter() {
        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: label_text.len(),
            },
        };

        let result = analyzer.transform_label(label_text.to_string(), span.clone());
        assert!(result.is_ok(), "Failed for label: {}", label_text);

        let semantic_token = result.unwrap();
        match semantic_token {
            SemanticToken::Label {
                text,
                span: token_span,
            } => {
                assert_eq!(text, *label_text);
                assert_eq!(token_span, span);
            }
            _ => panic!("Expected Label semantic token for {}", label_text),
        }
    }
}

/// Test Label transformation with different valid characters
#[test]
fn test_label_valid_characters() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = vec![
        "python",
        "my-label",
        "my_label",
        "label123",
        "Label123",
        "my-label_123",
        "org.example-label",
    ];

    for label_text in test_cases {
        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: label_text.len(),
            },
        };

        let result = analyzer.transform_label(label_text.to_string(), span.clone());
        assert!(result.is_ok(), "Failed for valid label: {}", label_text);

        let semantic_token = result.unwrap();
        match semantic_token {
            SemanticToken::Label { text, .. } => {
                assert_eq!(text, label_text);
            }
            _ => panic!("Expected Label semantic token for {}", label_text),
        }
    }
}

/// Test Label transformation with invalid characters
#[test]
fn test_label_invalid_characters() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = vec![
        ("123invalid", "Label must start with a letter"),
        ("invalid@symbol", "Invalid character '@'"),
        ("invalid space", "Invalid character ' '"),
        ("invalid!symbol", "Invalid character '!'"),
        ("invalid#symbol", "Invalid character '#'"),
    ];

    for (label_text, expected_error) in test_cases {
        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: label_text.len(),
            },
        };

        let result = analyzer.transform_label(label_text.to_string(), span);
        assert!(
            result.is_err(),
            "Should fail for invalid label: {}",
            label_text
        );

        match result.unwrap_err() {
            SemanticAnalysisError::AnalysisError(msg) => {
                assert!(
                    msg.contains(expected_error),
                    "Expected error containing '{}', got '{}'",
                    expected_error,
                    msg
                );
            }
            _ => panic!("Expected AnalysisError for {}", label_text),
        }
    }
}

/// Test Label transformation with empty content
#[test]
fn test_label_empty_content() {
    let analyzer = SemanticAnalyzer::new();

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 0 },
    };

    let result = analyzer.transform_label("".to_string(), span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("cannot be empty"));
        }
        _ => panic!("Expected AnalysisError for empty label"),
    }
}

/// Test Label transformation in full semantic analysis
#[test]
fn test_label_in_semantic_analysis() {
    let analyzer = SemanticAnalyzer::new();

    // Create a mix of tokens including Identifier
    let scanner_tokens = vec![
        ScannerToken::Text {
            content: "Hello".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Identifier {
            content: "python".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 6 },
                end: Position { row: 1, column: 12 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 13 },
                end: Position { row: 1, column: 15 },
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

    // Verify the Identifier was transformed to Label
    let label_token = &semantic_tokens.tokens[1];
    match label_token {
        SemanticToken::Label { text, span } => {
            assert_eq!(text, "python");
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 6);
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 12);
        }
        _ => panic!(
            "Expected Label semantic token at position 1, got {:?}",
            label_token
        ),
    }
}

/// Test Label builder method
#[test]
fn test_label_builder() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 6 },
    };

    let label = SemanticTokenBuilder::label("python".to_string(), span.clone());

    match label {
        SemanticToken::Label {
            text,
            span: label_span,
        } => {
            assert_eq!(text, "python");
            assert_eq!(label_span, span);
        }
        _ => panic!("Expected Label semantic token"),
    }
}

/// Test Label span trait implementation
#[test]
fn test_label_span_trait() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 6 },
    };

    let label = SemanticTokenBuilder::label("python".to_string(), span.clone());
    let retrieved_span = label.span();

    assert_eq!(retrieved_span, &span);
}

/// Test multiple Labels in sequence
#[test]
fn test_multiple_labels() {
    let analyzer = SemanticAnalyzer::new();

    // Create multiple Identifier tokens
    let scanner_tokens = vec![
        ScannerToken::Identifier {
            content: "python".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 6 },
            },
        },
        ScannerToken::Identifier {
            content: "org.example".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 7 },
                end: Position { row: 1, column: 18 },
            },
        },
        ScannerToken::Identifier {
            content: "custom-label".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 12 },
            },
        },
    ];

    // Run semantic analysis
    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 3);

    // Verify all tokens are Labels
    let expected_labels = ["python", "org.example", "custom-label"];
    for (i, token) in semantic_tokens.tokens.iter().enumerate() {
        match token {
            SemanticToken::Label { text, span } => {
                assert_eq!(text, expected_labels[i]);
                // Verify each label has correct position
                match i {
                    0 => {
                        assert_eq!(span.start.row, 1);
                        assert_eq!(span.start.column, 0);
                        assert_eq!(span.end.column, 6);
                    }
                    1 => {
                        assert_eq!(span.start.row, 1);
                        assert_eq!(span.start.column, 7);
                        assert_eq!(span.end.column, 18);
                    }
                    2 => {
                        assert_eq!(span.start.row, 2);
                        assert_eq!(span.start.column, 0);
                        assert_eq!(span.end.column, 12);
                    }
                    _ => panic!("Unexpected token index"),
                }
            }
            _ => panic!(
                "Expected Label semantic token at position {}, got {:?}",
                i, token
            ),
        }
    }
}

/// Test Label with structural tokens
#[test]
fn test_label_with_structural_tokens() {
    let analyzer = SemanticAnalyzer::new();

    // Create tokens including Identifier and structural tokens
    let scanner_tokens = vec![
        ScannerToken::Indent {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::Identifier {
            content: "python".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 4 },
                end: Position { row: 1, column: 10 },
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
        SemanticToken::Indent { span } => {
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 4);
        }
        _ => panic!("Expected Indent semantic token"),
    }

    // Verify Identifier is transformed to Label
    match &semantic_tokens.tokens[1] {
        SemanticToken::Label { text, span } => {
            assert_eq!(text, "python");
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 4);
            assert_eq!(span.end.column, 10);
        }
        _ => panic!("Expected Label semantic token"),
    }

    // Verify structural tokens are passed through unchanged
    match &semantic_tokens.tokens[2] {
        SemanticToken::Dedent { span } => {
            assert_eq!(span.start.row, 2);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 0);
        }
        _ => panic!("Expected Dedent semantic token"),
    }
}

/// Test Label validation helper methods
#[test]
fn test_label_validation_helpers() {
    let analyzer = SemanticAnalyzer::new();

    // Test valid label start characters
    assert!(analyzer.is_valid_label_start('a'));
    assert!(analyzer.is_valid_label_start('Z'));
    assert!(!analyzer.is_valid_label_start('1'));
    assert!(!analyzer.is_valid_label_start('_'));
    assert!(!analyzer.is_valid_label_start('-'));

    // Test valid label characters
    assert!(analyzer.is_valid_label_char('a'));
    assert!(analyzer.is_valid_label_char('Z'));
    assert!(analyzer.is_valid_label_char('1'));
    assert!(analyzer.is_valid_label_char('_'));
    assert!(analyzer.is_valid_label_char('-'));
    assert!(!analyzer.is_valid_label_char(' '));
    assert!(!analyzer.is_valid_label_char('@'));
    assert!(!analyzer.is_valid_label_char('!'));
}
