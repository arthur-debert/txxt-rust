//! Debug test to understand current tokenizer behavior with whitespace

use txxt::syntax::Lexer;

#[test]
fn debug_simple_whitespace() {
    let input = "hello world";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}

#[test]
fn debug_multiple_spaces() {
    let input = "hello    world";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}

#[test]
fn debug_tab() {
    let input = "hello\tworld";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}

#[test]
fn debug_annotation() {
    let input = ":: note ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}
