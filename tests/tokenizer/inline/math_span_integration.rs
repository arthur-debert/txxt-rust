//! Integration tests for math span tokenization with main lexer

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[test]
fn test_math_span_integration_simple() {
    let tokens = tokenize("#formula#");

    // Should have: MathSpan, Eof
    assert_eq!(tokens.len(), 2);

    match &tokens[0] {
        Token::MathSpan { content, span } => {
            assert_eq!(content, "formula");
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 9);
        }
        _ => panic!("Expected MathSpan token, got {:?}", tokens[0]),
    }

    match &tokens[1] {
        Token::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[1]),
    }
}

#[test]
fn test_math_span_integration_with_text() {
    let tokens = tokenize("The formula #E=mc^2# is famous");

    // Should have: Text, MathSpan, Text, Eof
    assert!(tokens.len() >= 4);

    // Find the math span
    let math_span = tokens
        .iter()
        .find(|token| matches!(token, Token::MathSpan { .. }))
        .expect("Should find MathSpan token");

    match math_span {
        Token::MathSpan { content, .. } => {
            assert_eq!(content, "E=mc^2");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_math_span_vs_math_delimiter() {
    let tokens = tokenize("#formula#");

    // Should produce MathSpan, not separate MathDelimiter tokens
    let math_delimiters: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::MathDelimiter { .. }))
        .collect();

    let math_spans: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::MathSpan { .. }))
        .collect();

    assert_eq!(
        math_delimiters.len(),
        0,
        "Should not have MathDelimiter tokens"
    );
    assert_eq!(
        math_spans.len(),
        1,
        "Should have exactly one MathSpan token"
    );
}

#[test]
fn test_math_span_adjacent() {
    let tokens = tokenize("#a##b#");

    // Should parse as: MathSpan("a"), MathSpan("b"), Eof
    let math_spans: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::MathSpan { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(math_spans, vec!["a", "b"]);
}

#[test]
fn test_incomplete_math_span_fallback() {
    let tokens = tokenize("#incomplete");

    // Should not produce MathSpan, should produce MathDelimiter + Text
    let has_math_span = tokens
        .iter()
        .any(|token| matches!(token, Token::MathSpan { .. }));

    assert!(
        !has_math_span,
        "Incomplete math span should not produce MathSpan token"
    );

    // Should have MathDelimiter for the standalone #
    let has_math_delimiter = tokens
        .iter()
        .any(|token| matches!(token, Token::MathDelimiter { .. }));

    assert!(
        has_math_delimiter,
        "Should produce MathDelimiter for standalone #"
    );
}
