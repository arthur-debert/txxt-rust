//! Test for issue #23: Parameter tokens have incorrect spans
//!
//! The bug: When parameters are parsed from annotations or definitions,
//! the resulting Parameter tokens have zero-width spans at incorrect positions,
//! making it impossible to track where they came from in the source.

use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::Lexer;

#[test]
fn test_parameter_spans_in_annotation() {
    let input = ":: note:key=value,flag ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Debug: print all tokens
    println!("All tokens for annotation test:");
    for token in &tokens {
        println!("  {:?}", token);
    }

    // Find parameter tokens
    let param_tokens: Vec<&ScannerToken> = tokens
        .iter()
        .filter(|t| matches!(t, ScannerToken::Parameter { .. }))
        .collect();

    assert_eq!(param_tokens.len(), 2, "Should have 2 parameter tokens");

    // Check first parameter (key=value)
    if let ScannerToken::Parameter { key, value, span } = param_tokens[0] {
        assert_eq!(key, "key");
        assert_eq!(value, "value");

        // The span should cover "key=value" starting at position 8
        assert_eq!(
            span.start.column, 8,
            "First parameter should start at column 8"
        );
        assert_eq!(
            span.end.column, 17,
            "First parameter should end at column 17"
        );
        assert!(
            span.end.column > span.start.column,
            "Parameter span should have non-zero width"
        );
    }

    // Check second parameter (flag)
    if let ScannerToken::Parameter { key, value, span } = param_tokens[1] {
        assert_eq!(key, "flag");
        assert_eq!(value, "true");

        // The span should cover "flag" starting at position 18
        assert_eq!(
            span.start.column, 18,
            "Second parameter should start at column 18"
        );
        assert_eq!(
            span.end.column, 22,
            "Second parameter should end at column 22"
        );
        assert!(
            span.end.column > span.start.column,
            "Parameter span should have non-zero width"
        );
    }
}

#[test]
fn test_parameter_spans_in_definition() {
    let input = "term:width=100,height=50 ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Find parameter tokens
    let param_tokens: Vec<&ScannerToken> = tokens
        .iter()
        .filter(|t| matches!(t, ScannerToken::Parameter { .. }))
        .collect();

    assert_eq!(param_tokens.len(), 2, "Should have 2 parameter tokens");

    // Check first parameter (width=100)
    if let ScannerToken::Parameter { key, value, span } = param_tokens[0] {
        assert_eq!(key, "width");
        assert_eq!(value, "100");

        // The span should cover "width=100" starting at position 5
        assert_eq!(
            span.start.column, 5,
            "First parameter should start at column 5"
        );
        assert_eq!(
            span.end.column, 14,
            "First parameter should end at column 14"
        );
        assert!(
            span.end.column > span.start.column,
            "Parameter span should have non-zero width"
        );
    }

    // Check second parameter (height=50)
    if let ScannerToken::Parameter { key, value, span } = param_tokens[1] {
        assert_eq!(key, "height");
        assert_eq!(value, "50");

        // The span should cover "height=50" starting at position 15
        assert_eq!(
            span.start.column, 15,
            "Second parameter should start at column 15"
        );
        assert_eq!(
            span.end.column, 24,
            "Second parameter should end at column 24"
        );
        assert!(
            span.end.column > span.start.column,
            "Parameter span should have non-zero width"
        );
    }
}

#[test]
fn test_colon_span_after_label() {
    let input = ":: note: ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Find the colon token (if any)
    let colon_token = tokens.iter().find(|t| matches!(t, ScannerToken::Colon { .. }));

    if let Some(ScannerToken::Colon { span }) = colon_token {
        // The colon should be at position 7
        assert_eq!(span.start.column, 7, "Colon should start at column 7");
        assert_eq!(span.end.column, 8, "Colon should end at column 8");
        assert_eq!(
            span.end.column - span.start.column,
            1,
            "Colon span should have width 1"
        );
    } else {
        // If no Colon token is created, that's also a bug - parameter integration should preserve it
        panic!("No Colon token found - parameter integration may be consuming structural colons");
    }
}
