//! Tests for VerbatimBlock semantic token transformation
//!
//! This module tests the transformation of scanner tokens into
//! VerbatimBlock semantic tokens as specified in Issue #89.

#![allow(unused_variables)]

use txxt::ast::scanner_tokens::{Position, ScannerToken, SourceSpan, WallType};
use txxt::ast::tokens::semantic::{SemanticToken, SemanticTokenBuilder, SemanticTokenSpan};
use txxt::parser::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_verbatim_block_basic_transformation() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "Hello World".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 11 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    def hello():\n        print(\"Hello\")\n".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 4 },
                end: Position { row: 4, column: 20 },
            },
        },
        ScannerToken::VerbatimLabel {
            content: "python".to_string(),
            span: SourceSpan {
                start: Position { row: 5, column: 0 },
                end: Position { row: 5, column: 8 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 5, column: 8 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::VerbatimBlock {
            title,
            wall,
            content,
            label,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            // Check that the title is correct
            match title.as_ref() {
                SemanticToken::TextSpan {
                    content: title_content,
                    ..
                } => {
                    assert_eq!(title_content, "Hello World");
                }
                _ => panic!("Expected TextSpan title, got {:?}", title.as_ref()),
            }

            // Check that the wall is correct (structural token)
            match wall.as_ref() {
                SemanticToken::TextSpan {
                    content: wall_content,
                    ..
                } => {
                    assert_eq!(wall_content, ""); // Wall is structural, no content
                }
                _ => panic!("Expected TextSpan wall, got {:?}", wall.as_ref()),
            }

            // Check that the content is correct
            match content.as_ref() {
                SemanticToken::TextSpan {
                    content: content_text,
                    ..
                } => {
                    assert_eq!(content_text, "    def hello():\n        print(\"Hello\")\n");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }

            // Check that the label is correct
            match label.as_ref() {
                SemanticToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "python");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }
        }
        _ => panic!(
            "Expected VerbatimBlock semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_verbatim_block_with_parameters() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "Code Example".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 12 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::Stretched,
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    console.log('Hello');\n".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 4 },
                end: Position { row: 3, column: 25 },
            },
        },
        ScannerToken::VerbatimLabel {
            content: "javascript:version=ES6".to_string(),
            span: SourceSpan {
                start: Position { row: 4, column: 0 },
                end: Position { row: 4, column: 22 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 4, column: 22 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::VerbatimBlock {
            title,
            wall,
            content,
            label,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_some());

            // Check that the title is correct
            match title.as_ref() {
                SemanticToken::TextSpan {
                    content: title_content,
                    ..
                } => {
                    assert_eq!(title_content, "Code Example");
                }
                _ => panic!("Expected TextSpan title, got {:?}", title.as_ref()),
            }

            // Check that the label is correct (should be just "javascript")
            match label.as_ref() {
                SemanticToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "javascript");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }

            // Check that the parameters are correct
            match parameters.as_ref().unwrap().as_ref() {
                SemanticToken::Parameters { params, .. } => {
                    assert!(params.contains_key("raw"));
                    assert_eq!(params.get("raw").unwrap(), "version=ES6");
                }
                _ => panic!("Expected Parameters, got {:?}", parameters.as_ref()),
            }
        }
        _ => panic!(
            "Expected VerbatimBlock semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_verbatim_block_multiple_content_tokens() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "Multi-line Code".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 15 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    function greet() {\n".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 4 },
                end: Position { row: 3, column: 0 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "        return 'Hello';\n".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 4, column: 0 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    }\n".to_string(),
            span: SourceSpan {
                start: Position { row: 4, column: 0 },
                end: Position { row: 5, column: 0 },
            },
        },
        ScannerToken::VerbatimLabel {
            content: "javascript".to_string(),
            span: SourceSpan {
                start: Position { row: 5, column: 0 },
                end: Position { row: 5, column: 10 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 5, column: 10 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::VerbatimBlock {
            title,
            wall,
            content,
            label,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            // Check that the content combines multiple tokens
            match content.as_ref() {
                SemanticToken::TextSpan {
                    content: content_text,
                    ..
                } => {
                    assert_eq!(
                        content_text,
                        "    function greet() {\n        return 'Hello';\n    }\n"
                    );
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected VerbatimBlock semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_verbatim_block_invalid_structure() {
    let analyzer = SemanticAnalyzer::new();

    // Test with too few tokens
    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "Title".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 2, column: 4 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("VerbatimBlock must have at least 3 tokens"));
        }
        _ => panic!("Expected AnalysisError for invalid structure"),
    }
}

#[test]
fn test_verbatim_block_no_title() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::Text {
            content: "Not a title".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 11 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    content".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 4 },
                end: Position { row: 2, column: 13 },
            },
        },
        ScannerToken::VerbatimLabel {
            content: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 3, column: 7 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 3, column: 7 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("VerbatimBlock must start with VerbatimTitle"));
        }
        _ => panic!("Expected AnalysisError for no title"),
    }
}

#[test]
fn test_verbatim_block_no_wall() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "Title".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: "Not a wall".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 10 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    content".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 3, column: 9 },
            },
        },
        ScannerToken::VerbatimLabel {
            content: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 4, column: 0 },
                end: Position { row: 4, column: 7 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 4, column: 7 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("VerbatimBlock must have IndentationWall after VerbatimTitle"));
        }
        _ => panic!("Expected AnalysisError for no wall"),
    }
}

