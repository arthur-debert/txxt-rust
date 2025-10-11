//! Debug test for VerbatimStart title content fix
//!
//! This test verifies that VerbatimStart tokens contain only the title content
//! without the trailing colon structural marker.

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_verbatim_title_without_colon() {
        let input = r#"My Code Title:
    print("hello")
()"#;

        println!("Input:\n{}", input);

        let tokens = tokenize(input);

        println!("\nTokens:");
        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::VerbatimStart { content, span } => {
                    println!(
                        "  {}: VerbatimStart {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                Token::VerbatimContent { content, span } => {
                    println!(
                        "  {}: VerbatimContent {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                Token::VerbatimEnd { content, span } => {
                    println!(
                        "  {}: VerbatimEnd {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                _ => {
                    println!("  {}: {:?}", i, token);
                }
            }
        }

        // Find VerbatimStart token
        let verbatim_start = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimStart { .. }))
            .expect("Should find VerbatimStart token");

        if let Token::VerbatimStart { content, .. } = verbatim_start {
            assert_eq!(
                content, "My Code Title",
                "VerbatimStart should contain title without colon"
            );
            assert!(
                !content.ends_with(':'),
                "VerbatimStart content should not end with colon"
            );
            println!(
                "✅ VerbatimStart correctly contains title without colon: '{}'",
                content
            );
        }
    }

    #[test]
    fn debug_various_titles() {
        let test_cases = vec![
            ("Simple:", "Simple"),
            ("With Spaces:", "With Spaces"),
            ("Complex Title Name:", "Complex Title Name"),
            ("123 Numbers:", "123 Numbers"),
        ];

        for (input_title, expected_title) in test_cases {
            let input = format!("{}\n    content\n()\n", input_title);
            println!("Testing: '{}'", input_title);

            let tokens = tokenize(&input);

            let verbatim_start = tokens
                .iter()
                .find(|token| matches!(token, Token::VerbatimStart { .. }))
                .expect("Should find VerbatimStart token");

            if let Token::VerbatimStart { content, .. } = verbatim_start {
                assert_eq!(
                    content, expected_title,
                    "Title '{}' should become '{}'",
                    input_title, expected_title
                );
                assert!(!content.ends_with(':'), "Content should not end with colon");
                println!("  ✅ '{}' -> '{}'", input_title, content);
            }
        }
    }
}
