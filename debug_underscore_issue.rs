use txxt::tokenizer::Lexer;
use txxt::ast::tokens::Token;

fn main() {
    // Focus on underscore cases which seem most problematic
    let test_cases = vec![
        "🎉_italic_",      // Expected: "🎉", "_", "italic", "_"  / Actual: "🎉_italic", "_"
        "café_italic_",    // Expected: "café", "_", "italic", "_"  / Actual: "café_italic", "_" 
        "🎉@citation",     // Expected: "🎉@citation" / Actual: "🎉", ???
        "café@citation",   // Expected: "café@citation" / Actual: ???
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
                Token::ItalicDelimiter { span } => {
                    println!("  [{}] ItalicDelimiter @ col {}-{}", i, span.start.column, span.end.column);
                }
                Token::AtSign { span } => {
                    println!("  [{}] AtSign @ col {}-{}", i, span.start.column, span.end.column);
                }
                Token::Eof { .. } => {},
                _ => {
                    println!("  [{}] Other token: {:?}", i, token);
                }
            }
        }
    }
    
    // Let's also trace the character-by-character logic for "🎉_italic_"
    println!("\n=== Character analysis for \"🎉_italic_\" ===");
    let input = "🎉_italic_";
    for (i, ch) in input.chars().enumerate() {
        println!("  [{}] '{}' U+{:04X} is_whitespace={} is_alphanumeric={}", 
                 i, ch, ch as u32, ch.is_whitespace(), ch.is_alphanumeric());
    }
}