#[test]
fn test_verbatim_block_empty_content() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "Title".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
        ScannerToken::VerbatimLabel {
            content: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 3, column: 7 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 3, column: 7 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span.clone());
    if let Err(e) = &result {
        println!("Error: {:?}", e);
    }
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::VerbatimBlock {
            title,
            wall,
            content,
            label,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            // Check that empty content is handled correctly
            match content.as_ref() {
                SemanticToken::TextSpan {
                    content: content_text,
                    ..
                } => {
                    assert_eq!(content_text, ""); // Empty content for empty verbatim blocks
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }
        }
        _ => panic!(
            "Expected VerbatimBlock semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_verbatim_block_no_label() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "Title".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    content".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 4 },
                end: Position { row: 2, column: 13 },
            },
        },
        ScannerToken::Text {
            content: "Not a label".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 3, column: 11 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 3, column: 11 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("VerbatimBlock must end with VerbatimLabel"));
        }
        _ => panic!("Expected AnalysisError for no label"),
    }
}

#[test]
fn test_verbatim_block_builder() {
    let title_token = SemanticTokenBuilder::text_span(
        "Hello World".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 11 },
        },
    );

    let wall_token = SemanticTokenBuilder::text_span(
        "".to_string(),
        SourceSpan {
            start: Position { row: 2, column: 0 },
            end: Position { row: 2, column: 4 },
        },
    );

    let content_token = SemanticTokenBuilder::text_span(
        "    def hello():\n        print(\"Hello\")\n".to_string(),
        SourceSpan {
            start: Position { row: 2, column: 4 },
            end: Position { row: 4, column: 20 },
        },
    );

    let label_token = SemanticTokenBuilder::text_span(
        "python".to_string(),
        SourceSpan {
            start: Position { row: 5, column: 0 },
            end: Position { row: 5, column: 8 },
        },
    );

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 5, column: 8 },
    };

    let semantic_token = SemanticTokenBuilder::verbatim_block(
        title_token,
        wall_token,
        content_token,
        label_token,
        None,
        span.clone(),
    );

    match semantic_token {
        SemanticToken::VerbatimBlock {
            title,
            wall,
            content,
            label,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            match title.as_ref() {
                SemanticToken::TextSpan {
                    content: title_content,
                    ..
                } => {
                    assert_eq!(title_content, "Hello World");
                }
                _ => panic!("Expected TextSpan title, got {:?}", title.as_ref()),
            }

            match wall.as_ref() {
                SemanticToken::TextSpan {
                    content: wall_content,
                    ..
                } => {
                    assert_eq!(wall_content, "");
                }
                _ => panic!("Expected TextSpan wall, got {:?}", wall.as_ref()),
            }

            match content.as_ref() {
                SemanticToken::TextSpan {
                    content: content_text,
                    ..
                } => {
                    assert_eq!(content_text, "    def hello():\n        print(\"Hello\")\n");
                }
                _ => panic!("Expected TextSpan content, got {:?}", content.as_ref()),
            }

            match label.as_ref() {
                SemanticToken::TextSpan {
                    content: label_content,
                    ..
                } => {
                    assert_eq!(label_content, "python");
                }
                _ => panic!("Expected TextSpan label, got {:?}", label.as_ref()),
            }
        }
        _ => panic!(
            "Expected VerbatimBlock semantic token, got {:?}",
            semantic_token
        ),
    }
}

