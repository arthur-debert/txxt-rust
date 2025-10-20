//! Tests for Plain Text Line semantic token transformation
//!
//! This module tests the transformation of sequences of Text scanner tokens
//! into PlainTextLine semantic tokens as specified in Issue #87.

use txxt::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};
use txxt::ast::tokens::high_level::{HighLevelToken, HighLevelTokenBuilder, HighLevelTokenSpan};
use txxt::lexer::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_plain_text_line_single_text_token() {
    let analyzer = SemanticAnalyzer::new();

    let text_tokens = vec![ScannerToken::Text {
        content: "Hello world".to_string(),
        span: SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 11 },
        },
    }];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 11 },
    };

    let result = analyzer.transform_plain_text_line(text_tokens, line_span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::PlainTextLine { content, span } => {
            assert_eq!(span, line_span);

            // Check that the content is a TextSpan
            match content.as_ref() {
                HighLevelToken::TextSpan {
                    content: text_content,
                    span: text_span,
                } => {
                    assert_eq!(text_content, "Hello world");
                    assert_eq!(text_span, &line_span);
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected PlainTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_plain_text_line_multiple_text_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let text_tokens = vec![
        ScannerToken::Text {
            content: "Hello".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 6 },
            },
        },
        ScannerToken::Text {
            content: "world".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 6 },
                end: Position { row: 1, column: 11 },
            },
        },
    ];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 11 },
    };

    let result = analyzer.transform_plain_text_line(text_tokens, line_span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::PlainTextLine { content, span } => {
            assert_eq!(span, line_span);

            // Check that the content is a combined TextSpan
            match content.as_ref() {
                HighLevelToken::TextSpan {
                    content: text_content,
                    span: text_span,
                } => {
                    assert_eq!(text_content, "Hello world");
                    assert_eq!(text_span, &line_span);
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected PlainTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_plain_text_line_empty_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let text_tokens = vec![];
    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 0 },
    };

    let result = analyzer.transform_plain_text_line(text_tokens, line_span);
    assert!(result.is_err(), "Should fail for empty tokens");

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Plain text line must contain at least one text token"));
        }
        _ => panic!("Expected AnalysisError for empty tokens"),
    }
}

#[test]
fn test_plain_text_line_invalid_token_type() {
    let analyzer = SemanticAnalyzer::new();

    let text_tokens = vec![
        ScannerToken::Text {
            content: "Hello".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Identifier {
            content: "invalid".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 12 },
            },
        },
    ];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 12 },
    };

    let result = analyzer.transform_plain_text_line(text_tokens, line_span);
    assert!(result.is_err(), "Should fail for non-Text tokens");

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Plain text line can only contain Text tokens"));
        }
        _ => panic!("Expected AnalysisError for invalid token type"),
    }
}

#[test]
fn test_plain_text_line_different_content_types() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        ("Simple text", vec!["Simple text"]),
        ("Text with numbers 123", vec!["Text with numbers 123"]),
        ("Text with symbols !@#$%", vec!["Text with symbols !@#$%"]),
        ("Unicode text: 你好世界", vec!["Unicode text: 你好世界"]),
        ("Hello world", vec!["Hello", " ", "world"]),
        ("Multi word text", vec!["Multi", " ", "word", " ", "text"]),
    ];

    for (expected_content, text_parts) in test_cases.iter() {
        let text_tokens: Vec<ScannerToken> = text_parts
            .iter()
            .enumerate()
            .map(|(i, part)| ScannerToken::Text {
                content: part.to_string(),
                span: SourceSpan {
                    start: Position { row: 1, column: i },
                    end: Position {
                        row: 1,
                        column: i + part.len(),
                    },
                },
            })
            .collect();

        let line_span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: expected_content.len(),
            },
        };

        let result = analyzer.transform_plain_text_line(text_tokens, line_span.clone());
        assert!(
            result.is_ok(),
            "Should succeed for content: {}",
            expected_content
        );

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::PlainTextLine { content, span } => {
                assert_eq!(span, line_span);

                match content.as_ref() {
                    HighLevelToken::TextSpan {
                        content: text_content,
                        ..
                    } => {
                        assert_eq!(text_content, *expected_content);
                    }
                    _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
                }
            }
            _ => panic!(
                "Expected PlainTextLine semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_plain_text_line_builder() {
    let text_span = HighLevelTokenBuilder::text_span(
        "Hello world".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 11 },
        },
    );

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 11 },
    };

    let semantic_token = HighLevelTokenBuilder::plain_text_line(text_span, line_span.clone());

    match semantic_token {
        HighLevelToken::PlainTextLine { content, span } => {
            assert_eq!(span, line_span);

            match content.as_ref() {
                HighLevelToken::TextSpan {
                    content: text_content,
                    ..
                } => {
                    assert_eq!(text_content, "Hello world");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected PlainTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_plain_text_line_span_trait() {
    let text_span = HighLevelTokenBuilder::text_span(
        "Hello world".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 11 },
        },
    );

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 11 },
    };

    let semantic_token = HighLevelTokenBuilder::plain_text_line(text_span, line_span.clone());
    let token_span = semantic_token.span();

    assert_eq!(token_span, &line_span);
}

#[test]
fn test_plain_text_line_different_positions() {
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
        let text_tokens = vec![ScannerToken::Text {
            content: "Test".to_string(),
            span: SourceSpan {
                start: *start,
                end: *end,
            },
        }];

        let line_span = SourceSpan {
            start: *start,
            end: *end,
        };

        let result = analyzer.transform_plain_text_line(text_tokens, line_span.clone());
        assert!(result.is_ok());

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::PlainTextLine { span, .. } => {
                assert_eq!(span, line_span);
            }
            _ => panic!(
                "Expected PlainTextLine semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_plain_text_line_complex_content() {
    let analyzer = SemanticAnalyzer::new();

    // Test with complex content including spaces, punctuation, and mixed case
    let text_tokens = vec![
        ScannerToken::Text {
            content: "Hello".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: ", ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 7 },
            },
        },
        ScannerToken::Text {
            content: "world".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 7 },
                end: Position { row: 1, column: 12 },
            },
        },
        ScannerToken::Text {
            content: "!".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 12 },
                end: Position { row: 1, column: 13 },
            },
        },
    ];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 13 },
    };

    let result = analyzer.transform_plain_text_line(text_tokens, line_span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::PlainTextLine { content, span } => {
            assert_eq!(span, line_span);

            match content.as_ref() {
                HighLevelToken::TextSpan {
                    content: text_content,
                    ..
                } => {
                    assert_eq!(text_content, "Hello, world!");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected PlainTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}
