//! Debug test to understand underscore tokenization

use txxt::lexer::Lexer;

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

#[test]
fn debug_italic_text() {
    let inputs = ["_italic text_", "_hello_", "_test_ text", "normal _0 text"];

    for input in inputs {
        println!("\nInput: {:?}", input);
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        println!("Token count: {}", tokens.len());

        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?}", i, token);
        }
        println!("---");
    }
}
