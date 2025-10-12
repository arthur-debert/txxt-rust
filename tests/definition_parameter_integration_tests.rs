//! Tests for definition parameter integration
//!
//! These tests verify that definitions with parameters are correctly split into:
//! - Clean text content for terms (without parameters)
//! - Individual Parameter tokens for each key=value pair
//! - DefinitionMarker tokens (::)
//!
//! Following the same pattern established for verbatim and annotation parameter integration.

use txxt::ast::tokens::Token;
use txxt::tokenizer::tokenize;

mod fixtures;
use fixtures::{ElementParameterFixture, ParameterFixtures};

#[cfg(test)]
mod definition_parameter_integration_tests {
    use super::*;

    #[test]
    fn test_definition_without_parameters() {
        let input = r#"Parser ::
    A program that analyzes text"#;
        let tokens = tokenize(input);

        // Find DefinitionMarker token
        let definition_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::DefinitionMarker { .. }))
            .collect();

        assert_eq!(
            definition_markers.len(),
            1,
            "Should have exactly 1 DefinitionMarker token"
        );

        // Find text content (should be clean term without parameters)
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
            text_tokens.contains(&"Parser"),
            "Should find clean 'Parser' term"
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
    fn test_definition_with_simple_parameter() {
        let fixture = ElementParameterFixture::definition(
            "API",
            ParameterFixtures::simple(),
            ParameterFixtures::simple_expected(),
        );
        let input = fixture.definition_input();
        let tokens = tokenize(&input);

        // Find DefinitionMarker token
        let definition_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::DefinitionMarker { .. }))
            .collect();

        assert_eq!(
            definition_markers.len(),
            1,
            "Should have exactly 1 DefinitionMarker token"
        );

        // Find text content - should be clean term without parameters
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
            text_tokens.contains(&"API"),
            "Should find clean 'API' term without parameters"
        );

        // Verify parameters were extracted as separate tokens
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_definition_with_multiple_parameters() {
        let fixture = ElementParameterFixture::definition(
            "Algorithm",
            ParameterFixtures::multiple(),
            ParameterFixtures::multiple_expected(),
        );
        let input = fixture.definition_input();
        let tokens = tokenize(&input);

        // Find text content - should be clean term
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
            text_tokens.contains(&"Algorithm"),
            "Should find clean 'Algorithm' term without parameters"
        );

        // Verify all parameters were extracted
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_definition_with_quoted_parameters() {
        let fixture = ElementParameterFixture::definition(
            "Concept",
            ParameterFixtures::quoted(),
            ParameterFixtures::quoted_expected(),
        );
        let input = fixture.definition_input();
        let tokens = tokenize(&input);

        // Find clean term
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
            text_tokens.contains(&"Concept"),
            "Should find clean 'Concept' term"
        );

        // Verify quoted parameters were parsed correctly
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_definition_with_boolean_parameters() {
        let fixture = ElementParameterFixture::definition(
            "Method",
            ParameterFixtures::boolean(),
            ParameterFixtures::boolean_expected(),
        );
        let input = fixture.definition_input();
        let tokens = tokenize(&input);

        // Find clean term
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
            text_tokens.contains(&"Method"),
            "Should find clean 'Method' term"
        );

        // Verify boolean parameters were parsed correctly
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_definition_with_namespaced_parameters() {
        let fixture = ElementParameterFixture::definition(
            "Function",
            ParameterFixtures::namespaced(),
            ParameterFixtures::namespaced_expected(),
        );
        let input = fixture.definition_input();
        let tokens = tokenize(&input);

        // Find clean term
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
            text_tokens.contains(&"Function"),
            "Should find clean 'Function' term"
        );

        // Verify namespaced parameters were parsed correctly
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        ParameterFixtures::verify_parameters(&param_tokens, &fixture.expected_params);
    }

    #[test]
    fn test_definition_with_complex_term() {
        let input = r#"Machine Learning Algorithm:category=ai,complexity=high ::
    An algorithm that learns patterns from data"#;
        let tokens = tokenize(input);

        // Find DefinitionMarker token
        let definition_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::DefinitionMarker { .. }))
            .collect();

        assert_eq!(
            definition_markers.len(),
            1,
            "Should have exactly 1 DefinitionMarker token"
        );

        // Find clean term (should handle multi-word terms)
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
            text_tokens.contains(&"Machine"),
            "Should find first part of term"
        );
        assert!(
            text_tokens.contains(&"Learning"),
            "Should find second part of term"
        );
        assert!(
            text_tokens.contains(&"Algorithm"),
            "Should find third part of term"
        );

        // Verify parameters were extracted
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        assert_eq!(
            param_tokens.len(),
            2,
            "Should have exactly 2 parameter tokens"
        );

        let expected_params = vec![("category", "ai"), ("complexity", "high")];
        ParameterFixtures::verify_parameters(&param_tokens, &expected_params);
    }

    #[test]
    fn test_multiple_definitions_with_parameters() {
        let input = r#"Term1:type=concept ::
    First definition

Term2:type=method,status=active ::
    Second definition"#;
        let tokens = tokenize(input);

        // Should have 2 DefinitionMarker tokens
        let definition_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::DefinitionMarker { .. }))
            .collect();

        assert_eq!(
            definition_markers.len(),
            2,
            "Should have exactly 2 DefinitionMarker tokens"
        );

        // Find clean terms
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
            text_tokens.contains(&"Term1"),
            "Should find clean 'Term1' term"
        );
        assert!(
            text_tokens.contains(&"Term2"),
            "Should find clean 'Term2' term"
        );

        // Verify all parameters were extracted
        let param_tokens = ParameterFixtures::extract_parameter_tokens(&tokens);
        assert_eq!(
            param_tokens.len(),
            3,
            "Should have exactly 3 parameter tokens"
        );

        let expected_params = vec![
            ("type", "concept"),
            ("type", "method"),
            ("status", "active"),
        ];

        // Check each parameter individually since we have duplicate keys
        for (key, value) in expected_params {
            assert!(
                param_tokens.contains(&(key.to_string(), value.to_string())),
                "Should contain parameter: {}={}",
                key,
                value
            );
        }
    }
}
