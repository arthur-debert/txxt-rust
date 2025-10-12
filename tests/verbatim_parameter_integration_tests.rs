//! Tests for verbatim label parameter integration
//!
//! These tests verify that verbatim labels with parameters are correctly split into:
//! - Clean VerbatimLabel tokens (without parameters)
//! - Individual Parameter tokens for each key=value pair

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

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
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            assert_eq!(content, "python", "VerbatimLabel should be just the label");
        }

        // Should not have any Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::Parameter { .. }))
            .collect();

        assert_eq!(
            param_tokens.len(),
            0,
            "Should not have Parameter tokens when no parameters"
        );
    }

    #[test]
    fn test_verbatim_label_with_single_parameter() {
        let input = r#"Code:
    print("hello")
:: python:version=3.9"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            assert_eq!(
                content, "python",
                "VerbatimLabel should be just the label without parameters"
            );
        }

        // Find Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            param_tokens.len(),
            1,
            "Should have exactly one Parameter token"
        );
        assert_eq!(param_tokens[0], ("version".to_string(), "3.9".to_string()));
    }

    #[test]
    fn test_verbatim_label_with_multiple_parameters() {
        let input = r#"Code:
    print("hello")
:: python:version=3.9,syntax=true,style=pep8"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            assert_eq!(content, "python", "VerbatimLabel should be just the label");
        }

        // Find Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            param_tokens.len(),
            3,
            "Should have exactly three Parameter tokens"
        );

        // Convert to set for order-independent comparison
        let expected_params = vec![
            ("version".to_string(), "3.9".to_string()),
            ("syntax".to_string(), "true".to_string()),
            ("style".to_string(), "pep8".to_string()),
        ];

        for expected in expected_params {
            assert!(
                param_tokens.contains(&expected),
                "Should contain parameter: {:?}",
                expected
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_quoted_parameters() {
        let input = r#"Example:
    content here
:: mylabel:title="My Document",author="Jane Doe""#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            assert_eq!(content, "mylabel", "VerbatimLabel should be just the label");
        }

        // Find Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            param_tokens.len(),
            2,
            "Should have exactly two Parameter tokens"
        );

        let expected_params = vec![
            ("title".to_string(), "My Document".to_string()),
            ("author".to_string(), "Jane Doe".to_string()),
        ];

        for expected in expected_params {
            assert!(
                param_tokens.contains(&expected),
                "Should contain parameter: {:?}",
                expected
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_boolean_parameters() {
        let input = r#"Example:
    content here
:: mylabel:debug,version=2.0,verbose"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            assert_eq!(content, "mylabel", "VerbatimLabel should be just the label");
        }

        // Find Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            param_tokens.len(),
            3,
            "Should have exactly three Parameter tokens"
        );

        let expected_params = vec![
            ("debug".to_string(), "true".to_string()), // Boolean shorthand
            ("version".to_string(), "2.0".to_string()),
            ("verbose".to_string(), "true".to_string()), // Boolean shorthand
        ];

        for expected in expected_params {
            assert!(
                param_tokens.contains(&expected),
                "Should contain parameter: {:?}",
                expected
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_namespaced_parameters() {
        let input = r#"Example:
    content here
:: mylabel:org.example.version=2.0,company.auth.enabled=true"#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            assert_eq!(content, "mylabel", "VerbatimLabel should be just the label");
        }

        // Find Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            param_tokens.len(),
            2,
            "Should have exactly two Parameter tokens"
        );

        let expected_params = vec![
            ("org.example.version".to_string(), "2.0".to_string()),
            ("company.auth.enabled".to_string(), "true".to_string()),
        ];

        for expected in expected_params {
            assert!(
                param_tokens.contains(&expected),
                "Should contain parameter: {:?}",
                expected
            );
        }
    }

    #[test]
    fn test_verbatim_label_with_escaped_parameters() {
        let input = r#"Example:
    content here
:: mylabel:message="She said, \"Hello!\"",path="C:\\Users\\Name""#;

        let tokens = tokenize(input);

        // Find VerbatimLabel token
        let label_token = tokens
            .iter()
            .find(|token| matches!(token, Token::VerbatimLabel { .. }))
            .expect("Should find VerbatimLabel token");

        if let Token::VerbatimLabel { content, .. } = label_token {
            assert_eq!(content, "mylabel", "VerbatimLabel should be just the label");
        }

        // Find Parameter tokens
        let param_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            param_tokens.len(),
            2,
            "Should have exactly two Parameter tokens"
        );

        let expected_params = vec![
            ("message".to_string(), r#"She said, "Hello!""#.to_string()),
            ("path".to_string(), r"C:\Users\Name".to_string()),
        ];

        for expected in expected_params {
            assert!(
                param_tokens.contains(&expected),
                "Should contain parameter: {:?}",
                expected
            );
        }
    }
}
