//! Parameter test fixtures for consistent testing across verbatim, annotation, and definition elements
//!
//! These fixtures provide reusable parameter syntax examples and expected token patterns
//! to ensure consistency across all parameter-enabled elements.

use txxt::ast::tokens::Token;

/// Common parameter syntax examples for testing
pub struct ParameterFixtures;

impl ParameterFixtures {
    /// Simple key=value parameter
    pub fn simple() -> &'static str {
        "version=3.9"
    }

    /// Multiple parameters with different types
    pub fn multiple() -> &'static str {
        "version=3.9,syntax=true,style=pep8"
    }

    /// Quoted string parameters
    pub fn quoted() -> &'static str {
        r#"title="My Document",author="Jane Doe""#
    }

    /// Escaped string parameters
    #[allow(dead_code)]
    pub fn escaped() -> &'static str {
        r#"message="She said, \"Hello!\"",path="C:\\Users\\Name""#
    }

    /// Boolean shorthand parameters
    pub fn boolean() -> &'static str {
        "debug,version=2.0,verbose"
    }

    /// Namespaced parameters
    pub fn namespaced() -> &'static str {
        "org.example.version=2.0,company.auth.enabled=true"
    }

    /// Complex mixed parameters
    #[allow(dead_code)]
    pub fn complex() -> &'static str {
        r#"debug,title="Complex Example",namespace.key=value,count=42"#
    }

    /// Expected parameter tokens for simple() fixture
    pub fn simple_expected() -> Vec<(&'static str, &'static str)> {
        vec![("version", "3.9")]
    }

    /// Expected parameter tokens for multiple() fixture
    pub fn multiple_expected() -> Vec<(&'static str, &'static str)> {
        vec![("version", "3.9"), ("syntax", "true"), ("style", "pep8")]
    }

    /// Expected parameter tokens for quoted() fixture
    pub fn quoted_expected() -> Vec<(&'static str, &'static str)> {
        vec![("title", "My Document"), ("author", "Jane Doe")]
    }

    /// Expected parameter tokens for escaped() fixture
    #[allow(dead_code)]
    pub fn escaped_expected() -> Vec<(&'static str, &'static str)> {
        vec![
            ("message", r#"She said, "Hello!""#),
            ("path", r"C:\Users\Name"),
        ]
    }

    /// Expected parameter tokens for boolean() fixture
    pub fn boolean_expected() -> Vec<(&'static str, &'static str)> {
        vec![("debug", "true"), ("version", "2.0"), ("verbose", "true")]
    }

    /// Expected parameter tokens for namespaced() fixture
    pub fn namespaced_expected() -> Vec<(&'static str, &'static str)> {
        vec![
            ("org.example.version", "2.0"),
            ("company.auth.enabled", "true"),
        ]
    }

    /// Expected parameter tokens for complex() fixture
    #[allow(dead_code)]
    pub fn complex_expected() -> Vec<(&'static str, &'static str)> {
        vec![
            ("debug", "true"),
            ("title", "Complex Example"),
            ("namespace.key", "value"),
            ("count", "42"),
        ]
    }

    /// Extract actual parameter tokens from tokenized output
    pub fn extract_parameter_tokens(tokens: &[Token]) -> Vec<(String, String)> {
        tokens
            .iter()
            .filter_map(|token| {
                if let Token::Parameter { key, value, .. } = token {
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Verify parameter tokens match expected values (order-independent)
    pub fn verify_parameters(actual: &[(String, String)], expected: &[(&str, &str)]) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "Should have exactly {} parameter tokens",
            expected.len()
        );

        for (expected_key, expected_value) in expected {
            assert!(
                actual.contains(&(expected_key.to_string(), expected_value.to_string())),
                "Should contain parameter: {}={}",
                expected_key,
                expected_value
            );
        }
    }
}

/// Test fixture builder for element-specific parameter integration tests
pub struct ElementParameterFixture {
    pub label: &'static str,
    pub parameters: &'static str,
    pub expected_params: Vec<(&'static str, &'static str)>,
}

impl ElementParameterFixture {
    /// Create verbatim block fixture with parameters
    pub fn verbatim(
        label: &'static str,
        parameters: &'static str,
        expected: Vec<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            label,
            parameters,
            expected_params: expected,
        }
    }

    /// Create annotation fixture with parameters
    #[allow(dead_code)]
    pub fn annotation(
        label: &'static str,
        parameters: &'static str,
        expected: Vec<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            label,
            parameters,
            expected_params: expected,
        }
    }

    /// Create definition fixture with parameters
    #[allow(dead_code)]
    pub fn definition(
        label: &'static str,
        parameters: &'static str,
        expected: Vec<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            label,
            parameters,
            expected_params: expected,
        }
    }

    /// Generate verbatim input string
    pub fn verbatim_input(&self) -> String {
        format!(
            r#"Code:
    example content
:: {}:{}"#,
            self.label, self.parameters
        )
    }

    /// Generate annotation input string (inline)
    #[allow(dead_code)]
    pub fn annotation_input(&self) -> String {
        format!(
            ":: {}:{} :: annotation content",
            self.label, self.parameters
        )
    }

    /// Generate annotation input string (block)
    #[allow(dead_code)]
    pub fn annotation_block_input(&self) -> String {
        format!(
            r#":: {}:{} ::
    annotation content here"#,
            self.label, self.parameters
        )
    }

    /// Generate definition input string
    #[allow(dead_code)]
    pub fn definition_input(&self) -> String {
        format!(
            r#"{}:{} ::
    definition content here"#,
            self.label, self.parameters
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use txxt::tokenizer::tokenize;

    #[test]
    fn test_parameter_fixtures_consistency() {
        // Test that our fixtures produce expected results
        let simple_params = ParameterFixtures::simple();
        let expected = ParameterFixtures::simple_expected();

        // This test verifies the fixture data is consistent
        assert_eq!(simple_params, "version=3.9");
        assert_eq!(expected, vec![("version", "3.9")]);
    }

    #[test]
    fn test_element_parameter_fixture() {
        let fixture = ElementParameterFixture::verbatim(
            "python",
            ParameterFixtures::simple(),
            ParameterFixtures::simple_expected(),
        );

        let input = fixture.verbatim_input();
        assert!(input.contains(":: python:version=3.9"));
    }

    #[test]
    fn test_extract_parameter_tokens() {
        // This will test the actual parameter extraction once we implement it
        let input = r#"Code:
    test
:: label:key=value"#;

        let tokens = tokenize(input);
        let params = ParameterFixtures::extract_parameter_tokens(&tokens);

        // Should find the parameter token once we implement annotation parameters
        // For now, this validates the extraction function works
        assert!(!params.is_empty()); // Should have parameter tokens
    }
}
