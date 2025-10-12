//! Simple test for annotation parameter integration

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[test]
fn test_simple_annotation_parameter() {
    let input = ":: warning:severity=high :: Critical issue";
    let tokens = tokenize(input);

    // Find parameter tokens
    let param_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let Token::Parameter { key, value, .. } = token {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(param_tokens.len(), 1);
    assert_eq!(param_tokens[0], ("severity", "high"));

    // Find clean label
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let Token::Text { content, .. } = token {
                Some(content.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(text_tokens.contains(&"warning"));
}

#[test]
fn test_simple_definition_parameter() {
    let input = "API:version=2.0 ::\n    Application Programming Interface";
    let tokens = tokenize(input);

    // Find parameter tokens
    let param_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let Token::Parameter { key, value, .. } = token {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(param_tokens.len(), 1);
    assert_eq!(param_tokens[0], ("version", "2.0"));

    // Find clean term
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let Token::Text { content, .. } = token {
                Some(content.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(text_tokens.contains(&"API"));
}
