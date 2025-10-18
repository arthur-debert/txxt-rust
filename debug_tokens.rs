use txxt::lexer::Lexer;

fn main() {
    let input = ":: warning:severity=high :: Critical security information";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    
    println!("Input: {}", input);
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {:2}: {:?}", i, token);
    }
}
