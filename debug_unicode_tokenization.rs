use txxt::tokenizer::Lexer;
use txxt::ast::tokens::Token;

fn main() {
    let test_cases = vec![
        "🎉*bold*",
        "🎉_italic_",
        "🎉[ref]",
        "café [ref]",
        "café*bold*",
        "café_italic_",
        "🎉 *bold*",  // with space
        "🎉 _italic_", // with space
        "🎉 [ref]",   // with space
    ];

    for input in test_cases {
        println!("\n=== Input: {:?} ===", input);
        println!("Input length: {} bytes, {} chars", input.len(), input.chars().count());
        
        // Show character breakdown
        print!("Characters: ");
        for (i, ch) in input.chars().enumerate() {
            print!("[{}:'{}' U+{:04X}] ", i, ch, ch as u32);
        }
        println!();
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        println!("\nTokens:");
        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::Text { content, span } => {
                    println!("  [{}] Text: {:?} @ col {}-{}", i, content, span.start.column, span.end.column);
                }
                Token::BoldDelimiter { span } => {
                    println!("  [{}] BoldDelimiter @ col {}-{}", i, span.start.column, span.end.column);
                }
                Token::ItalicDelimiter { span } => {
                    println!("  [{}] ItalicDelimiter @ col {}-{}", i, span.start.column, span.end.column);
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
                Token::Eof { span } => {
                    println!("  [{}] Eof @ col {}", i, span.start.column);
                }
                _ => {
                    println!("  [{}] Other token: {:?}", i, token);
                }
            }
        }
    }
}