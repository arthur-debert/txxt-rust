//! Tests for Definition semantic token transformation
//!
//! This module tests the transformation of scanner tokens into
//! Definition semantic tokens as specified in Issue #88.

use txxt::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};
use txxt::ast::semantic_tokens::{SemanticToken, SemanticTokenBuilder, SemanticTokenSpan};
use txxt::parser::pipeline::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_definition_basic_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "Term".to_string(),
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
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 7 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 7 },
    };

    let result = analyzer.transform_definition(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::Definition {
            term,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            // Check that the term is a TextSpan
            match term.as_ref() {
                SemanticToken::TextSpan {
                    content: term_content,
                    ..
                } => {
                    assert_eq!(term_content, "Term");
                }
                _ => panic!("Expected TextSpan term, got {:?}", term.as_ref()),
            }
        }
        _ => panic!(
            "Expected Definition semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_definition_with_parameters() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "Term".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position { row: 1, column: 4 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: "ref=important".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 18 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 18 },
                end: Position { row: 1, column: 19 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 19 },
                end: Position { row: 1, column: 21 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 21 },
    };

    let result = analyzer.transform_definition(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::Definition {
            term,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_some());

            // Check that the term is correct
            match term.as_ref() {
                SemanticToken::TextSpan {
                    content: term_content,
                    ..
                } => {
                    assert_eq!(term_content, "Term");
                }
                _ => panic!("Expected TextSpan term, got {:?}", term.as_ref()),
            }

            // Check that the parameters are correct
            match parameters.as_ref().unwrap().as_ref() {
                SemanticToken::Parameters { params, .. } => {
                    assert!(params.contains_key("raw"));
                    assert_eq!(params.get("raw").unwrap(), "ref=important");
                }
                _ => panic!("Expected Parameters, got {:?}", parameters.as_ref()),
            }
        }
        _ => panic!(
            "Expected Definition semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_definition_complex_term() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "Machine".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
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
            content: "Learning".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 8 },
                end: Position { row: 1, column: 16 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 16 },
                end: Position { row: 1, column: 17 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 17 },
                end: Position { row: 1, column: 19 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 19 },
    };

    let result = analyzer.transform_definition(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::Definition {
            term,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            // Check that the term combines multiple text tokens
            match term.as_ref() {
                SemanticToken::TextSpan {
                    content: term_content,
                    ..
                } => {
                    assert_eq!(term_content, "Machine Learning");
                }
                _ => panic!("Expected TextSpan term, got {:?}", term.as_ref()),
            }
        }
        _ => panic!(
            "Expected Definition semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_definition_invalid_structure() {
    let analyzer = SemanticAnalyzer::new();

    // Test with too few tokens
    let tokens = vec![ScannerToken::Text {
        content: "Term".to_string(),
        span: SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 4 },
        },
    }];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 4 },
    };

    let result = analyzer.transform_definition(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Definition must have at least 3 tokens"));
        }
        _ => panic!("Expected AnalysisError for invalid structure"),
    }
}

#[test]
fn test_definition_no_closing_marker() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "Term".to_string(),
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
            content: "content".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 12 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 12 },
    };

    let result = analyzer.transform_definition(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("Definition must end with TxxtMarker"));
        }
        _ => panic!("Expected AnalysisError for no closing marker"),
    }
}

#[test]
fn test_definition_builder() {
    let term_token = SemanticTokenBuilder::text_span(
        "Term".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 4 },
        },
    );

    let parameters_token = SemanticTokenBuilder::parameters(
        std::collections::HashMap::from([("ref".to_string(), "important".to_string())]),
        SourceSpan {
            start: Position { row: 1, column: 5 },
            end: Position { row: 1, column: 18 },
        },
    );

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 21 },
    };

    let semantic_token =
        SemanticTokenBuilder::definition(term_token, Some(parameters_token), span.clone());

    match semantic_token {
        SemanticToken::Definition {
            term,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_some());

            match term.as_ref() {
                SemanticToken::TextSpan {
                    content: term_content,
                    ..
                } => {
                    assert_eq!(term_content, "Term");
                }
                _ => panic!("Expected TextSpan term, got {:?}", term.as_ref()),
            }

            match parameters.as_ref().unwrap().as_ref() {
                SemanticToken::Parameters { params, .. } => {
                    assert!(params.contains_key("ref"));
                    assert_eq!(params.get("ref").unwrap(), "important");
                }
                _ => panic!("Expected Parameters, got {:?}", parameters.as_ref()),
            }
        }
        _ => panic!(
            "Expected Definition semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_definition_span_trait() {
    let term_token = SemanticTokenBuilder::text_span(
        "Term".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 4 },
        },
    );

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 7 },
    };

    let semantic_token = SemanticTokenBuilder::definition(term_token, None, span.clone());
    let token_span = semantic_token.span();

    assert_eq!(token_span, &span);
}

