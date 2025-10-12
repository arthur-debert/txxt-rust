//! Debug test to understand blankline tokenization

#[cfg(test)]
mod tests {
    use txxt::tokenizer::tokenize;

    #[test]
    fn debug_simple_blankline() {
        let input = "\n\n";
        let tokens = tokenize(input);

        println!("Input: {:?}", input);
        println!("Number of tokens: {}", tokens.len());

        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?}", i, token);
        }

        assert!(!tokens.is_empty(), "Should have tokens");
    }

    #[test]
    fn debug_text_with_blankline() {
        let input = "text\n\nmore";
        let tokens = tokenize(input);

        println!("Input: {:?}", input);
        println!("Number of tokens: {}", tokens.len());

        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?}", i, token);
        }

        // Count BlankLine tokens
        let blankline_count = tokens
            .iter()
            .filter(|token| matches!(token, txxt::ast::tokens::Token::BlankLine { .. }))
            .count();
        println!("BlankLine count: {}", blankline_count);

        assert!(!tokens.is_empty(), "Should have tokens");
    }

    #[test]
    fn debug_whitespace_only() {
        let input = " ";
        let tokens = tokenize(input);

        println!("Input: {:?}", input);
        println!("Number of tokens: {}", tokens.len());

        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?}", i, token);
        }

        assert!(!tokens.is_empty(), "Should have tokens");
    }
}
