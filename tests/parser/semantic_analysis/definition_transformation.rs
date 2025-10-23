#![allow(deprecated)]
//! Tests for Definition semantic token transformation
//!
//! This module tests the transformation of scanner tokens into
//! Definition semantic tokens after grammar simplification.
//!
//! New syntax: Term: (single colon, no inline parameters)
//! Parameters come from optional trailing annotations in AST construction.

use txxt::cst::high_level_tokens::{HighLevelToken, HighLevelTokenBuilder, HighLevelTokenSpan};
use txxt::cst::{Position, ScannerToken, SourceSpan};
use txxt::syntax::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_definition_basic_transformation() {
    let analyzer = SemanticAnalyzer::new();

    // New syntax: Term: (just text + colon, no TxxtMarker)
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
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 5 },
    };

    let result = analyzer.transform_definition(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::Definition {
            term,
            parameters,
            span: token_span,
            ..
        } => {
            assert_eq!(token_span, span);
            // In new syntax, parameters are always None (come from trailing annotations)
            assert!(parameters.is_none());

            // Check that the term is a TextSpan
            match term.as_ref() {
                HighLevelToken::TextSpan {
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
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position { row: 1, column: 16 },
                end: Position { row: 1, column: 17 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 17 },
    };

    let result = analyzer.transform_definition(tokens, span.clone());
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::Definition {
            term,
            parameters,
            span: token_span,
            ..
        } => {
            assert_eq!(token_span, span);
            assert!(parameters.is_none());

            match term.as_ref() {
                HighLevelToken::TextSpan {
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

    // Only one token - not enough for definition
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

    if let Err(SemanticAnalysisError::AnalysisError(msg)) = result {
        assert!(
            msg.contains("Definition pattern must contain Colon")
                || msg.contains("Definition must have a term")
        );
    } else {
        panic!("Expected AnalysisError");
    }
}

#[test]
fn test_definition_no_closing_marker() {
    let analyzer = SemanticAnalyzer::new();

    // Text but no closing colon
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
    ];

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 5 },
    };

    let result = analyzer.transform_definition(tokens, span);
    assert!(result.is_err());

    if let Err(SemanticAnalysisError::AnalysisError(msg)) = result {
        assert!(msg.contains("Definition pattern must contain Colon"));
    } else {
        panic!("Expected AnalysisError with Colon message");
    }
}

#[test]
fn test_definition_builder() {
    let term_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 6 },
    };
    let term = HighLevelTokenBuilder::text_span("Parser".to_string(), term_span.clone());

    let def_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 7 },
    };
    let definition = HighLevelTokenBuilder::definition(term.clone(), None, def_span.clone());

    match definition {
        HighLevelToken::Definition {
            term: def_term,
            parameters,
            ..
        } => {
            assert_eq!(*def_term, term);
            assert!(parameters.is_none());
        }
        _ => panic!("Expected Definition token"),
    }
}

#[test]
fn test_definition_span_trait() {
    let term_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 4 },
    };
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 10 },
    };

    let term = HighLevelTokenBuilder::text_span("Term".to_string(), term_span);
    let definition = HighLevelTokenBuilder::definition(term, None, span.clone());

    assert_eq!(definition.span(), &span);
}

#[test]
fn test_definition_different_terms() {
    let analyzer = SemanticAnalyzer::new();

    let test_cases = vec![
        ("API", "API"),
        ("REST", "REST"),
        ("HTTP", "HTTP"),
        ("Parser Component", "Parser Component"),
    ];

    for (input, expected) in test_cases {
        let mut tokens = vec![];
        let mut col = 0;

        // Split on whitespace and create tokens
        for (i, word) in input.split_whitespace().enumerate() {
            if i > 0 {
                tokens.push(ScannerToken::Whitespace {
                    content: " ".to_string(),
                    span: SourceSpan {
                        start: Position {
                            row: 1,
                            column: col,
                        },
                        end: Position {
                            row: 1,
                            column: col + 1,
                        },
                    },
                });
                col += 1;
            }

            tokens.push(ScannerToken::Text {
                content: word.to_string(),
                span: SourceSpan {
                    start: Position {
                        row: 1,
                        column: col,
                    },
                    end: Position {
                        row: 1,
                        column: col + word.len(),
                    },
                },
            });
            col += word.len();
        }

        // Add closing colon
        tokens.push(ScannerToken::Colon {
            span: SourceSpan {
                start: Position {
                    row: 1,
                    column: col,
                },
                end: Position {
                    row: 1,
                    column: col + 1,
                },
            },
        });

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position {
                row: 1,
                column: col + 1,
            },
        };

        let result = analyzer.transform_definition(tokens, span);
        assert!(result.is_ok(), "Failed to transform term: {}", input);

        let semantic_token = result.unwrap();
        match semantic_token {
            HighLevelToken::Definition { term, .. } => match term.as_ref() {
                HighLevelToken::TextSpan {
                    content: term_content,
                    ..
                } => {
                    assert_eq!(term_content, expected);
                }
                _ => panic!("Expected TextSpan term"),
            },
            _ => panic!("Expected Definition semantic token"),
        }
    }
}

// NOTE: Tests for inline parameters have been removed as parameters now come
// from optional trailing annotations in AST construction, not from the
// Definition high-level token itself.