#[test]
fn test_definition_different_terms() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = ["Term", "Algorithm", "Data Structure", "API", "Function"];

    for term_text in test_cases.iter() {
        let tokens = vec![
            ScannerToken::Text {
                content: term_text.to_string(),
                span: SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position {
                        row: 1,
                        column: term_text.len(),
                    },
                },
            },
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position {
                        row: 1,
                        column: term_text.len(),
                    },
                    end: Position {
                        row: 1,
                        column: term_text.len() + 1,
                    },
                },
            },
            ScannerToken::TxxtMarker {
                span: SourceSpan {
                    start: Position {
                        row: 1,
                        column: term_text.len() + 1,
                    },
                    end: Position {
                        row: 1,
                        column: term_text.len() + 3,
                    },
                },
            },
        ];

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: term_text.len() + 3,
            },
        };

        let result = analyzer.transform_definition(tokens, span.clone());
        assert!(result.is_ok(), "Should succeed for term: {}", term_text);

        let semantic_token = result.unwrap();
        match semantic_token {
            SemanticToken::Definition {
                term,
                parameters,
                span: token_span,
            } => {
                assert_eq!(token_span, span);
                assert!(parameters.is_none());

                match term.as_ref() {
                    SemanticToken::TextSpan {
                        content: actual_term,
                        ..
                    } => {
                        assert_eq!(actual_term, *term_text);
                    }
                    _ => panic!("Expected TextSpan term, got {:?}", term.as_ref()),
                }
            }
            _ => panic!(
                "Expected Definition semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_definition_with_multiple_parameters() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "Algorithm".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 9 },
            },
        },
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position { row: 1, column: 9 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::Text {
            content: "type=sorting".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 22 },
            },
        },
        ScannerToken::Comma {
            span: SourceSpan {
                start: Position { row: 1, column: 22 },
                end: Position { row: 1, column: 23 },
            },
        },
        ScannerToken::Text {
            content: "complexity=O(n log n)".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 23 },
                end: Position { row: 1, column: 43 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 43 },
                end: Position { row: 1, column: 44 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 44 },
                end: Position { row: 1, column: 46 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 46 },
    };

    let result = analyzer.transform_definition(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::Definition {
            term,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_some());

            match term.as_ref() {
                SemanticToken::TextSpan {
                    content: term_content,
                    ..
                } => {
                    assert_eq!(term_content, "Algorithm");
                }
                _ => panic!("Expected TextSpan term, got {:?}", term.as_ref()),
            }

            match parameters.as_ref().unwrap().as_ref() {
                SemanticToken::Parameters { params, .. } => {
                    assert!(params.contains_key("raw"));
                    assert_eq!(
                        params.get("raw").unwrap(),
                        "type=sorting,complexity=O(n log n)"
                    );
                }
                _ => panic!("Expected Parameters, got {:?}", parameters.as_ref()),
            }
        }
        _ => panic!(
            "Expected Definition semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_definition_empty_parameters() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "Term".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position { row: 1, column: 4 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 6 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 6 },
                end: Position { row: 1, column: 8 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 8 },
    };

    let result = analyzer.transform_definition(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::Definition {
            term,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none()); // Empty parameters should result in None

            match term.as_ref() {
                SemanticToken::TextSpan {
                    content: term_content,
                    ..
                } => {
                    assert_eq!(term_content, "Term");
                }
                _ => panic!("Expected TextSpan term, got {:?}", term.as_ref()),
            }
        }
        _ => panic!(
            "Expected Definition semantic token, got {:?}",
            semantic_token
        ),
    }
}