#[test]
fn test_verbatim_block_span_trait() {
    let title_token = SemanticTokenBuilder::text_span(
        "Title".to_string(),
        SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 5 },
        },
    );

    let wall_token = SemanticTokenBuilder::text_span(
        "".to_string(),
        SourceSpan {
            start: Position { row: 2, column: 0 },
            end: Position { row: 2, column: 4 },
        },
    );

    let content_token = SemanticTokenBuilder::text_span(
        "    content".to_string(),
        SourceSpan {
            start: Position { row: 2, column: 4 },
            end: Position { row: 2, column: 13 },
        },
    );

    let label_token = SemanticTokenBuilder::text_span(
        "label".to_string(),
        SourceSpan {
            start: Position { row: 3, column: 0 },
            end: Position { row: 3, column: 7 },
        },
    );

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 3, column: 7 },
    };

    let semantic_token = SemanticTokenBuilder::verbatim_block(
        title_token,
        wall_token,
        content_token,
        label_token,
        None,
        span.clone(),
    );
    let token_span = semantic_token.span();

    assert_eq!(token_span, &span);
}

#[test]
fn test_verbatim_block_different_wall_types() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = [WallType::InFlow(4), WallType::Stretched];

    for wall_type in test_cases.iter() {
        let tokens = vec![
            ScannerToken::VerbatimTitle {
                content: "Test".to_string(),
                span: SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 4 },
                },
            },
            ScannerToken::IndentationWall {
                level: 4,
                wall_type: wall_type.clone(),
                span: SourceSpan {
                    start: Position { row: 2, column: 0 },
                    end: Position { row: 2, column: 4 },
                },
            },
            ScannerToken::IgnoreTextSpan {
                content: "    content".to_string(),
                span: SourceSpan {
                    start: Position { row: 2, column: 4 },
                    end: Position { row: 2, column: 13 },
                },
            },
            ScannerToken::VerbatimLabel {
                content: "label".to_string(),
                span: SourceSpan {
                    start: Position { row: 3, column: 0 },
                    end: Position { row: 3, column: 7 },
                },
            },
        ];

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 3, column: 7 },
        };

        let result = analyzer.transform_verbatim_block(tokens, span.clone());
        assert!(
            result.is_ok(),
            "Should succeed for wall type: {:?}",
            wall_type
        );

        let semantic_token = result.unwrap();
        match semantic_token {
            SemanticToken::VerbatimBlock {
                title,
                wall,
                content,
                label,
                parameters,
                span: token_span,
            } => {
                assert_eq!(token_span, span);
                assert!(parameters.is_none());

                // All components should be present
                assert!(matches!(title.as_ref(), SemanticToken::TextSpan { .. }));
                assert!(matches!(wall.as_ref(), SemanticToken::TextSpan { .. }));
                assert!(matches!(content.as_ref(), SemanticToken::TextSpan { .. }));
                assert!(matches!(label.as_ref(), SemanticToken::TextSpan { .. }));
            }
            _ => panic!(
                "Expected VerbatimBlock semantic token, got {:?}",
                semantic_token
            ),
        }
    }
}

#[test]
fn test_verbatim_block_empty_title() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimTitle {
            content: "".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::IndentationWall {
            level: 4,
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 4 },
            },
        },
        ScannerToken::IgnoreTextSpan {
            content: "    content".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 4 },
                end: Position { row: 2, column: 13 },
            },
        },
        ScannerToken::VerbatimLabel {
            content: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 3, column: 7 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 3, column: 7 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        SemanticToken::VerbatimBlock {
            title,
            wall,
            content,
            label,
            parameters,
            span: token_span,
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            // Check that empty title is handled correctly
            match title.as_ref() {
                SemanticToken::TextSpan {
                    content: title_content,
                    ..
                } => {
                    assert_eq!(title_content, "");
                }
                _ => panic!("Expected TextSpan title, got {:?}", title.as_ref()),
            }
        }
        _ => panic!(
            "Expected VerbatimBlock semantic token, got {:?}",
            semantic_token
        ),
    }
}
