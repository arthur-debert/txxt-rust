//! Debug test for verbatim block terminator parsing
//!
//! This test verifies that the tokenizer correctly captures VerbatimLabel tokens
//! with label and parameter information instead of skipping the terminator line.

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_verbatim_with_label_and_params() {
        let input = r#"Python Code:
    print("hello world")
    x = 42
:: python:version=3.9,syntax_highlight=true"#;

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

        // Verify we have a VerbatimLabel token with the terminator content
        let verbatim_end_tokens: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::VerbatimLabel { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimLabel token"
        );

        if let Token::VerbatimLabel { content, .. } = &verbatim_end_tokens[0] {
            assert!(
                content.contains("python"),
                "VerbatimLabel should contain label 'python'"
            );
            assert!(
                content.contains("version=3.9"),
                "VerbatimLabel should contain parameter 'version=3.9'"
            );
            assert!(
                content.contains("syntax_highlight=true"),
                "VerbatimLabel should contain parameter 'syntax_highlight=true'"
            );
            println!("\n✅ VerbatimLabel token correctly captured: {}", content);
        }
    }

    #[test]
    fn debug_verbatim_simple_label() {
        let input = r#"Code:
    some content
:: mylabel"#;

        println!("Input:\n{}", input);

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let verbatim_end_tokens: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::VerbatimLabel { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimLabel token"
        );

        if let Token::VerbatimLabel { content, .. } = &verbatim_end_tokens[0] {
            assert!(
                content.contains("mylabel"),
                "VerbatimLabel should contain full terminator"
            );
            println!("✅ Simple label correctly captured: {}", content);
        }
    }

    #[test]
    fn debug_verbatim_empty_terminator() {
        let input = r#"Code:
    some content
:: empty"#;

        println!("Input:\n{}", input);

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let verbatim_end_tokens: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::VerbatimLabel { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimLabel token"
        );

        if let Token::VerbatimLabel { content, .. } = &verbatim_end_tokens[0] {
            assert!(
                content.contains("empty"),
                "VerbatimLabel should contain empty terminator"
            );
            println!("✅ Empty terminator correctly captured: {}", content);
        }
    }
}
