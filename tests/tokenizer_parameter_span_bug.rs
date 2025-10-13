//! Minimal test case demonstrating parameter token span bug
//!
//! Parameter tokens are created with zero-width spans at the end of input,
//! making it impossible to know where in the source text they came from.

use txxt::tokenizer::tokenize;

#[test]
#[ignore = "Known bug: Parameter tokens have zero-width spans"]
fn test_parameter_token_spans() {
    let input = ":: note:id=123 ::";
    let tokens = tokenize(input);
    
    // Find the parameter token
    let param_token = tokens.iter().find(|t| {
        matches!(t, txxt::ast::tokens::Token::Parameter { .. })
    }).expect("Should have a parameter token");
    
    let span = param_token.span();
    
    // The parameter "id=123" is at columns 8-13 (after "note:")
    // But the span incorrectly shows column 17 (end of input)
    assert_ne!(span.start.column, span.end.column, 
        "Parameter token should not have zero-width span");
    
    // The span should cover the actual text "id=123"
    assert!(span.end.column > span.start.column + 5,
        "Span should be at least 6 characters wide for 'id=123'");
}

#[test]
#[ignore = "Known bug: Synthetic colon token has zero-width span at position 0"]
fn test_synthetic_colon_token_span() {
    let input = ":: note:id=123 ::";
    let tokens = tokenize(input);
    
    // Find the colon token
    let colon_token = tokens.iter().find(|t| {
        matches!(t, txxt::ast::tokens::Token::Colon { .. })
    }).expect("Should have a colon token");
    
    let span = colon_token.span();
    
    // The colon is synthetically created but has position 0,0
    assert_ne!(span.start.column, 0,
        "Colon token should not be at position 0");
    
    // It should be positioned after "note" at column 7
    assert_eq!(span.start.column, 7,
        "Colon should be at column 7 (after 'note')");
}