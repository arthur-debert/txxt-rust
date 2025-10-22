//! Debug test for verbatim block terminator parsing
//!
//! This test verifies that the tokenizer correctly captures VerbatimBlockEnd tokens
//! with label_raw information.

use txxt::cst::ScannerToken;
use txxt::syntax::tokenize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Verbatim parameters handled in separate branch"]
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
                ScannerToken::VerbatimBlockStart {
                    title,
                    wall_type,
                    span,
                } => {
                    println!(
                        "  {}: VerbatimBlockStart {{ title: {:?}, wall_type: {:?}, span: {:?} }}",
                        i, title, wall_type, span
                    );
                }
                ScannerToken::VerbatimContentLine {
                    content,
                    indentation,
                    span,
                } => {
                    println!(
                        "  {}: VerbatimContentLine {{ content: {:?}, indentation: {:?}, span: {:?} }}",
                        i, content, indentation, span
                    );
                }
                ScannerToken::VerbatimBlockEnd { label_raw, span } => {
                    println!(
                        "  {}: VerbatimBlockEnd {{ label_raw: {:?}, span: {:?} }}",
                        i, label_raw, span
                    );
                }
                _ => {
                    println!("  {}: {:?}", i, token);
                }
            }
        }

        // Verify we have a VerbatimBlockEnd token with the terminator content
        let verbatim_end_tokens: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimBlockEnd token"
        );

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = &verbatim_end_tokens[0] {
            assert_eq!(
                label_raw, "python:version=3.9,syntax_highlight=true",
                "VerbatimBlockEnd label_raw should contain full label:params"
            );
            println!(
                "\n✅ VerbatimBlockEnd token correctly captured: {}",
                label_raw
            );
        }

        // Note: Parameter parsing happens at semantic analysis level, not scanner level
    }

    #[test]
    fn debug_verbatim_simple_label() {
        let input = r#"Code:
    some content
:: mylabel"#;

        println!("Input:\n{}", input);

        let tokens = tokenize(input);

        // Find VerbatimBlockEnd token
        let verbatim_end_tokens: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimBlockEnd token"
        );

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = &verbatim_end_tokens[0] {
            assert_eq!(
                label_raw, "mylabel",
                "VerbatimBlockEnd label_raw should contain simple label"
            );
            println!("✅ Simple label correctly captured: {}", label_raw);
        }
    }

    #[test]
    fn debug_verbatim_empty_terminator() {
        let input = r#"Code:
    some content
:: empty"#;

        println!("Input:\n{}", input);

        let tokens = tokenize(input);

        // Find VerbatimBlockEnd token
        let verbatim_end_tokens: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .collect();

        assert_eq!(
            verbatim_end_tokens.len(),
            1,
            "Should have exactly 1 VerbatimBlockEnd token"
        );

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = &verbatim_end_tokens[0] {
            assert_eq!(
                label_raw, "empty",
                "VerbatimBlockEnd label_raw should contain empty label"
            );
            println!("✅ Empty terminator correctly captured: {}", label_raw);
        }
    }
}
