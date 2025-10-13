//! Debug Unicode position handling

use txxt::tokenizer::Lexer;

#[test]
fn debug_emoji_position() {
    let input = "🎉- item";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Input chars: {:?}", input.chars().collect::<Vec<_>>());
    println!("Input char count: {}", input.chars().count());
    println!("Input byte length: {}", input.len());
    println!();

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}

#[test]
fn debug_sequence_marker_at_line_start() {
    let input = "- item with 🎉 emoji";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  Token {}: {:?}", i, token);
    }
}

#[test]
fn debug_text_with_accented() {
    let input = "café text";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Input chars: {:?}", input.chars().collect::<Vec<_>>());
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  Token {}: {:?}", i, token);
    }
}
