//! Verify that the tokenizer actually handles Unicode correctly

use txxt::tokenizer::Lexer;

#[test]
fn verify_lexer_counts_characters_not_bytes() {
    // This test proves that column positions are character-based, not byte-based
    let input = "café"; // 4 characters, but 5 bytes (é is 2 bytes)
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let text = tokens
        .iter()
        .find(|t| matches!(t, txxt::ast::tokens::Token::Text { .. }))
        .unwrap();
    match text {
        txxt::ast::tokens::Token::Text { span, content } => {
            assert_eq!(content, "café");
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 4, "Should be 4 characters, not 5 bytes");
        }
        _ => unreachable!(),
    }
}

#[test]
fn verify_sequence_marker_after_unicode() {
    // Put sequence marker at start of line to be recognized
    let input = "- café item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Find the text token for "café"
    let text = tokens
        .iter()
        .find(|t| {
            if let txxt::ast::tokens::Token::Text { content, .. } = t {
                content == "café"
            } else {
                false
            }
        })
        .unwrap();

    match text {
        txxt::ast::tokens::Token::Text { span, .. } => {
            assert_eq!(
                span.start.column, 2,
                "café should start after '- ' at column 2"
            );
            assert_eq!(
                span.end.column, 6,
                "café should end at column 6 (4 chars from column 2)"
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn verify_the_real_bug_is_in_parameters() {
    // The REAL bug is that parameters create tokens at position (0,0)
    let input = ":: label:key=value ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("\nTokens for parameter test:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  Token {}: {:?}", i, token);
    }

    // Parameters should be there but with wrong position
    let param = tokens
        .iter()
        .find(|t| matches!(t, txxt::ast::tokens::Token::Parameter { .. }));

    if let Some(txxt::ast::tokens::Token::Parameter { span, key, value }) = param {
        println!(
            "Parameter '{}={}' span: start=({},{}), end=({},{})",
            key, value, span.start.row, span.start.column, span.end.row, span.end.column
        );
        // The bug is that the span has zero width!
        assert_eq!(
            span.end.column - span.start.column,
            0,
            "This is a bug - parameter span has zero width"
        );
    }
}

#[test]
fn verify_sequence_marker_roman_numeral_calculation() {
    // Test that Roman numerals like "xiii." work correctly
    // "xiii." is 5 characters, so end should be start + 5
    let input = "xiii. test";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let marker = tokens
        .iter()
        .find(|t| matches!(t, txxt::ast::tokens::Token::SequenceMarker { .. }))
        .expect("Should find sequence marker");

    match marker {
        txxt::ast::tokens::Token::SequenceMarker { span, marker_type } => {
            println!("Roman marker type: {:?}", marker_type);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 5, "xiii. is 5 characters");
            // Verify the arithmetic is correct: 0 + "xiii.".len() = 0 + 5 = 5 ✓
        }
        _ => unreachable!(),
    }
}
