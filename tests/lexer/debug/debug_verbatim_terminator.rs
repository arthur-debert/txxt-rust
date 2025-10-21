//! Debug test for verbatim block terminator parsing
//!
//! This test verifies that the tokenizer correctly captures VerbatimLabel tokens
//! with label and parameter information instead of skipping the terminator line.

use txxt::cst::ScannerToken;
use txxt::syntax::tokenize;

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
                ScannerToken::VerbatimTitle { content, span } => {
                    println!(
                        "  {}: VerbatimTitle {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                ScannerToken::IndentationWall {
                    level,
                    wall_type,
                    span,
                } => {
                    println!(
                        "  {}: IndentationWall {{ level: {}, wall_type: {:?}, span: {:?} }}",
                        i, level, wall_type, span
                    );
                }
                ScannerToken::IgnoreTextSpan { content, span } => {
                    println!(
                        "  {}: IgnoreText {{ content: {:?}, span: {:?} }}",
                        i, content, span
                    );
                }
                ScannerToken::VerbatimLabel { content, span } => {
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
            .filter(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimLabel token"
        );

        if let ScannerToken::VerbatimLabel { content, .. } = &verbatim_end_tokens[0] {
            assert_eq!(
                content, "python",
                "VerbatimLabel should contain ONLY the label 'python'"
            );
            assert!(
                !content.contains("version=3.9"),
                "VerbatimLabel should NOT contain parameters (now separate tokens)"
            );
            assert!(
                !content.contains("syntax_highlight=true"),
                "VerbatimLabel should NOT contain parameters (now separate tokens)"
            );
            println!("\n✅ VerbatimLabel token correctly captured: {}", content);
        }

        // UPDATED: Check that parameters were extracted as separate Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let ScannerToken::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            param_tokens.len(),
            2,
            "Should have extracted 2 Parameter tokens"
        );
        assert!(
            param_tokens.contains(&("version".to_string(), "3.9".to_string())),
            "Should have version=3.9 parameter"
        );
        assert!(
            param_tokens.contains(&("syntax_highlight".to_string(), "true".to_string())),
            "Should have syntax_highlight=true parameter"
        );
        println!(
            "✅ Parameter tokens correctly extracted: {:?}",
            param_tokens
        );
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
            .filter(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimLabel token"
        );

        if let ScannerToken::VerbatimLabel { content, .. } = &verbatim_end_tokens[0] {
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
            .filter(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimLabel token"
        );

        if let ScannerToken::VerbatimLabel { content, .. } = &verbatim_end_tokens[0] {
            assert!(
                content.contains("empty"),
                "VerbatimLabel should contain empty terminator"
            );
            println!("✅ Empty terminator correctly captured: {}", content);
        }
    }
}
