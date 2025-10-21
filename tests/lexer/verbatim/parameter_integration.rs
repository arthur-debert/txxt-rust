//! Tests for verbatim label parameter integration
//!
//! These tests verify that verbatim labels with parameters are correctly stored in
//! VerbatimBlockEnd's label_raw field. Parameter parsing happens at semantic analysis level.

use txxt::cst::ScannerToken;
use txxt::syntax::tokenize;

#[cfg(test)]
mod verbatim_parameter_integration_tests {
    use super::*;

    #[test]
    fn test_verbatim_label_without_parameters() {
        let input = r#"Code:
    print("hello")
:: python"#;

        let tokens = tokenize(input);

        // Find VerbatimBlockEnd token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            assert_eq!(label_raw, "python", "VerbatimBlockEnd label_raw should be just the label");
            assert!(!label_raw.contains(':'), "label_raw should not contain parameters");
        }
    }

    #[test]
    fn test_verbatim_label_with_single_parameter() {
        let input = r#"Code:
    print("hello")
:: python:version=3.9"#;

        let tokens = tokenize(input);

        // Find VerbatimBlockEnd token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            assert_eq!(
                label_raw, "python:version=3.9",
                "VerbatimBlockEnd label_raw should contain label and parameters"
            );

            // Verify the format
            let parts: Vec<&str> = label_raw.split(':').collect();
            assert_eq!(parts.len(), 2, "Should have label and params separated by colon");
            assert_eq!(parts[0], "python", "First part should be label");
            assert_eq!(parts[1], "version=3.9", "Second part should be parameters");
        }

        // Note: Parameter tokens are NOT created at scanner level.
        // They are parsed at semantic analysis level.
    }

    #[test]
    fn test_verbatim_label_with_multiple_parameters() {
        let input = r#"Code:
    print("hello")
:: python:version=3.9,syntax=true"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            assert_eq!(
                label_raw, "python:version=3.9,syntax=true",
                "VerbatimBlockEnd label_raw should contain all parameters"
            );

            // Verify structure
            assert!(label_raw.starts_with("python:"), "Should start with label:");
            assert!(label_raw.contains("version=3.9"), "Should contain version parameter");
            assert!(label_raw.contains("syntax=true"), "Should contain syntax parameter");
        }
    }

    #[test]
    fn test_verbatim_label_with_boolean_parameters() {
        let input = r#"Code:
    example
:: label:flag=true,debug=false"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            assert_eq!(
                label_raw, "label:flag=true,debug=false",
                "VerbatimBlockEnd label_raw should preserve boolean parameter values"
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_quoted_parameters() {
        let input = r#"Code:
    example
:: label:message="hello world",name="test""#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            assert_eq!(
                label_raw, r#"label:message="hello world",name="test""#,
                "VerbatimBlockEnd label_raw should preserve quoted parameter values"
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_escaped_parameters() {
        let input = r#"Code:
    example
:: label:path="C:\folder\file.txt""#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            assert_eq!(
                label_raw, r#"label:path="C:\folder\file.txt""#,
                "VerbatimBlockEnd label_raw should preserve escaped characters"
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_namespaced_parameters() {
        let input = r#"Code:
    example
:: label:custom.namespace.key=value"#;

        let tokens = tokenize(input);

        let label_token = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::VerbatimBlockEnd { .. }))
            .expect("Should find VerbatimBlockEnd token");

        if let ScannerToken::VerbatimBlockEnd { label_raw, .. } = label_token {
            assert_eq!(
                label_raw, "label:custom.namespace.key=value",
                "VerbatimBlockEnd label_raw should preserve namespaced parameter keys"
            );
        }
    }
}
