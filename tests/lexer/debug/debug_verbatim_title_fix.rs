//! Debug test for VerbatimBlockStart title content
//!
//! This test verifies that VerbatimBlockStart tokens contain only the title content
//! without the trailing colon structural marker.

use txxt::cst::ScannerToken;
use txxt::syntax::tokenize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_verbatim_title_without_colon() {
        let input = r#"My Code Title:
    print("hello")
:: code ::"#;

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

        // Find VerbatimBlockStart token
        let verbatim_start = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockStart { .. }))
            .expect("Should find VerbatimBlockStart token");

        if let ScannerToken::VerbatimBlockStart { title, .. } = verbatim_start {
            assert_eq!(
                title, "My Code Title",
                "VerbatimBlockStart should contain title without colon"
            );
            assert!(
                !title.ends_with(':'),
                "VerbatimBlockStart title should not end with colon"
            );
            println!(
                "✅ VerbatimBlockStart correctly contains title without colon: '{}'",
                title
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
            let input = format!("{}\n    content\n:: label ::\n", input_title);
            println!("Testing: '{}'", input_title);

            let tokens = tokenize(&input);

            let verbatim_start = tokens
                .iter()
                .find(|token| matches!(token, ScannerToken::VerbatimBlockStart { .. }))
                .expect("Should find VerbatimBlockStart token");

            if let ScannerToken::VerbatimBlockStart { title, .. } = verbatim_start {
                assert_eq!(
                    title, expected_title,
                    "Title '{}' should become '{}'",
                    input_title, expected_title
                );
                assert!(!title.ends_with(':'), "Title should not end with colon");
                println!("  ✅ '{}' -> '{}'", input_title, title);
            }
        }
    }
}
