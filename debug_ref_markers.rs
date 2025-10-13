use txxt::tokenizer::Lexer;
use txxt::ast::tokens::Token;

fn main() {
    let test_cases = vec![
        "café [ref]",
        "café@cite", 
        "🎉 [ref]",
        "🎉@cite",
    ];

    for input in test_cases {
        println!("\n=== Input: {:?} ===", input);
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        println!("Tokens:");
        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::Text { content, span } => {
                    println!("  [{}] Text: {:?} @ col {}-{}", i, content, span.start.column, span.end.column);
                }
                Token::LeftBracket { span } => {
                    println!("  [{}] LeftBracket @ col {}-{}", i, span.start.column, span.end.column);
                }
                Token::RightBracket { span } => {
                    println!("  [{}] RightBracket @ col {}-{}", i, span.start.column, span.end.column);
                }
                Token::RefMarker { content, span } => {
                    println!("  [{}] RefMarker: {:?} @ col {}-{}", i, content, span.start.column, span.end.column);
                }
                Token::AtSign { span } => {
                    println!("  [{}] AtSign @ col {}-{}", i, span.start.column, span.end.column);
                }
                Token::Whitespace { content, span } => {
                    println!("  [{}] Whitespace {:?} @ col {}-{}", i, content, span.start.column, span.end.column);
                }
                Token::Eof { .. } => {},
                _ => {
                    println!("  [{}] Other: {:?}", i, token);
                }
            }
        }
        
        // Check what the test is looking for
        let has_left_bracket = tokens.iter().any(|t| matches!(t, Token::LeftBracket { .. }));
        let has_at_sign = tokens.iter().any(|t| matches!(t, Token::AtSign { .. }));
        println!("Has LeftBracket: {}", has_left_bracket);
        println!("Has AtSign: {}", has_at_sign);
    }
}