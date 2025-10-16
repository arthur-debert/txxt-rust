//! Debug test to understand parameter tokenization

use txxt::lexer::Lexer;

#[test]
fn debug_annotation_parameters() {
    let input = ":: note:key=value,flag ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}

#[test]
fn debug_definition_parameters() {
    let input = "term:width=100,height=50 ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Input: {:?}", input);
    println!("Token count: {}", tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
}
