//! Tests for annotation parameter integration
//!
//! These tests verify that annotations with parameters are correctly split into:
//! - Clean AnnotationMarker tokens (::)
//! - Clean text content for labels (without parameters)
//! - Individual Parameter tokens for each key=value pair
//!
//! Following the same pattern established for verbatim parameter integration.

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

mod fixtures;
use fixtures::{ElementParameterFixture, ParameterFixtures};

#[cfg(test)]
mod annotation_parameter_integration_tests {
    use super::*;

    #[test]
    fn test_annotation_without_parameters() {
        let input = ":: warning :: Critical security issue";
        let tokens = tokenize(input);

        // Find AnnotationMarker tokens
        let annotation_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::AnnotationMarker { .. }))
            .collect();

        assert_eq!(
            annotation_markers.len(),
            2,
            "Should have exactly 2 AnnotationMarker tokens"
        );

        // Find text content (should be clean label without parameters)
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"warning"),
            "Should find clean 'warning' label"
        );
        assert!(
            text_tokens.contains(&"Critical"),
            "Should find annotation content"
        );

        // Should not have any Parameter tokens
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        assert_eq!(
            param_tokens.len(),
            0,
            "Should not have Parameter tokens when no parameters"
        );
    }

    #[test]
    fn test_annotation_with_simple_parameter() {
        let fixture = ElementParameterFixture::annotation(
            "warning",
            ParameterFixtures::simple(),
            ParameterFixtures::simple_expected(),
        );
        let input = fixture.annotation_input();
        let tokens = tokenize(&input);

        // Find AnnotationMarker tokens
        let annotation_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::AnnotationMarker { .. }))
            .collect();

        assert_eq!(
            annotation_markers.len(),
            2,
            "Should have exactly 2 AnnotationMarker tokens"
        );

        // Find text content - should be clean label without parameters
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"warning"),
            "Should find clean 'warning' label without parameters"
        );

        // Verify parameters were extracted as separate tokens
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_annotation_with_multiple_parameters() {
        let fixture = ElementParameterFixture::annotation(
            "meta",
            ParameterFixtures::multiple(),
            ParameterFixtures::multiple_expected(),
        );
        let input = fixture.annotation_input();
        let tokens = tokenize(&input);

        // Find text content - should be clean label
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"meta"),
            "Should find clean 'meta' label without parameters"
        );

        // Verify all parameters were extracted
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_annotation_with_quoted_parameters() {
        let fixture = ElementParameterFixture::annotation(
            "info",
            ParameterFixtures::quoted(),
            ParameterFixtures::quoted_expected(),
        );
        let input = fixture.annotation_input();
        let tokens = tokenize(&input);

        // Find clean label
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"info"),
            "Should find clean 'info' label"
        );

        // Verify quoted parameters were parsed correctly
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_annotation_with_escaped_parameters() {
        let fixture = ElementParameterFixture::annotation(
            "data",
            ParameterFixtures::escaped(),
            ParameterFixtures::escaped_expected(),
        );
        let input = fixture.annotation_input();
        let tokens = tokenize(&input);

        // Find clean label
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"data"),
            "Should find clean 'data' label"
        );

        // Verify escaped parameters were parsed correctly
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_annotation_with_boolean_parameters() {
        let fixture = ElementParameterFixture::annotation(
            "config",
            ParameterFixtures::boolean(),
            ParameterFixtures::boolean_expected(),
        );
        let input = fixture.annotation_input();
        let tokens = tokenize(&input);

        // Find clean label
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"config"),
            "Should find clean 'config' label"
        );

        // Verify boolean parameters were parsed correctly
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_annotation_with_namespaced_parameters() {
        let fixture = ElementParameterFixture::annotation(
            "system",
            ParameterFixtures::namespaced(),
            ParameterFixtures::namespaced_expected(),
        );
        let input = fixture.annotation_input();
        let tokens = tokenize(&input);

        // Find clean label
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"system"),
            "Should find clean 'system' label"
        );

        // Verify namespaced parameters were parsed correctly
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_annotation_block_with_parameters() {
        let fixture = ElementParameterFixture::annotation(
            "description",
            ParameterFixtures::simple(),
            ParameterFixtures::simple_expected(),
        );
        let input = fixture.annotation_block_input();
        let tokens = tokenize(&input);

        // Find AnnotationMarker tokens
        let annotation_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::AnnotationMarker { .. }))
            .collect();

        assert_eq!(
            annotation_markers.len(),
            2,
            "Should have exactly 2 AnnotationMarker tokens for block annotation"
        );

        // Find clean label
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"description"),
            "Should find clean 'description' label"
        );

        // Verify parameters were extracted
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_multiple_annotations_with_parameters() {
        let input = r#":: warning:severity=high :: Critical issue
:: info:type=note :: Additional information"#;
        let tokens = tokenize(input);

        // Should have 4 AnnotationMarker tokens (2 per annotation)
        let annotation_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::AnnotationMarker { .. }))
            .collect();

        assert_eq!(
            annotation_markers.len(),
            4,
            "Should have exactly 4 AnnotationMarker tokens"
        );

        // Find clean labels
        let text_tokens: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if let Token::Text { content, .. } = token {
                    Some(content.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            text_tokens.contains(&"warning"),
            "Should find clean 'warning' label"
        );
        assert!(
            text_tokens.contains(&"info"),
            "Should find clean 'info' label"
        );

        // Verify all parameters were extracted
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        assert_eq!(
            param_tokens.len(),
            2,
            "Should have exactly 2 parameter tokens"
        );

        let expected_params = vec![("severity", "high"), ("type", "note")];
        ParameterFixtures::verify_parameters(&param_tokens, &expected_params);
    }
}
