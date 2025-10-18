//! Tests for Text Span semantic token transformation
//!
//! This module tests the transformation of Text scanner tokens into TextSpan
//! semantic tokens as specified in Issue #85.

use txxt::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};
use txxt::ast::semantic_tokens::{SemanticToken, SemanticTokenBuilder, SemanticTokenSpan};
use txxt::parser::pipeline::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_text_span_basic_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 5 },
    };

    let result = analyzer.transform_text_span("Hello".to_string(), span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::TextSpan {
            content,
            span: token_span,
        } => {
            assert_eq!(content, "Hello");
            assert_eq!(token_span, span);
        }
        _ => panic!("Expected TextSpan semantic token, got {:?}", semantic_token),
    }
}

#[test]
fn test_text_span_different_content() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        "Simple text",
        "Text with numbers 123",
        "Text with symbols !@#$%",
        "Unicode text: 你好世界",
        "Text with spaces and   tabs",
    ];

    for text_content in test_cases.iter() {
        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: text_content.len(),
            },
        };

        let result = analyzer.transform_text_span(text_content.to_string(), span.clone());
        assert!(result.is_ok(), "Should succeed for text: {}", text_content);

        let semantic_token = result.unwrap();
        match semantic_token {
            SemanticToken::TextSpan {
                content,
                span: token_span,
            } => {
                assert_eq!(content, *text_content);
                assert_eq!(token_span, span);
            }
            _ => panic!("Expected TextSpan semantic token, got {:?}", semantic_token),
        }
    }
}

#[test]
fn test_text_span_empty_content() {
    let analyzer = SemanticAnalyzer::new();

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 0 },
    };

    let result = analyzer.transform_text_span("".to_string(), span);
    assert!(result.is_err(), "Should fail for empty content");

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Text span content cannot be empty"));
        }
        _ => panic!("Expected AnalysisError for empty content"),
    }
}

#[test]
fn test_text_span_different_positions() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        (
            Position { row: 1, column: 0 },
            Position { row: 1, column: 5 },
        ),
        (
            Position { row: 2, column: 10 },
            Position { row: 2, column: 20 },
        ),
        (
            Position { row: 5, column: 3 },
            Position { row: 5, column: 8 },
        ),
    ];

    for (start, end) in test_cases.iter() {
        let span = SourceSpan {
            start: *start,
            end: *end,
        };

        let result = analyzer.transform_text_span("Test".to_string(), span.clone());
        assert!(result.is_ok());

        let semantic_token = result.unwrap();
        match semantic_token {
            SemanticToken::TextSpan {
                span: token_span, ..
            } => {
                assert_eq!(token_span, span);
            }
            _ => panic!("Expected TextSpan semantic token, got {:?}", semantic_token),
        }
    }
}

#[test]
fn test_text_span_in_semantic_analysis() {
    let analyzer = SemanticAnalyzer::new();

    let scanner_tokens = vec![
        ScannerToken::Text {
            content: "Hello world".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 11 },
            },
        },
        ScannerToken::Text {
            content: "Another line".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 12 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 2);

    // Check first text span
    match &semantic_tokens.tokens[0] {
        SemanticToken::TextSpan { content, span } => {
            assert_eq!(content, "Hello world");
            assert_eq!(span.start.row, 1);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 1);
            assert_eq!(span.end.column, 11);
        }
        _ => panic!(
            "Expected TextSpan semantic token, got {:?}",
            semantic_tokens.tokens[0]
        ),
    }

    // Check second text span
    match &semantic_tokens.tokens[1] {
        SemanticToken::TextSpan { content, span } => {
            assert_eq!(content, "Another line");
            assert_eq!(span.start.row, 2);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 2);
            assert_eq!(span.end.column, 12);
        }
        _ => panic!(
            "Expected TextSpan semantic token, got {:?}",
            semantic_tokens.tokens[1]
        ),
    }
}

#[test]
fn test_text_span_builder() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 5 },
    };

    let semantic_token = SemanticTokenBuilder::text_span("Hello".to_string(), span.clone());

    match semantic_token {
        SemanticToken::TextSpan {
            content,
            span: token_span,
        } => {
            assert_eq!(content, "Hello");
            assert_eq!(token_span, span);
        }
        _ => panic!("Expected TextSpan semantic token, got {:?}", semantic_token),
    }
}

#[test]
fn test_text_span_span_trait() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 5 },
    };

    let semantic_token = SemanticTokenBuilder::text_span("Hello".to_string(), span.clone());
    let token_span = semantic_token.span();

    assert_eq!(token_span, &span);
}

#[test]
fn test_text_span_with_structural_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let scanner_tokens = vec![
        ScannerToken::Indent {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Text {
            content: "Indented text".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 15 },
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
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 3);

    // Check that structural tokens are preserved
    match &semantic_tokens.tokens[0] {
        SemanticToken::Indent { .. } => {} // OK
        _ => panic!(
            "Expected Indent semantic token, got {:?}",
            semantic_tokens.tokens[0]
        ),
    }

    // Check that text span is created
    match &semantic_tokens.tokens[1] {
        SemanticToken::TextSpan { content, .. } => {
            assert_eq!(content, "Indented text");
        }
        _ => panic!(
            "Expected TextSpan semantic token, got {:?}",
            semantic_tokens.tokens[1]
        ),
    }

    // Check that dedent is preserved
    match &semantic_tokens.tokens[2] {
        SemanticToken::Dedent { .. } => {} // OK
        _ => panic!(
            "Expected Dedent semantic token, got {:?}",
            semantic_tokens.tokens[2]
        ),
    }
}

#[test]
fn test_text_span_multiple_text_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let scanner_tokens = vec![
        ScannerToken::Text {
            content: "First".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: "Second".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 11 },
            },
        },
        ScannerToken::Text {
            content: "Third".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 11 },
                end: Position { row: 1, column: 16 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();
    assert_eq!(semantic_tokens.len(), 3);

    // Each text token should become a separate TextSpan
    let expected_contents = ["First", "Second", "Third"];
    for (i, expected_content) in expected_contents.iter().enumerate() {
        match &semantic_tokens.tokens[i] {
            SemanticToken::TextSpan { content, .. } => {
                assert_eq!(content, *expected_content);
            }
            _ => panic!(
                "Expected TextSpan semantic token, got {:?}",
                semantic_tokens.tokens[i]
            ),
        }
    }
}
