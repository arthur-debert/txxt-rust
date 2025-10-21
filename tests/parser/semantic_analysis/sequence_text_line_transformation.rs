#![allow(deprecated)]
//! Tests for Sequence Text Line semantic token transformation
//!
//! This module tests the transformation of sequence markers followed by text content
//! into SequenceTextLine semantic tokens as specified in Issue #86.

use txxt::cst::high_level_tokens::{
    HighLevelNumberingForm, HighLevelNumberingStyle, HighLevelToken, HighLevelTokenBuilder,
    HighLevelTokenSpan,
};
use txxt::cst::{Position, ScannerToken, SourceSpan};
use txxt::syntax::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_sequence_text_line_single_text_token() {
    let analyzer = SemanticAnalyzer::new();

    let marker_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    );

    let text_tokens = vec![ScannerToken::Text {
        content: "First item".to_string(),
        span: SourceSpan {
            start: Position { row: 1, column: 2 },
            end: Position { row: 1, column: 11 },
        },
    }];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 11 },
    };

    let result =
        analyzer.transform_sequence_text_line(marker_token, text_tokens, line_span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::SequenceTextLine {
            indentation_chars: _,
            marker,
            content,
            span,
            ..
        } => {
            assert_eq!(span, line_span);

            // Check that the marker is preserved
            match marker.as_ref() {
                HighLevelToken::SequenceMarker {
                    style,
                    form,
                    marker: marker_text,
                    ..
                } => {
                    assert_eq!(*style, HighLevelNumberingStyle::Numeric);
                    assert_eq!(*form, HighLevelNumberingForm::Regular);
                    assert_eq!(marker_text, "1.");
                }
                _ => panic!("Expected SequenceMarker, got {:?}", marker.as_ref()),
            }

            // Check that the content is a TextSpan
            match content.as_ref() {
                HighLevelToken::TextSpan {
                    content: text_content,
                    span: text_span,
                    ..
                } => {
                    assert_eq!(text_content, "First item");
                    assert_eq!(text_span, &line_span);
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected SequenceTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_text_line_multiple_text_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let marker_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Plain,
        HighLevelNumberingForm::Regular,
        "-".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 1 },
        },
    );

    let text_tokens = vec![
        ScannerToken::Text {
            content: "Item".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 1 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: " with".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::Text {
            content: " content".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 17 },
            },
        },
    ];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 17 },
    };

    let result =
        analyzer.transform_sequence_text_line(marker_token, text_tokens, line_span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::SequenceTextLine {
            indentation_chars: _,
            marker,
            content,
            span,
            ..
        } => {
            assert_eq!(span, line_span);

            // Check that the marker is preserved
            match marker.as_ref() {
                HighLevelToken::SequenceMarker {
                    style,
                    form,
                    marker: marker_text,
                    ..
                } => {
                    assert_eq!(*style, HighLevelNumberingStyle::Plain);
                    assert_eq!(*form, HighLevelNumberingForm::Regular);
                    assert_eq!(marker_text, "-");
                }
                _ => panic!("Expected SequenceMarker, got {:?}", marker.as_ref()),
            }

            // Check that the content is combined
            match content.as_ref() {
                HighLevelToken::TextSpan {
                    content: text_content,
                    span: text_span,
                    ..
                } => {
                    assert_eq!(text_content, "Item with content");
                    assert_eq!(text_span, &line_span);
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected SequenceTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_text_line_empty_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let marker_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    );

    let text_tokens = vec![];
    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 2 },
    };

    let result = analyzer.transform_sequence_text_line(marker_token, text_tokens, line_span);
    assert!(result.is_err(), "Should fail for empty tokens");

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Sequence text line must contain at least one text token"));
        }
        _ => panic!("Expected AnalysisError for empty tokens"),
    }
}

