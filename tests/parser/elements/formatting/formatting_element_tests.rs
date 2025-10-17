//! Formatting Elements Parser Tests
//!
//! These tests validate the parsing of formatting inline elements
//! (strong, emphasis, code, math) using the TxxtCorpora framework.

use txxt::parser::elements::formatting::*;
use txxt::ast::elements::formatting::inlines::TextTransform;
use txxt::ast::elements::tokens::{Token, SourceSpan, Position};

/// Helper function to create a test source span
fn test_span() -> SourceSpan {
    SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 0, column: 1 },
    }
}

/// Test strong (bold) element parsing
#[test]
fn test_parse_strong_simple() {
    // Create simple test tokens for strong element
    let tokens = vec![
        Token::Text {
            content: "*".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "bold".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "*".to_string(),
            span: test_span(),
        },
    ];

    let result = parse_strong(&tokens);
    assert!(result.is_ok());
    
    if let Ok(TextTransform::Strong(content)) = result {
        assert_eq!(content.len(), 1);
        match &content[0] {
            TextTransform::Identity(text) => {
                assert_eq!(text.content(), "bold");
            }
            _ => panic!("Expected Identity transform in strong content"),
        }
    } else {
        panic!("Expected Strong transform");
    }
}

/// Test emphasis (italic) element parsing
#[test]
fn test_parse_emphasis_simple() {
    // Create simple test tokens for emphasis element
    let tokens = vec![
        Token::Text {
            content: "_".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "italic".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "_".to_string(),
            span: test_span(),
        },
    ];

    let result = parse_emphasis(&tokens);
    assert!(result.is_ok());
    
    if let Ok(TextTransform::Emphasis(content)) = result {
        assert_eq!(content.len(), 1);
        match &content[0] {
            TextTransform::Identity(text) => {
                assert_eq!(text.content(), "italic");
            }
            _ => panic!("Expected Identity transform in emphasis content"),
        }
    } else {
        panic!("Expected Emphasis transform");
    }
}

/// Test code element parsing
#[test]
fn test_parse_code_simple() {
    // Create simple test tokens for code element
    let tokens = vec![
        Token::Text {
            content: "`".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "function_name".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "`".to_string(),
            span: test_span(),
        },
    ];

    let result = parse_code(&tokens);
    assert!(result.is_ok());
    
    if let Ok(TextTransform::Code(text)) = result {
        assert_eq!(text.content(), "function_name");
    } else {
        panic!("Expected Code transform");
    }
}

/// Test math element parsing
#[test]
fn test_parse_math_simple() {
    // Create simple test tokens for math element
    let tokens = vec![
        Token::Text {
            content: "#".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "x = y + 2".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "#".to_string(),
            span: test_span(),
        },
    ];

    let result = parse_math(&tokens);
    assert!(result.is_ok());
    
    if let Ok(TextTransform::Math(text)) = result {
        assert_eq!(text.content(), "x = y + 2");
    } else {
        panic!("Expected Math transform");
    }
}

/// Test pattern recognition for strong elements
#[test]
fn test_is_strong_pattern() {
    let strong_tokens = vec![
        Token::Text {
            content: "*".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "content".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "*".to_string(),
            span: test_span(),
        },
    ];

    assert!(is_strong_pattern(&strong_tokens));

    let not_strong_tokens = vec![
        Token::Text {
            content: "_".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "content".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "_".to_string(),
            span: test_span(),
        },
    ];

    assert!(!is_strong_pattern(&not_strong_tokens));
}

/// Test pattern recognition for emphasis elements
#[test]
fn test_is_emphasis_pattern() {
    let emphasis_tokens = vec![
        Token::Text {
            content: "_".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "content".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "_".to_string(),
            span: test_span(),
        },
    ];

    assert!(is_emphasis_pattern(&emphasis_tokens));

    let not_emphasis_tokens = vec![
        Token::Text {
            content: "*".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "content".to_string(),
            span: test_span(),
        },
        Token::Text {
            content: "*".to_string(),
            span: test_span(),
        },
    ];

    assert!(!is_emphasis_pattern(&not_emphasis_tokens));
}

/// Test error handling for empty content
#[test]
fn test_formatting_empty_content_error() {
    let empty_tokens: Vec<Token> = vec![];

    assert!(parse_strong(&empty_tokens).is_err());
    assert!(parse_emphasis(&empty_tokens).is_err());
    assert!(parse_code(&empty_tokens).is_err());
    assert!(parse_math(&empty_tokens).is_err());
}

/// Test nesting validation for strong elements
#[test]
fn test_strong_nesting_validation() {
    let nested_asterisk_content = vec![
        Token::Text {
            content: "*".to_string(),
            span: test_span(),
        },
    ];

    // Should fail validation due to nested asterisk in content
    assert!(validate_strong_nesting(&nested_asterisk_content).is_err());

    let valid_content_tokens = vec![
        Token::Text {
            content: "content".to_string(),
            span: test_span(),
        },
    ];

    // Should pass validation
    assert!(validate_strong_nesting(&valid_content_tokens).is_ok());
}

/// Test formatting elements integration
#[test]
fn test_parse_formatting_elements_integration() {
    // Test with simple text tokens
    let tokens = vec![
        Token::Text {
            content: "simple text".to_string(),
            span: test_span(),
        },
    ];

    let result = parse_formatting_elements(&tokens);
    assert!(result.is_ok());
    
    let transforms = result.unwrap();
    assert_eq!(transforms.len(), 1);
    
    match &transforms[0] {
        TextTransform::Identity(text) => {
            assert_eq!(text.content(), "simple text");
        }
        _ => panic!("Expected Identity transform"),
    }
}

/// Test formatting inlines wrapper function
#[test]
fn test_parse_formatting_inlines() {
    let tokens = vec![
        Token::Text {
            content: "test content".to_string(),
            span: test_span(),
        },
    ];

    let result = parse_formatting_inlines(&tokens);
    assert!(result.is_ok());
    
    let inlines = result.unwrap();
    assert_eq!(inlines.len(), 1);
    
    // Should be wrapped in Inline::TextLine
    match &inlines[0] {
        txxt::ast::elements::formatting::inlines::Inline::TextLine(transform) => {
            match transform {
                TextTransform::Identity(text) => {
                    assert_eq!(text.content(), "test content");
                }
                _ => panic!("Expected Identity transform"),
            }
        }
        _ => panic!("Expected TextLine inline"),
    }
}

// TODO: Add tests using TxxtCorpora when test cases are defined in specs
// These would load test cases from docs/specs/elements/formatting/formatting.txxt

/// Placeholder test for future TxxtCorpora integration
#[test]
#[ignore] // Ignore until test corpus is defined
fn test_formatting_with_corpora() {
    // This test would use TxxtCorpora to load test cases from the specification
    // Example:
    // let corpus = TxxtCorpora::load("txxt.core.spec.formatting.strong.simple").unwrap();
    // let result = parse_strong_from_text(&corpus.source_text);
    // assert!(result.is_ok());
}