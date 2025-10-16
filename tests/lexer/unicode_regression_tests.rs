//! Regression tests for Unicode tokenization issues
//!
//! These tests ensure that Unicode characters (including emoji) are properly
//! tokenized and that column positions are calculated correctly.

use txxt::ast::tokens::Token;
use txxt::lexer::Lexer;

#[test]
fn test_emoji_text_tokenization() {
    // When emoji and text are adjacent without space, they form one text token
    let input = "🎉text";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    assert_eq!(tokens.len(), 2); // Text + Eof
    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, "🎉text");
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 5); // emoji(1) + text(4)
        }
        _ => panic!("Expected Text token"),
    }
}

#[test]
fn test_emoji_with_space_tokenization() {
    // When emoji and text are separated by space, they form separate tokens
    let input = "🎉 text";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    assert_eq!(tokens.len(), 4); // Text + Whitespace + Text + Eof

    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, "🎉");
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 1);
        }
        _ => panic!("Expected Text token for emoji"),
    }

    match &tokens[2] {
        Token::Text { content, span } => {
            assert_eq!(content, "text");
            assert_eq!(span.start.column, 2);
            assert_eq!(span.end.column, 6);
        }
        _ => panic!("Expected Text token for text"),
    }
}

#[test]
fn test_emoji_dash_tokenization_fix() {
    // REGRESSION FIX: Dash after emoji should be tokenized
    let input = "🎉- item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // After fix: dash should be present
    let has_dash = tokens.iter().any(|t| matches!(t, Token::Dash { .. }));
    let has_sequence_marker = tokens
        .iter()
        .any(|t| matches!(t, Token::SequenceMarker { .. }));

    // Either dash or sequence marker should be present
    assert!(
        has_dash || has_sequence_marker,
        "Dash after emoji should be tokenized"
    );

    // Verify the dash is at the correct position
    let dash_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t, Token::Dash { .. }))
        .collect();

    if !dash_tokens.is_empty() {
        match &dash_tokens[0] {
            Token::Dash { span } => {
                assert_eq!(
                    span.start.column, 1,
                    "Dash should start at column 1 after emoji"
                );
                assert_eq!(span.end.column, 2, "Dash should end at column 2");
            }
            _ => unreachable!(),
        }
    }

    // Should have: Text(emoji) + Dash + Whitespace + Text(item) + Eof
    assert_eq!(tokens.len(), 5);
}

#[test]
fn test_sequence_marker_at_line_start() {
    // Sequence markers work correctly at column 0
    let input = "- item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let sequence_markers: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t, Token::SequenceMarker { .. }))
        .collect();

    assert_eq!(sequence_markers.len(), 1);

    match &tokens[0] {
        Token::SequenceMarker { span, .. } => {
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 1);
        }
        _ => panic!("Expected SequenceMarker token"),
    }
}

#[test]
fn test_accented_character_span_calculation() {
    // Test that accented characters have correct column positions
    let input = "café";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, "café");
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 4); // 4 characters, not 5 bytes
        }
        _ => panic!("Expected Text token"),
    }
}

#[test]
fn test_emoji_sequence_marker_column_positions() {
    // When sequence marker appears after emoji, positions should be character-based
    let input = "🎉1. item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Currently: "🎉1" is tokenized as text, period separate, no sequence marker
    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, "🎉1");
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 2); // emoji(1) + digit(1)
        }
        _ => panic!("Expected Text token"),
    }

    // The period is tokenized separately
    match &tokens[1] {
        Token::Period { span } => {
            assert_eq!(span.start.column, 2);
            assert_eq!(span.end.column, 3);
        }
        _ => panic!("Expected Period token"),
    }
}

#[test]
fn test_mixed_unicode_tokenization() {
    // Test various Unicode characters in sequence
    let input = "🎉café→résumé";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // All characters should form a single text token (no spaces)
    match &tokens[0] {
        Token::Text { content, span } => {
            assert_eq!(content, "🎉café→résumé");
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 12); // Count characters, not bytes
        }
        _ => panic!("Expected Text token"),
    }
}

#[test]
fn test_unicode_with_inline_formatting() {
    // Test that inline formatting works with Unicode
    let input = "café *bold* résumé";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Should have: Text(café) + Space + BoldDelimiter + Text(bold) + BoldDelimiter + Space + Text(résumé)
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(text_tokens, vec!["café", "bold", "résumé"]);

    // Check bold delimiters
    let bold_delimiters: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t, Token::BoldDelimiter { .. }))
        .collect();

    assert_eq!(bold_delimiters.len(), 2);
}