#[test]
fn test_sequence_text_line_invalid_token_type() {
    let analyzer = SemanticAnalyzer::new();

    let marker_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    );

    let text_tokens = vec![
        ScannerToken::Text {
            content: "Valid".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 7 },
            },
        },
        ScannerToken::Identifier {
            content: "invalid".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 7 },
                end: Position { row: 1, column: 14 },
            },
        },
    ];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 14 },
    };

    let result = analyzer.transform_sequence_text_line(marker_token, text_tokens, line_span);
    assert!(result.is_err(), "Should fail for non-Text tokens");

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Sequence text line can only contain Text tokens"));
        }
        _ => panic!("Expected AnalysisError for invalid token type"),
    }
}

#[test]
fn test_sequence_text_line_invalid_marker_type() {
    let analyzer = SemanticAnalyzer::new();

    let marker_token = HighLevelTokenBuilder::text_span(
        "1.".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    );

    let text_tokens = vec![ScannerToken::Text {
        content: "Content".to_string(),
        span: SourceSpan {
            start: Position { row: 1, column: 2 },
            end: Position { row: 1, column: 9 },
        },
    }];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 9 },
    };

    let result = analyzer.transform_sequence_text_line(marker_token, text_tokens, line_span);
    assert!(result.is_err(), "Should fail for non-SequenceMarker");

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Sequence text line marker must be a SequenceMarker token"));
        }
        _ => panic!("Expected AnalysisError for invalid marker type"),
    }
}

#[test]
fn test_sequence_text_line_different_marker_types() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        (
            HighLevelTokenBuilder::sequence_marker(
                HighLevelNumberingStyle::Numeric,
                HighLevelNumberingForm::Regular,
                "1.".to_string(),
                SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 2 },
                },
            ),
            "Numeric item",
        ),
        (
            HighLevelTokenBuilder::sequence_marker(
                HighLevelNumberingStyle::Alphabetic,
                HighLevelNumberingForm::Regular,
                "a.".to_string(),
                SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 2 },
                },
            ),
            "Alphabetic item",
        ),
        (
            HighLevelTokenBuilder::sequence_marker(
                HighLevelNumberingStyle::Roman,
                HighLevelNumberingForm::Regular,
                "i.".to_string(),
                SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 2 },
                },
            ),
            "Roman item",
        ),
        (
            HighLevelTokenBuilder::sequence_marker(
                HighLevelNumberingStyle::Plain,
                HighLevelNumberingForm::Regular,
                "-".to_string(),
                SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 1 },
                },
            ),
            "Plain item",
        ),
    ];

    for (marker_token, expected_content) in test_cases.iter() {
        let text_tokens = vec![ScannerToken::Text {
            content: expected_content.to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position {
                    row: 1,
                    column: 2 + expected_content.len(),
                },
            },
        }];

        let line_span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: 2 + expected_content.len(),
            },
        };

        let result = analyzer.transform_sequence_text_line(
            marker_token.clone(),
            text_tokens,
            line_span.clone(),
        );
        assert!(
            result.is_ok(),
            "Should succeed for marker: {:?}",
            marker_token
        );

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::SequenceTextLine {
                indentation_chars: _,
                marker,
                content,
                span,
                ..
            } => {
                assert_eq!(span, line_span);

                // Check that the marker is preserved
                match marker.as_ref() {
                    HighLevelToken::SequenceMarker { .. } => {} // OK
                    _ => panic!("Expected SequenceMarker, got {:?}", marker.as_ref()),
                }

                // Check that the content is correct
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
                "Expected SequenceTextLine semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_sequence_text_line_builder() {
    let marker_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    );

    let content_token = HighLevelTokenBuilder::text_span(
        "Hello world".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 2 },
            end: Position { row: 1, column: 13 },
        },
    );

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 13 },
    };

    let semantic_token = HighLevelTokenBuilder::sequence_text_line(
        String::new(),
        marker_token,
        content_token,
        line_span.clone(),
    );

    match semantic_token {
        HighLevelToken::SequenceTextLine {
            indentation_chars: _,
            marker,
            content,
            span,
            ..
        } => {
            assert_eq!(span, line_span);

            match marker.as_ref() {
                HighLevelToken::SequenceMarker {
                    style,
                    form,
                    marker: marker_text,
                    ..
                } => {
                    assert_eq!(*style, HighLevelNumberingStyle::Numeric);
                    assert_eq!(*form, HighLevelNumberingForm::Regular);
                    assert_eq!(marker_text, "1.");
                }
                _ => panic!("Expected SequenceMarker, got {:?}", marker.as_ref()),
            }

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
            "Expected SequenceTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_sequence_text_line_span_trait() {
    let marker_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    );

    let content_token = HighLevelTokenBuilder::text_span(
        "Hello world".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 2 },
            end: Position { row: 1, column: 13 },
        },
    );

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 13 },
    };

    let semantic_token = HighLevelTokenBuilder::sequence_text_line(
        String::new(),
        marker_token,
        content_token,
        line_span.clone(),
    );
    let token_span = semantic_token.span();

    assert_eq!(token_span, &line_span);
}

