//! Tests for VerbatimBlock V2 semantic token transformation (Issue #132)
//!
//! This module tests the transformation of NEW scanner tokens
//! (VerbatimBlockStart, VerbatimContentLine, VerbatimBlockEnd) into
//! VerbatimBlock semantic tokens.

use txxt::cst::high_level_tokens::HighLevelToken;
use txxt::cst::{Position, ScannerToken, SourceSpan, WallType};
use txxt::syntax::semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

#[test]
fn test_verbatim_block_v2_simple_in_flow() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Code example".to_string(),
            wall_type: WallType::InFlow(0),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "print('hello')".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "python".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 3, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 3, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span.clone());
    assert!(result.is_ok(), "Should successfully transform v2 tokens");

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::VerbatimBlock {
            title,
            content,
            label,
            parameters,
            wall_type,
            ..
        } => {
            // Check title
            match title.as_ref() {
                HighLevelToken::TextSpan {
                    content: title_content,
                    ..
                } => {
                    assert_eq!(title_content, "Code example");
                }
                _ => panic!("Expected TextSpan title"),
            }

            // Check content (Architecture fix: content is Vec<HighLevelToken>)
            assert_eq!(content.len(), 1, "Should have 1 content line");
            match &content[0] {
                HighLevelToken::IgnoreLine {
                    content: line_content,
                    ..
                } => {
                    // IgnoreLine contains full indentation + content
                    assert_eq!(line_content, "    print('hello')");
                }
                _ => panic!("Expected IgnoreLine high-level token"),
            }

            // Check label (now Label token from unified parser)
            match label.as_ref() {
                HighLevelToken::Label { text, .. } => {
                    assert_eq!(text, "python");
                }
                _ => panic!("Expected Label token"),
            }

            assert!(parameters.is_none());
            assert_eq!(wall_type, WallType::InFlow(0));
        }
        _ => panic!("Expected VerbatimBlock token"),
    }
}

#[test]
fn test_verbatim_block_v2_multiple_content_lines() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Multi-line code".to_string(),
            wall_type: WallType::InFlow(0),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "def hello():".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "print('hi')".to_string(),
            indentation: "        ".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 3, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "python".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 4, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 4, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::VerbatimBlock { content, .. } => {
            // Content is now Vec<HighLevelToken>
            assert_eq!(content.len(), 2, "Should have 2 content lines");

            // First line
            match &content[0] {
                HighLevelToken::IgnoreLine {
                    content: line_content,
                    ..
                } => {
                    assert_eq!(line_content, "    def hello():");
                }
                _ => panic!("Expected IgnoreLine high-level token"),
            }

            // Second line
            match &content[1] {
                HighLevelToken::IgnoreLine {
                    content: line_content,
                    ..
                } => {
                    assert_eq!(line_content, "        print('hi')");
                }
                _ => panic!("Expected IgnoreLine high-level token"),
            }
        }
        _ => panic!("Expected VerbatimBlock token"),
    }
}

#[test]
fn test_verbatim_block_v2_with_blank_lines() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Code with blanks".to_string(),
            wall_type: WallType::InFlow(0),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "line1".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::BlankLine {
            whitespace: "".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 3, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "line3".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 4, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "code".to_string(),
            span: SourceSpan {
                start: Position { row: 4, column: 0 },
                end: Position { row: 5, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 5, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::VerbatimBlock { content, .. } => {
            // Content is now Vec<HighLevelToken>
            assert_eq!(content.len(), 3, "Should have 3 high-level tokens");

            // First line
            match &content[0] {
                HighLevelToken::IgnoreLine {
                    content: line_content,
                    ..
                } => {
                    assert_eq!(line_content, "    line1");
                }
                _ => panic!("Expected IgnoreLine high-level token"),
            }

            // Blank line
            assert!(
                matches!(&content[1], HighLevelToken::BlankLine { .. }),
                "Expected BlankLine high-level token"
            );

            // Third line
            match &content[2] {
                HighLevelToken::IgnoreLine {
                    content: line_content,
                    ..
                } => {
                    assert_eq!(line_content, "    line3");
                }
                _ => panic!("Expected IgnoreLine high-level token"),
            }
        }
        _ => panic!("Expected VerbatimBlock token"),
    }
}

#[test]
fn test_verbatim_block_v2_empty_block() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Empty block".to_string(),
            wall_type: WallType::InFlow(0),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "empty".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 2, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::VerbatimBlock { content, .. } => {
            // Empty block should have no content lines
            assert_eq!(
                content.len(),
                0,
                "Empty block should have empty content Vec"
            );
        }
        _ => panic!("Expected VerbatimBlock token"),
    }
}

