//! Tests for verbatim label parsing functionality
//!
//! These tests demonstrate that VerbatimBlockEnd tokens should:
//! 1. Extract label content WITHOUT the :: prefix
//! 2. Store the raw label and parameters together as label_raw
//! 3. Handle various label formats

use txxt::cst::ScannerToken;
use txxt::syntax::tokenize;

#[cfg(test)]
mod verbatim_label_tests {
    use super::*;

    #[test]
    fn test_verbatim_label_without_prefix() {
        let input = r#"Code:
    print("hello")
:: python ::"#;

        let tokens = tokenize(input);

        // Find VerbatimBlockEnd token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            // VerbatimBlockEnd should contain ONLY the label, not the :: prefix
            assert_eq!(
                label_raw, "python",
                "VerbatimBlockEnd label_raw should contain ONLY the label, not the :: prefix"
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_simple_parameters() {
        let input = r#"Code:
    print("hello")
:: python version=3.9,syntax=true ::"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            // VerbatimBlockEnd contains the raw content between :: markers
            assert_eq!(
                label_raw, "python version=3.9,syntax=true",
                "VerbatimBlockEnd label_raw should contain label and parameters"
            );

            // The raw string should NOT include :: prefix
            assert!(
                !label_raw.starts_with("::"),
                "label_raw should not include :: prefix"
            );
        }

        // Note: Parameter parsing happens at the semantic analysis level,
        // not at the scanner token level
    }

    #[test]
    fn test_verbatim_label_parameter_separation() {
        let input = r#"Example:
    content here
:: mylabel key1=value1,key2=value2 ::"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            // VerbatimBlockEnd stores the raw content between :: markers
            assert_eq!(
                label_raw, "mylabel key1=value1,key2=value2",
                "VerbatimBlockEnd label_raw should contain full label and params"
            );
            assert!(!label_raw.starts_with("::"), "Should not start with ::");
        }

        // Parameter parsing happens at semantic analysis level
    }

    #[test]
    fn test_verbatim_label_edge_cases() {
        let test_cases = vec![
            (":: simple ::", "simple"),
            (":: label-with-dashes ::", "label-with-dashes"),
            (":: label_with_underscores ::", "label_with_underscores"),
            (":: namespace.label ::", "namespace.label"),
        ];

        for (terminator_line, expected_label) in test_cases {
            let input = format!("Title:\n    content\n{}", terminator_line);
            let tokens = tokenize(&input);

            let label_token = tokens
                .iter()
                .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
                .unwrap_or_else(|| {
                    panic!(
                        "Should find VerbatimBlockEnd token for: {}",
                        terminator_line
                    )
                });

            if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
                assert_eq!(
                    label_raw, expected_label,
                    "VerbatimBlockEnd label_raw for '{}' should be '{}'",
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
:: mylabel ::"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            // This verifies the FIXED behavior - label_raw excludes "::"
            assert_eq!(
                label_raw, "mylabel",
                "FIXED: VerbatimBlockEnd label_raw now correctly excludes :: prefix"
            );
        }
    }
}
