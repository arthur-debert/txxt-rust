#![allow(deprecated)]

use txxt::ast::elements::formatting::inlines::TextTransform;
use txxt::cst::{Position, ScannerToken, SourceSpan};
use txxt::semantic::elements::formatting::parse_formatting_elements;

fn create_bold_delimiter(start: usize, end: usize) -> ScannerToken {
    ScannerToken::BoldDelimiter {
        span: SourceSpan {
            start: Position {
                row: 0,
                column: start,
            },
            end: Position {
                row: 0,
                column: end,
            },
        },
    }
}

fn create_italic_delimiter(start: usize, end: usize) -> ScannerToken {
    ScannerToken::ItalicDelimiter {
        span: SourceSpan {
            start: Position {
                row: 0,
                column: start,
            },
            end: Position {
                row: 0,
                column: end,
            },
        },
    }
}

fn create_code_delimiter(start: usize, end: usize) -> ScannerToken {
    ScannerToken::CodeDelimiter {
        span: SourceSpan {
            start: Position {
                row: 0,
                column: start,
            },
            end: Position {
                row: 0,
                column: end,
            },
        },
    }
}

fn create_text(content: &str, start: usize, end: usize) -> ScannerToken {
    ScannerToken::Text {
        content: content.to_string(),
        span: SourceSpan {
            start: Position {
                row: 0,
                column: start,
            },
            end: Position {
                row: 0,
                column: end,
            },
        },
    }
}

#[test]
fn test_simple_bold() {
    let tokens = vec![
        create_bold_delimiter(0, 1),
        create_text("hello", 1, 6),
        create_bold_delimiter(6, 7),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();

    // Verify structure and content
    assert_eq!(result.len(), 1);
    if let TextTransform::Strong(inner) = &result[0] {
        assert_eq!(inner.len(), 1);
        if let TextTransform::Identity(text) = &inner[0] {
            assert_eq!(text.content(), "hello");
        } else {
            panic!("Expected Identity transform");
        }
    } else {
        panic!("Expected Strong transform");
    }
}

#[test]
fn test_simple_italic() {
    let tokens = vec![
        create_italic_delimiter(0, 1),
        create_text("hello", 1, 6),
        create_italic_delimiter(6, 7),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();

    // Verify structure and content
    assert_eq!(result.len(), 1);
    if let TextTransform::Emphasis(inner) = &result[0] {
        assert_eq!(inner.len(), 1);
        if let TextTransform::Identity(text) = &inner[0] {
            assert_eq!(text.content(), "hello");
            // Verify tokens preserve original positions
            assert_eq!(text.tokens.tokens.len(), 1);
        } else {
            panic!("Expected Identity transform");
        }
    } else {
        panic!("Expected Emphasis transform");
    }
}

#[test]
fn test_simple_code() {
    let tokens = vec![
        create_code_delimiter(0, 1),
        create_text("hello", 1, 6),
        create_code_delimiter(6, 7),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();

    // Verify structure and content
    assert_eq!(result.len(), 1);
    if let TextTransform::Code(text) = &result[0] {
        assert_eq!(text.content(), "hello");
    } else {
        panic!("Expected Code transform");
    }
}

#[test]
fn test_nested_bold_italic() {
    let tokens = vec![
        create_bold_delimiter(0, 1),
        create_italic_delimiter(1, 2),
        create_text("hello", 2, 7),
        create_italic_delimiter(7, 8),
        create_bold_delimiter(8, 9),
    ];
    let result = parse_formatting_elements(&tokens).unwrap();

    // Verify nested structure and content
    assert_eq!(result.len(), 1);
    if let TextTransform::Strong(bold_inner) = &result[0] {
        assert_eq!(bold_inner.len(), 1);
        if let TextTransform::Emphasis(italic_inner) = &bold_inner[0] {
            assert_eq!(italic_inner.len(), 1);
            if let TextTransform::Identity(text) = &italic_inner[0] {
                assert_eq!(text.content(), "hello");
            } else {
                panic!("Expected Identity transform");
            }
        } else {
            panic!("Expected Emphasis transform");
        }
    } else {
        panic!("Expected Strong transform");
    }
}