#[test]
fn test_verbatim_block_v2_with_parameters() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Code".to_string(),
            wall_type: WallType::InFlow(0),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "x = 1".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            // New grammar (issue #139): whitespace separator instead of colon
            label_raw: "python version=3.11,style=pep8".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 3, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 3, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::VerbatimBlock {
            label, parameters, ..
        } => {
            // Label should be just "python" (now Label token from unified parser)
            match label.as_ref() {
                HighLevelToken::Label { text, .. } => {
                    assert_eq!(text, "python");
                }
                _ => panic!("Expected Label token"),
            }

            // Parameters should be parsed into individual key-value pairs (Phase 4.1)
            assert!(parameters.is_some());
            match parameters.as_ref().unwrap().as_ref() {
                HighLevelToken::Parameters { params, .. } => {
                    // After Phase 4.1, parameters are parsed using scan_parameter_string()
                    // which extracts individual key-value pairs
                    assert_eq!(params.len(), 2, "Should have 2 parameters");
                    assert_eq!(params.get("version").unwrap(), "3.11");
                    assert_eq!(params.get("style").unwrap(), "pep8");
                }
                _ => panic!("Expected Parameters"),
            }
        }
        _ => panic!("Expected VerbatimBlock token"),
    }
}

#[test]
fn test_verbatim_block_v2_stretched_mode() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Title".to_string(),
            wall_type: WallType::Stretched,
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "content at column 0".to_string(),
            indentation: "".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 3, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 3, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::VerbatimBlock {
            content, wall_type, ..
        } => {
            assert_eq!(wall_type, WallType::Stretched);

            // Content is now Vec<HighLevelToken>
            assert_eq!(content.len(), 1, "Should have 1 content line");
            match &content[0] {
                HighLevelToken::IgnoreLine {
                    content: line_content,
                    ..
                } => {
                    // Stretched mode has no indentation in the content
                    assert_eq!(line_content, "content at column 0");
                }
                _ => panic!("Expected IgnoreLine high-level token"),
            }
        }
        _ => panic!("Expected VerbatimBlock token"),
    }
}

#[test]
fn test_verbatim_block_v2_indented_block() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Indented".to_string(),
            wall_type: WallType::InFlow(4),
            span: SourceSpan {
                start: Position { row: 0, column: 4 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "content".to_string(),
            indentation: "        ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 4 },
                end: Position { row: 3, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 4 },
        end: Position { row: 3, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_ok());

    let semantic_token = result.unwrap();
    match semantic_token {
        HighLevelToken::VerbatimBlock { wall_type, .. } => {
            assert_eq!(wall_type, WallType::InFlow(4));
        }
        _ => panic!("Expected VerbatimBlock token"),
    }
}

#[test]
fn test_verbatim_block_v2_error_no_start() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimContentLine {
            content: "content".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 2, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("must start with VerbatimBlockStart"));
        }
        _ => panic!("Expected AnalysisError"),
    }
}

#[test]
fn test_verbatim_block_v2_error_no_end() {
    let analyzer = SemanticAnalyzer::new();

    let tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Title".to_string(),
            wall_type: WallType::InFlow(0),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "content".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
    ];

    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 2, column: 0 },
    };

    let result = analyzer.transform_verbatim_block(tokens, span);
    assert!(result.is_err());

    match result.unwrap_err() {
        SemanticAnalysisError::AnalysisError(msg) => {
            assert!(msg.contains("must end with VerbatimBlockEnd"));
        }
        _ => panic!("Expected AnalysisError"),
    }
}

#[test]
fn test_verbatim_block_v2_integration_with_analyzer() {
    // Test full pipeline from scanner tokens to high-level tokens
    let analyzer = SemanticAnalyzer::new();

    let scanner_tokens = vec![
        ScannerToken::VerbatimBlockStart {
            title: "Example".to_string(),
            wall_type: WallType::InFlow(0),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 1, column: 0 },
            },
        },
        ScannerToken::VerbatimContentLine {
            content: "code here".to_string(),
            indentation: "    ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 2, column: 0 },
            },
        },
        ScannerToken::VerbatimBlockEnd {
            label_raw: "label".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 3, column: 0 },
            },
        },
        ScannerToken::Eof {
            span: SourceSpan {
                start: Position { row: 3, column: 0 },
                end: Position { row: 3, column: 0 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let high_level_tokens = result.unwrap();
    assert_eq!(high_level_tokens.tokens.len(), 1);

    match &high_level_tokens.tokens[0] {
        HighLevelToken::VerbatimBlock { title, .. } => match title.as_ref() {
            HighLevelToken::TextSpan {
                content: title_content,
                ..
            } => {
                assert_eq!(title_content, "Example");
            }
            _ => panic!("Expected TextSpan title"),
        },
        _ => panic!("Expected VerbatimBlock token"),
    }
}
