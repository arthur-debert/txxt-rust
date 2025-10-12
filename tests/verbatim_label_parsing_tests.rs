//! Tests for verbatim label parsing functionality
//!
//! These tests demonstrate that VerbatimLabel tokens should:
//! 1. Extract label content WITHOUT the :: prefix
//! 2. Properly separate labels from parameters
//! 3. Handle various parameter formats

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

#[cfg(test)]
mod verbatim_label_tests {
    use super::*;

    #[test]
    fn test_verbatim_label_without_prefix() {
        let input = r#"Code:
    print("hello")
:: python"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            // This test SHOULD FAIL initially - content currently includes "::"
            assert_eq!(
                content, "python",
                "VerbatimLabel should contain ONLY the label, not the :: prefix"
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_simple_parameters() {
        let input = r#"Code:
    print("hello")
:: python:version=3.9,syntax=true"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            // This test SHOULD FAIL initially - no parameter parsing
            assert_eq!(
                content, "python:version=3.9,syntax=true",
                "VerbatimLabel should contain label and parameters without :: prefix"
            );
        }
    }

    #[test]
    fn test_verbatim_label_parameter_separation() {
        let input = r#"Example:
    content here
:: mylabel:key1=value1,key2=value2"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            // For now, we expect the full label:params string (without ::)
            assert_eq!(
                content, "mylabel:key1=value1,key2=value2",
                "VerbatimLabel should contain full label:params without :: prefix"
            );

            // The content should be parseable into label and parameters
            assert!(content.contains("mylabel"), "Should contain the label part");
            assert!(content.contains("key1=value1"), "Should contain parameters");
            assert!(!content.starts_with("::"), "Should not start with ::");
        }
    }

    #[test]
    fn test_verbatim_label_edge_cases() {
        let test_cases = vec![
            (":: simple", "simple"),
            (":: label-with-dashes", "label-with-dashes"),
            (":: label_with_underscores", "label_with_underscores"),
            (":: namespace.label", "namespace.label"),
        ];

        for (terminator_line, expected_label) in test_cases {
            let input = format!("Title:\n    content\n{}", terminator_line);
            let tokens = tokenize(&input);

            let label_token = tokens
                .iter()
                .find(|token| matches!(token, Token::VerbatimLabel { .. }))
                .unwrap_or_else(|| panic!("Should find VerbatimLabel token for: {}",
                    terminator_line));

            if let Token::VerbatimLabel { content, .. } = label_token {
                assert_eq!(
                    content, expected_label,
                    "VerbatimLabel for '{}' should be '{}'",
                    terminator_line, expected_label
                );
            }
        }
    }

    #[test]
    fn test_fixed_behavior_verification() {
        // This test verifies the FIXED behavior
        let input = r#"Code:
    example
:: mylabel"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            // This verifies the FIXED behavior - content excludes "::"
            assert_eq!(
                content, "mylabel",
                "FIXED: VerbatimLabel now correctly excludes :: prefix"
            );
        }
    }
}
