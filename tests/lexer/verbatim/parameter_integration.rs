//! Tests for verbatim label parameter integration
//!
//! These tests verify that verbatim labels with parameters are correctly split into:
//! - Clean VerbatimLabel tokens (without parameters)
//! - Parameter components as basic tokens (Identifier, Equals, Text/QuotedString)

use crate::infrastructure::parameter_fixtures::{
    assert_parameters_match, extract_parameters_from_verbatim_label,
};
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

        // Find VerbatimLabel token
        let label_raw = tokens
            .iter()
            .find_map(|token| match token {
                ScannerToken::VerbatimLabel { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .expect("Should find VerbatimLabel token");

        assert_eq!(
            label_raw, "python",
            "VerbatimLabel should contain just the label"
        );

        // Should not have any parameters
        let params = extract_parameters_from_verbatim_label(label_raw);
        assert_eq!(
            params.len(),
            0,
            "Should not have parameters when none specified"
        );
    }

    #[test]
    fn test_verbatim_label_with_single_parameter() {
        let input = r#"Code:
    print("hello")
:: python:version=3.9"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_raw = tokens
            .iter()
            .find_map(|token| match token {
                ScannerToken::VerbatimLabel { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .expect("Should find VerbatimLabel token");

        assert_eq!(
            label_raw, "python:version=3.9",
            "VerbatimBlockEnd should contain label and parameters"
        );

        // Extract label (before first colon)
        let label = label_raw.split(':').next().unwrap();
        assert_eq!(label, "python", "Label should be just 'python'");

        // Extract and check parameters
        let params = extract_parameters_from_verbatim_label(label_raw);
        assert_eq!(params.len(), 1, "Should have exactly one parameter");
        assert_eq!(
            params.get("version"),
            Some(&"3.9".to_string()),
            "Should contain version=3.9"
        );
    }

    #[test]
    fn test_verbatim_label_with_multiple_parameters() {
        let input = r#"Code:
    print("hello")
:: python:version=3.9,syntax=true,style=pep8"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_raw = tokens
            .iter()
            .find_map(|token| match token {
                ScannerToken::VerbatimLabel { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .expect("Should find VerbatimLabel token");

        // Extract label (before first colon)
        let label = label_raw.split(':').next().unwrap();
        assert_eq!(label, "python", "Label should be just 'python'");

        // Extract and check parameters
        let params = extract_parameters_from_verbatim_label(label_raw);

        assert_eq!(params.len(), 3, "Should have exactly three parameters");

        // Check parameters using assert helper
        let expected_params = vec![("version", "3.9"), ("syntax", "true"), ("style", "pep8")];

        assert_parameters_match(&params, &expected_params);
    }

    #[test]
    fn test_verbatim_label_with_quoted_parameters() {
        let input = r#"Example:
    content here
:: mylabel:title="My Document",author="Jane Doe""#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_raw = tokens
            .iter()
            .find_map(|token| match token {
                ScannerToken::VerbatimLabel { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .expect("Should find VerbatimLabel token");

        // Extract label (before first colon)
        let label = label_raw.split(':').next().unwrap();
        assert_eq!(label, "mylabel", "Label should be just 'mylabel'");

        // Extract and check parameters
        let params = extract_parameters_from_verbatim_label(label_raw);

        assert_eq!(params.len(), 2, "Should have exactly two parameters");

        let expected_params = vec![("title", "My Document"), ("author", "Jane Doe")];

        assert_parameters_match(&params, &expected_params);
    }

    #[test]
    fn test_verbatim_label_with_boolean_parameters() {
        let input = r#"Example:
    content here
:: mylabel:debug,version=2.0,verbose"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_raw = tokens
            .iter()
            .find_map(|token| match token {
                ScannerToken::VerbatimLabel { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .expect("Should find VerbatimLabel token");

        // Extract label (before first colon)
        let label = label_raw.split(':').next().unwrap();
        assert_eq!(label, "mylabel", "Label should be just 'mylabel'");

        // Extract and check parameters
        let params = extract_parameters_from_verbatim_label(label_raw);

        assert_eq!(params.len(), 3, "Should have exactly three parameters");

        let expected_params = vec![
            ("debug", "true"), // Boolean shorthand
            ("version", "2.0"),
            ("verbose", "true"), // Boolean shorthand
        ];

        assert_parameters_match(&params, &expected_params);
    }

    #[test]
    fn test_verbatim_label_with_namespaced_parameters() {
        let input = r#"Example:
    content here
:: mylabel:org.example.version=2.0,company.auth.enabled=true"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_raw = tokens
            .iter()
            .find_map(|token| match token {
                ScannerToken::VerbatimLabel { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .expect("Should find VerbatimLabel token");

        // Extract label (before first colon)
        let label = label_raw.split(':').next().unwrap();
        assert_eq!(label, "mylabel", "Label should be just 'mylabel'");

        // Extract and check parameters
        let params = extract_parameters_from_verbatim_label(label_raw);

        assert_eq!(params.len(), 2, "Should have exactly two parameters");

        let expected_params = vec![
            ("org.example.version", "2.0"),
            ("company.auth.enabled", "true"),
        ];

        assert_parameters_match(&params, &expected_params);
    }

    #[test]
    fn test_verbatim_label_with_escaped_parameters() {
        let input = r#"Example:
    content here
:: mylabel:message="She said, \"Hello!\"",path="C:\\Users\\Name""#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_raw = tokens
            .iter()
            .find_map(|token| match token {
                ScannerToken::VerbatimLabel { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .expect("Should find VerbatimLabel token");

        // Extract label (before first colon)
        let label = label_raw.split(':').next().unwrap();
        assert_eq!(label, "mylabel", "Label should be just 'mylabel'");

        // Extract and check parameters
        let params = extract_parameters_from_verbatim_label(label_raw);

        assert_eq!(params.len(), 2, "Should have exactly two parameters");

        let expected_params = vec![
            ("message", r#"She said, "Hello!""#),
            ("path", r"C:\Users\Name"),
        ];

        assert_parameters_match(&params, &expected_params);
    }
}