#[test]
fn test_sequence_text_line_different_positions() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [
        (
            Position { row: 1, column: 0 },
            Position { row: 1, column: 5 },
        ),
        (
            Position { row: 2, column: 4 },
            Position { row: 2, column: 12 },
        ),
        (
            Position { row: 5, column: 8 },
            Position { row: 5, column: 16 },
        ),
    ];

    for (start, end) in test_cases.iter() {
        let marker_token = HighLevelTokenBuilder::sequence_marker(
            HighLevelNumberingStyle::Numeric,
            HighLevelNumberingForm::Regular,
            "1.".to_string(),
            SourceSpan {
                start: *start,
                end: Position {
                    row: start.row,
                    column: start.column + 2,
                },
            },
        );

        let text_tokens = vec![ScannerToken::Text {
            content: "Test".to_string(),
            span: SourceSpan {
                start: Position {
                    row: start.row,
                    column: start.column + 2,
                },
                end: *end,
            },
        }];

        let line_span = SourceSpan {
            start: *start,
            end: *end,
        };

        let result =
            analyzer.transform_sequence_text_line(marker_token, text_tokens, line_span.clone());
        assert!(result.is_ok());

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::SequenceTextLine { span, .. } => {
                assert_eq!(span, line_span);
            }
            _ => panic!(
                "Expected SequenceTextLine semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_sequence_text_line_complex_content() {
    let analyzer = SemanticAnalyzer::new();

    let marker_token = HighLevelTokenBuilder::sequence_marker(
        HighLevelNumberingStyle::Numeric,
        HighLevelNumberingForm::Regular,
        "1.".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 2 },
        },
    );

    // Test with complex content including spaces, punctuation, and mixed case
    let text_tokens = vec![
        ScannerToken::Text {
            content: "Complex".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 9 },
            },
        },
        ScannerToken::Text {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 9 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::Text {
            content: "item".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 14 },
            },
        },
        ScannerToken::Text {
            content: "!".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 14 },
                end: Position { row: 1, column: 15 },
            },
        },
    ];

    let line_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 15 },
    };

    let result =
        analyzer.transform_sequence_text_line(marker_token, text_tokens, line_span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::SequenceTextLine {
            indentation_chars: _,
            marker,
            content,
            span,
            ..
        } => {
            assert_eq!(span, line_span);

            match marker.as_ref() {
                HighLevelToken::SequenceMarker {
                    style,
                    form,
                    marker: marker_text,
                    ..
                } => {
                    assert_eq!(*style, HighLevelNumberingStyle::Numeric);
                    assert_eq!(*form, HighLevelNumberingForm::Regular);
                    assert_eq!(marker_text, "1.");
                }
                _ => panic!("Expected SequenceMarker, got {:?}", marker.as_ref()),
            }

            match content.as_ref() {
                HighLevelToken::TextSpan {
                    content: text_content,
                    ..
                } => {
                    assert_eq!(text_content, "Complex item!");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected SequenceTextLine semantic token, got {:?}",
            semantic_token
        ),
    }
}
