//! Tests for verbatim label parsing functionality
//!
//! These tests demonstrate that VerbatimLabel tokens should:
//! 1. Extract label content WITHOUT the :: prefix
//! 2. Properly separate labels from parameters
//! 3. Handle various parameter formats

use txxt::cst::ScannerToken;
use txxt::syntax::tokenize;

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
            .find(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let ScannerToken::VerbatimLabel { content, .. } = label_token {
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
            .find(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let ScannerToken::VerbatimLabel { content, .. } = label_token {
            // UPDATED: VerbatimLabel now contains ONLY the label, not parameters
            assert_eq!(
                content, "python",
                "VerbatimLabel should contain ONLY the label without parameters"
            );
        }

        // UPDATED: Check that parameters were extracted as separate tokens
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
            "Should have extracted 2 parameter tokens"
        );
        assert!(param_tokens.contains(&("version".to_string(), "3.9".to_string())));
        assert!(param_tokens.contains(&("syntax".to_string(), "true".to_string())));
    }

    #[test]
    fn test_verbatim_label_parameter_separation() {
        let input = r#"Example:
    content here
:: mylabel:key1=value1,key2=value2"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let ScannerToken::VerbatimLabel { content, .. } = label_token {
            // UPDATED: VerbatimLabel now contains ONLY the label
            assert_eq!(
                content, "mylabel",
                "VerbatimLabel should contain ONLY the label without parameters"
            );
            assert!(
                !content.contains(":"),
                "Label should not contain colon separator"
            );
            assert!(!content.starts_with("::"), "Should not start with ::");
        }

        // UPDATED: Check that parameters were extracted as separate tokens
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
            "Should have extracted 2 parameter tokens"
        );
        assert!(param_tokens.contains(&("key1".to_string(), "value1".to_string())));
        assert!(param_tokens.contains(&("key2".to_string(), "value2".to_string())));
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
                .find(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
                .unwrap_or_else(|| {
                    panic!("Should find VerbatimLabel token for: {}", terminator_line)
                });

            if let ScannerToken::VerbatimLabel { content, .. } = label_token {
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
            .find(|token| matches!(token, ScannerToken::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let ScannerToken::VerbatimLabel { content, .. } = label_token {
            // This verifies the FIXED behavior - content excludes "::"
            assert_eq!(
                content, "mylabel",
                "FIXED: VerbatimLabel now correctly excludes :: prefix"
            );
        }
    }
}
