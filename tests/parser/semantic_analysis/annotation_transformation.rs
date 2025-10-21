//! Tests for Annotation semantic token transformation
//!
//! This module tests the transformation of scanner tokens into
//! Annotation semantic tokens as specified in Issue #88.

use txxt::cst::high_level_tokens::{HighLevelToken, HighLevelTokenBuilder, HighLevelTokenSpan};
use txxt::cst::{Position, ScannerToken, SourceSpan};
use txxt::syntax::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_annotation_basic_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 3 },
            },
        },
        ScannerToken::Text {
            content: "note".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 3 },
                end: Position { row: 1, column: 7 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 7 },
                end: Position { row: 1, column: 8 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 8 },
                end: Position { row: 1, column: 10 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 10 },
    };

    let result = analyzer.transform_annotation(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::Annotation {
            label,
            parameters,
            content,
            span: token_span,
            ..
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());
            assert!(content.is_none());

            // Check that the label is a TextSpan
            match label.as_ref() {
                HighLevelToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "note");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }
        }
        _ => panic!(
            "Expected Annotation semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_annotation_with_content() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 3 },
            },
        },
        ScannerToken::Text {
            content: "warning".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 3 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 11 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 11 },
                end: Position { row: 1, column: 13 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 13 },
                end: Position { row: 1, column: 14 },
            },
        },
        ScannerToken::Text {
            content: "This is important".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 14 },
                end: Position { row: 1, column: 30 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 30 },
    };

    let result = analyzer.transform_annotation(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::Annotation {
            label,
            parameters,
            content,
            span: token_span,
            ..
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());
            assert!(content.is_some());

            // Check that the label is correct
            match label.as_ref() {
                HighLevelToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "warning");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }

            // Check that the content is correct
            match content.as_ref().unwrap().as_ref() {
                HighLevelToken::TextSpan {
                    content: content_text,
                    ..
                } => {
                    assert_eq!(content_text, "This is important");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected Annotation semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_annotation_with_parameters() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 3 },
            },
        },
        ScannerToken::Text {
            content: "meta".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 3 },
                end: Position { row: 1, column: 7 },
            },
        },
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position { row: 1, column: 7 },
                end: Position { row: 1, column: 8 },
            },
        },
        ScannerToken::Text {
            content: "version=2.0".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 8 },
                end: Position { row: 1, column: 19 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 19 },
                end: Position { row: 1, column: 20 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 20 },
                end: Position { row: 1, column: 22 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 22 },
    };

    let result = analyzer.transform_annotation(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::Annotation {
            label,
            parameters,
            content,
            span: token_span,
            ..
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_some());
            assert!(content.is_none());

            // Check that the label is correct
            match label.as_ref() {
                HighLevelToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "meta");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }

            // Check that the parameters are correct
            match parameters.as_ref().unwrap().as_ref() {
                HighLevelToken::Parameters { params, .. } => {
                    assert!(params.contains_key("raw"));
                    assert_eq!(params.get("raw").unwrap(), "version=2.0");
                }
                _ => panic!("Expected Parameters, got {:?}", parameters.as_ref()),
            }
        }
        _ => panic!(
            "Expected Annotation semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_annotation_invalid_structure() {
    let analyzer = SemanticAnalyzer::new();

    // Test with too few tokens
    let tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Text {
            content: "note".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 6 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 6 },
    };

    let result = analyzer.transform_annotation(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Annotation must have at least 5 tokens"));
        }
        _ => panic!("Expected AnalysisError for invalid structure"),
    }
}

#[test]
fn test_annotation_no_opening_marker() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "note".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 4 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 11 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 11 },
                end: Position { row: 1, column: 13 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 13 },
    };

    let result = analyzer.transform_annotation(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Annotation must start with TxxtMarker"));
        }
        _ => panic!("Expected AnalysisError for no opening marker"),
    }
}

#[test]
fn test_annotation_no_closing_marker() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 3 },
            },
        },
        ScannerToken::Text {
            content: "note".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 3 },
                end: Position { row: 1, column: 7 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 7 },
                end: Position { row: 1, column: 8 },
            },
        },
        ScannerToken::Text {
            content: "content".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 8 },
                end: Position { row: 1, column: 15 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 15 },
    };

    let result = analyzer.transform_annotation(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Annotation must have closing TxxtMarker"));
        }
        _ => panic!("Expected AnalysisError for no closing marker"),
    }
}

