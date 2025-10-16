//! Debug test for VerbatimTitle title content fix
//!
//! This test verifies that VerbatimTitle tokens contain only the title content
//! without the trailing colon structural marker.

use txxt::ast::tokens::Token;
use txxt::lexer::tokenize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_verbatim_title_without_colon() {
        let input = r#"My Code Title:
    print("hello")
:: code"#;

        println!("Input:\n{}", input);

        let tokens = tokenize(input);

        println!("\nTokens:");
        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::VerbatimTitle { content, span } => {
                    println!(
                        "  {}: VerbatimTitle {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                Token::VerbatimContent { content, span } => {
                    println!(
                        "  {}: VerbatimContent {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                Token::VerbatimLabel { content, span } => {
                    println!(
                        "  {}: VerbatimLabel {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                _ => {
                    println!("  {}: {:?}", i, token);
                }
            }
        }

        // Find VerbatimTitle token
        let verbatim_start = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimTitle { .. }))
            .expect("Should find VerbatimTitle token");

        if let Token::VerbatimTitle { content, .. } = verbatim_start {
            assert_eq!(
                content, "My Code Title",
                "VerbatimTitle should contain title without colon"
            );
            assert!(
                !content.ends_with(':'),
                "VerbatimTitle content should not end with colon"
            );
            println!(
                "✅ VerbatimTitle correctly contains title without colon: '{}'",
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
            let input = format!("{}\n    content\n:: label\n", input_title);
            println!("Testing: '{}'", input_title);

            let tokens = tokenize(&input);

            let verbatim_start = tokens
                .iter()
                .find(|token| matches!(token, Token::VerbatimTitle { .. }))
                .expect("Should find VerbatimTitle token");

            if let Token::VerbatimTitle { content, .. } = verbatim_start {
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
