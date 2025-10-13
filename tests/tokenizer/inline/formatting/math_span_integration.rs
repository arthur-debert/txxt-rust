//! Integration tests for math delimiter tokenization with main lexer

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[test]
fn test_math_delimiter_integration_simple() {
    let tokens = tokenize("#formula#");

    // Should have: MathDelimiter, Text, MathDelimiter, Eof
    assert_eq!(tokens.len(), 4);

    match &tokens[0] {
        Token::MathDelimiter { span } => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 1);
        }
        _ => panic!("Expected MathDelimiter token, got {:?}", tokens[0]),
    }

    match &tokens[1] {
        Token::Text { content, .. } => {
            assert_eq!(content, "formula");
        }
        _ => panic!("Expected Text token, got {:?}", tokens[1]),
    }

    match &tokens[2] {
        Token::MathDelimiter { .. } => {}
        _ => panic!("Expected MathDelimiter token, got {:?}", tokens[2]),
    }

    match &tokens[3] {
        Token::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[3]),
    }
}

#[test]
fn test_math_delimiter_integration_with_text() {
    let tokens = tokenize("The formula #E=mc^2# is famous");

    // Should have: Text, MathDelimiter, Text, MathDelimiter, Text, Eof
    assert!(tokens.len() >= 6);

    // Find the math delimiters and content
    let math_delimiters: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::MathDelimiter { .. }))
        .collect();
    assert_eq!(math_delimiters.len(), 2);

    // Find the text tokens with "E", "mc", "2" (math content is broken into multiple tokens)
    let e_token = tokens
        .iter()
        .find(|token| {
            if let Token::Text { content, .. } = token {
                content == "E"
            } else {
                false
            }
        })
        .expect("Should find 'E' token");

    let mc_token = tokens
        .iter()
        .find(|token| {
            if let Token::Text { content, .. } = token {
                content == "mc"
            } else {
                false
            }
        })
        .expect("Should find 'mc' token");

    let two_token = tokens
        .iter()
        .find(|token| {
            if let Token::Text { content, .. } = token {
                content == "2"
            } else {
                false
            }
        })
        .expect("Should find '2' token");

    // Verify the tokens have correct content
    match e_token {
        Token::Text { content, .. } => assert_eq!(content, "E"),
        _ => unreachable!(),
    }
    match mc_token {
        Token::Text { content, .. } => assert_eq!(content, "mc"),
        _ => unreachable!(),
    }
    match two_token {
        Token::Text { content, .. } => assert_eq!(content, "2"),
        _ => unreachable!(),
    }
}

#[test]
fn test_math_delimiter_consistency() {
    let tokens = tokenize("#formula#");

    // Should now produce MathDelimiter tokens, not MathSpan
    let math_delimiters: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::MathDelimiter { .. }))
        .collect();

    assert_eq!(
        math_delimiters.len(),
        2,
        "Should have exactly two MathDelimiter tokens"
    );
}

#[test]
fn test_math_delimiter_adjacent() {
    let tokens = tokenize("#a##b#");

    // Should parse as: MathDelimiter, Text("a"), MathDelimiter, MathDelimiter, Text("b"), MathDelimiter, Eof
    let math_delimiters: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::MathDelimiter { .. }))
        .collect();

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(math_delimiters.len(), 4); // Four # delimiters
    assert_eq!(text_tokens, vec!["a", "b"]); // Two text content pieces
}

#[test]
fn test_incomplete_math_delimiter_behavior() {
    let tokens = tokenize("#incomplete");

    // Should produce MathDelimiter + Text (standalone behavior)
    let has_math_delimiter = tokens
        .iter()
        .any(|token| matches!(token, Token::MathDelimiter { .. }));

    assert!(
        has_math_delimiter,
        "Should produce MathDelimiter for standalone #"
    );

    let has_text = tokens.iter().any(|token| {
        if let Token::Text { content, .. } = token {
            content == "incomplete"
        } else {
            false
        }
    });

    assert!(has_text, "Should produce Text token for 'incomplete'");
}
