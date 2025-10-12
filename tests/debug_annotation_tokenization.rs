//! Debug test to understand current annotation tokenization

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[test]
fn debug_current_annotation_tokenization() {
    let input = ":: warning:version=3.9 :: annotation content";
    println!("Input: {}", input);

    let tokens = tokenize(input);
    println!("\nCurrent tokens:");
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::AnnotationMarker { content, span } => {
                println!(
                    "  {}: AnnotationMarker {{ content: {:?}, span: {:?} }}",
                    i, content, span
                );
            }
            Token::Text { content, span } => {
                println!(
                    "  {}: Text {{ content: {:?}, span: {:?} }}",
                    i, content, span
                );
            }
            Token::Parameter { key, value, span } => {
                println!(
                    "  {}: Parameter {{ key: {:?}, value: {:?}, span: {:?} }}",
                    i, key, value, span
                );
            }
            _ => {
                println!("  {}: {:?}", i, token);
            }
        }
    }
}
