//! Debug test for quoted annotation parameters

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[test]
fn debug_quoted_annotation_parameters() {
    let input = r#":: info:title="My Document",author="Jane Doe" :: content"#;
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

    // Extract parameters for inspection
    let param_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let Token::Parameter { key, value, .. } = token {
                Some((key.clone(), value.clone()))
            } else {
                None
            }
        })
        .collect();

    println!("\nExtracted parameters: {:?}", param_tokens);
}
