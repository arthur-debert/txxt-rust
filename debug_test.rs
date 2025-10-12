use txxt::tokenizer::tokenize;

fn main() {
    let input = ":: warning:severity=high :: Critical issue";
    let tokens = tokenize(input);
    
    println!("Input: {}", input);
    println!("Tokens ({}):", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    // Check for parameters specifically
    let param_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            if let txxt::ast::tokens::Token::Parameter { key, value, .. } = token {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
        .collect();
    
    println!("Parameter tokens found: {:?}", param_tokens);
}