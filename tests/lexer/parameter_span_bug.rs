//! Test for issue #23: Parameter tokens have incorrect spans
//!
//! Migrated to unified parameter system: Parameters are now represented as
//! Identifier/Text + Equals + Text/QuotedString + Comma tokens at scanner level,
//! which are then assembled into Parameters high-level token.

use txxt::cst::ScannerToken;
use txxt::syntax::Lexer;

// Use the shared test infrastructure from tests/infrastructure/
use crate::infrastructure::parameter_fixtures::{extract_parameters_from_tokens, tokens_contain_parameter};

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

    // Extract parameters using unified system
    let params = extract_parameters_from_tokens(&tokens);

    assert_eq!(params.len(), 2, "Should have 2 parameters");
    assert!(tokens_contain_parameter(&tokens, "key", "value"), "Should have key=value parameter");
    assert!(tokens_contain_parameter(&tokens, "flag", "true"), "Should have flag parameter with implicit true");

    // Verify individual token spans for parameter components
    // Check "key" identifier span
    let key_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { content, .. } if content == "key"))
        .expect("Should find 'key' text token");

    if let ScannerToken::Text { span, .. } = key_token {
        assert_eq!(span.start.column, 8, "key should start at column 8");
        assert_eq!(span.end.column, 11, "key should end at column 11");
    }

    // Check equals sign span
    let equals_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Equals { .. }))
        .expect("Should find equals token");

    if let ScannerToken::Equals { span } = equals_token {
        assert_eq!(span.start.column, 11, "equals should start at column 11");
        assert_eq!(span.end.column, 12, "equals should end at column 12");
    }

    // Check "value" text span
    let value_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { content, .. } if content == "value"))
        .expect("Should find 'value' text token");

    if let ScannerToken::Text { span, .. } = value_token {
        assert_eq!(span.start.column, 12, "value should start at column 12");
        assert_eq!(span.end.column, 17, "value should end at column 17");
    }

    // Check "flag" identifier span
    let flag_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { content, .. } if content == "flag"))
        .expect("Should find 'flag' text token");

    if let ScannerToken::Text { span, .. } = flag_token {
        assert_eq!(span.start.column, 18, "flag should start at column 18");
        assert_eq!(span.end.column, 22, "flag should end at column 22");
    }
}

#[test]
fn test_parameter_spans_in_definition() {
    let input = "term:width=100,height=50 ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Debug: print all tokens
    println!("All tokens for definition test:");
    for token in &tokens {
        println!("  {:?}", token);
    }

    // Extract parameters using unified system
    let params = extract_parameters_from_tokens(&tokens);

    assert_eq!(params.len(), 2, "Should have 2 parameters");
    assert!(tokens_contain_parameter(&tokens, "width", "100"), "Should have width=100 parameter");
    assert!(tokens_contain_parameter(&tokens, "height", "50"), "Should have height=50 parameter");

    // Verify individual token spans for parameter components
    // Check "width" identifier span
    let width_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { content, .. } if content == "width"))
        .expect("Should find 'width' text token");

    if let ScannerToken::Text { span, .. } = width_token {
        assert_eq!(span.start.column, 5, "width should start at column 5");
        assert_eq!(span.end.column, 10, "width should end at column 10");
    }

    // Check "100" value span
    let value_100_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { content, .. } if content == "100"))
        .expect("Should find '100' text token");

    if let ScannerToken::Text { span, .. } = value_100_token {
        assert_eq!(span.start.column, 11, "100 should start at column 11");
        assert_eq!(span.end.column, 14, "100 should end at column 14");
    }

    // Check "height" identifier span
    let height_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { content, .. } if content == "height"))
        .expect("Should find 'height' text token");

    if let ScannerToken::Text { span, .. } = height_token {
        assert_eq!(span.start.column, 15, "height should start at column 15");
        assert_eq!(span.end.column, 21, "height should end at column 21");
    }

    // Check "50" value span
    let value_50_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { content, .. } if content == "50"))
        .expect("Should find '50' text token");

    if let ScannerToken::Text { span, .. } = value_50_token {
        assert_eq!(span.start.column, 22, "50 should start at column 22");
        assert_eq!(span.end.column, 24, "50 should end at column 24");
    }
}

#[test]
fn test_colon_span_after_label() {
    let input = ":: note: ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Find the colon token (if any)
    let colon_token = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Colon { .. }));

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
