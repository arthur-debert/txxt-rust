//! Debug test to understand underscore tokenization

use txxt::tokenizer::Lexer;

#[test]
fn debug_underscore_annotation() {
    let input = ":: _0 ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}

#[test]
fn debug_underscore_alone() {
    let input = "_0";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}
