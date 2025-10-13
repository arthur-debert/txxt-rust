//! Test documenting whitespace loss in tokenizer
//!
//! The tokenizer drops whitespace between tokens, making exact
//! source reconstruction impossible.

use txxt::tokenizer::tokenize;

#[test]
#[ignore = "Known bug: Tokenizer drops whitespace between tokens"]
fn test_whitespace_preservation() {
    let test_cases = vec![
        ("a b", "ab"),           // Space between words is lost
        ("(1) Item", "(1)Item"), // Space after paren is lost  
        (":: note ::", "::note::"), // Spaces around labels are lost
        ("a  b", "ab"),          // Multiple spaces lost
        ("a\tb", "ab"),          // Tabs lost
    ];
    
    for (input, expected_broken) in test_cases {
        let tokens = tokenize(input);
        
        // Reconstruct by concatenating token content
        let mut reconstructed = String::new();
        for token in &tokens {
            if let Some(content) = get_token_content(&token) {
                reconstructed.push_str(content);
            }
        }
        
        println!("Input: {:?} → Reconstructed: {:?}", input, reconstructed);
        
        // This assertion documents the bug
        assert_eq!(reconstructed, expected_broken,
            "Whitespace should be preserved but isn't");
            
        // What we want:
        assert_ne!(reconstructed, input,
            "Reconstructed text should match input but doesn't due to whitespace loss");
    }
}

fn get_token_content(token: &txxt::ast::tokens::Token) -> Option<&str> {
    use txxt::ast::tokens::Token::*;
    match token {
        Text { content, .. } => Some(content),
        AnnotationMarker { content, .. } => Some(content),
        DefinitionMarker { content, .. } => Some(content),
        LeftParen { .. } => Some("("),
        RightParen { .. } => Some(")"),
        _ => None,
    }
}