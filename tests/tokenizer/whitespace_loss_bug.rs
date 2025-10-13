//! Test for issue #24: Tokenizer drops all whitespace between tokens
//!
//! The bug: The tokenizer skips spaces and tabs without creating tokens,
//! making it impossible to reconstruct the original text with proper spacing.

use txxt::ast::tokens::Token;
use txxt::tokenizer::Lexer;

#[test]
fn test_whitespace_preservation_in_text() {
    let input = "hello world";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Should have 4 tokens: Text("hello"), Whitespace(" "), Text("world"), Eof
    assert_eq!(
        tokens.len(),
        4,
        "Should have 4 tokens including whitespace and Eof"
    );

    // Reconstruct original by concatenating token content
    let reconstructed: String = tokens
        .iter()
        .map(|t| match t {
            Token::Text { content, .. } => content.as_str(),
            Token::Whitespace { content, .. } => content.as_str(),
            _ => "",
        })
        .collect();

    assert_eq!(
        reconstructed, input,
        "Reconstructed text should match original"
    );
}

#[test]
fn test_whitespace_in_parenthesized_list() {
    let input = "(1) First item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Space after ")" is preserved
    let token_strings: Vec<String> = tokens
        .iter()
        .map(|t| match t {
            Token::LeftParen { .. } => "(".to_string(),
            Token::RightParen { .. } => ")".to_string(),
            Token::Text { content, .. } => content.clone(),
            Token::Whitespace { content, .. } => content.clone(),
            _ => "".to_string(),
        })
        .collect();

    let reconstructed = token_strings.join("");
    assert_eq!(
        reconstructed, input,
        "Should reconstruct original with spacing"
    );

    // Verify space after right paren
    let right_paren_idx = tokens
        .iter()
        .position(|t| matches!(t, Token::RightParen { .. }))
        .expect("Should have RightParen token");

    // Next token should be whitespace
    if let Some(next_token) = tokens.get(right_paren_idx + 1) {
        assert!(
            matches!(next_token, Token::Whitespace { .. }),
            "Token after RightParen should be Whitespace, got {:?}",
            next_token
        );
    }
}

#[test]
fn test_whitespace_in_annotation() {
    let input = ":: note ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Spaces around "note" are preserved
    let reconstructed: String = tokens
        .iter()
        .map(|t| match t {
            Token::AnnotationMarker { content, .. } => content.as_str(),
            Token::Text { content, .. } => content.as_str(),
            Token::Whitespace { content, .. } => content.as_str(),
            _ => "",
        })
        .collect();

    assert_eq!(
        reconstructed, input,
        "Reconstructed annotation should match original"
    );

    // Should have whitespace tokens around "note"
    let has_whitespace = tokens.iter().any(|t| matches!(t, Token::Whitespace { .. }));
    assert!(has_whitespace, "Should have whitespace tokens");
}

#[test]
fn test_multiple_spaces() {
    let input = "hello    world"; // 4 spaces
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Should have 4 tokens: Text, Whitespace(4 spaces), Text, Eof
    assert_eq!(tokens.len(), 4, "Should have 4 tokens including whitespace");

    // Find whitespace token
    let whitespace_token = tokens
        .iter()
        .find(|t| matches!(t, Token::Whitespace { .. }));

    if let Some(Token::Whitespace { content, .. }) = whitespace_token {
        assert_eq!(content.len(), 4, "Should preserve all 4 spaces");
        assert_eq!(content, "    ", "Whitespace content should be 4 spaces");
    } else {
        panic!("No whitespace token found");
    }
}

#[test]
fn test_tab_preservation() {
    let input = "hello\tworld";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Should have 4 tokens: Text, Whitespace(tab), Text, Eof
    assert_eq!(tokens.len(), 4, "Should have 4 tokens including tab");

    // Should have whitespace token with tab
    let whitespace_token = tokens
        .iter()
        .find(|t| matches!(t, Token::Whitespace { .. }));

    if let Some(Token::Whitespace { content, .. }) = whitespace_token {
        assert_eq!(content, "\t", "Should preserve tab character");
    } else {
        panic!("No whitespace token found for tab");
    }
}

#[test]
fn test_whitespace_distinction() {
    let input1 = "a b"; // 1 space
    let input2 = "a  b"; // 2 spaces
    let input3 = "a   b"; // 3 spaces

    let mut lexer1 = Lexer::new(input1);
    let mut lexer2 = Lexer::new(input2);
    let mut lexer3 = Lexer::new(input3);

    let tokens1 = lexer1.tokenize();
    let tokens2 = lexer2.tokenize();
    let tokens3 = lexer3.tokenize();

    // All should have same structure but different whitespace content
    assert_eq!(tokens1.len(), 4, "Should have Text, Whitespace, Text, Eof");
    assert_eq!(tokens2.len(), 4, "Should have Text, Whitespace, Text, Eof");
    assert_eq!(tokens3.len(), 4, "Should have Text, Whitespace, Text, Eof");

    // Extract whitespace content
    let ws1 = tokens1
        .iter()
        .find_map(|t| match t {
            Token::Whitespace { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .unwrap();

    let ws2 = tokens2
        .iter()
        .find_map(|t| match t {
            Token::Whitespace { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .unwrap();

    let ws3 = tokens3
        .iter()
        .find_map(|t| match t {
            Token::Whitespace { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .unwrap();

    // Different amounts of whitespace should be preserved
    assert_eq!(ws1, " ", "Should have 1 space");
    assert_eq!(ws2, "  ", "Should have 2 spaces");
    assert_eq!(ws3, "   ", "Should have 3 spaces");
}