#[test]
fn test_annotation_builder() {
    let label_token = HighLevelTokenBuilder::text_span(
        "note".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 3 },
            end: Position { row: 1, column: 7 },
        },
    );

    let content_token = HighLevelTokenBuilder::text_span(
        "This is important".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 14 },
            end: Position { row: 1, column: 30 },
        },
    );

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 30 },
    };

    let semantic_token =
        HighLevelTokenBuilder::annotation(label_token, None, Some(content_token), span.clone());

    match semantic_token {
        HighLevelToken::Annotation {
            label,
            parameters,
            content,
            span: token_span,
            ..
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());
            assert!(content.is_some());

            match label.as_ref() {
                HighLevelToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "note");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }

            match content.as_ref().unwrap().as_ref() {
                HighLevelToken::TextSpan {
                    content: content_text,
                    ..
                } => {
                    assert_eq!(content_text, "This is important");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected Annotation semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_annotation_span_trait() {
    let label_token = HighLevelTokenBuilder::text_span(
        "note".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 3 },
            end: Position { row: 1, column: 7 },
        },
    );

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 10 },
    };

    let semantic_token = HighLevelTokenBuilder::annotation(label_token, None, None, span.clone());
    let token_span = semantic_token.span();

    assert_eq!(token_span, &span);
}

#[test]
fn test_annotation_different_labels() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = ["note", "warning", "author", "title", "meta"];

    for label_text in test_cases.iter() {
        let tokens = vec![
            ScannerToken::TxxtMarker {
                span: SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 2 },
                },
            },
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position { row: 1, column: 2 },
                    end: Position { row: 1, column: 3 },
                },
            },
            ScannerToken::Text {
                content: label_text.to_string(),
                span: SourceSpan {
                    start: Position { row: 1, column: 3 },
                    end: Position {
                        row: 1,
                        column: 3 + label_text.len(),
                    },
                },
            },
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position {
                        row: 1,
                        column: 3 + label_text.len(),
                    },
                    end: Position {
                        row: 1,
                        column: 4 + label_text.len(),
                    },
                },
            },
            ScannerToken::TxxtMarker {
                span: SourceSpan {
                    start: Position {
                        row: 1,
                        column: 4 + label_text.len(),
                    },
                    end: Position {
                        row: 1,
                        column: 6 + label_text.len(),
                    },
                },
            },
        ];

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: 6 + label_text.len(),
            },
        };

        let result = analyzer.transform_annotation(tokens, span.clone());
        assert!(result.is_ok(), "Should succeed for label: {}", label_text);

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::Annotation {
                label,
                parameters,
                content,
                span: token_span,
                ..
            } => {
                assert_eq!(token_span, span);
                assert!(parameters.is_none());
                assert!(content.is_none());

                match label.as_ref() {
                    HighLevelToken::TextSpan {
                        content: actual_label,
                        ..
                    } => {
                        assert_eq!(actual_label, *label_text);
                    }
                    _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
                }
            }
            _ => panic!(
                "Expected Annotation semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_annotation_complex_content() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 3 },
            },
        },
        ScannerToken::Text {
            content: "description".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 3 },
                end: Position { row: 1, column: 14 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 14 },
                end: Position { row: 1, column: 15 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 15 },
                end: Position { row: 1, column: 17 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 17 },
                end: Position { row: 1, column: 18 },
            },
        },
        ScannerToken::Text {
            content: "This".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 18 },
                end: Position { row: 1, column: 22 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 22 },
                end: Position { row: 1, column: 23 },
            },
        },
        ScannerToken::Text {
            content: "is".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 23 },
                end: Position { row: 1, column: 25 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 25 },
                end: Position { row: 1, column: 26 },
            },
        },
        ScannerToken::Text {
            content: "complex".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 26 },
                end: Position { row: 1, column: 33 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 33 },
    };

    let result = analyzer.transform_annotation(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::Annotation {
            label,
            parameters,
            content,
            span: token_span,
            ..
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());
            assert!(content.is_some());

            match label.as_ref() {
                HighLevelToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "description");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }

            match content.as_ref().unwrap().as_ref() {
                HighLevelToken::TextSpan {
                    content: content_text,
                    ..
                } => {
                    assert_eq!(content_text, "This is complex");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected Annotation semantic token, got {:?}",
            semantic_token
        ),
    }
}
