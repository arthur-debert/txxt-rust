//! Tests for parameter tokenization functionality
//!
//! Tests the key=value,key2=value2 parameter syntax parsing with support for:
//! - Basic key=value pairs
//! - Quoted strings with escape sequences
//! - Boolean shorthand (key without value)
//! - Namespaced keys (org.example.metadata)

use proptest::prelude::*;
use std::collections::HashMap;
use txxt::cst::{Position, ScannerToken};
use txxt::syntax::elements::components::parameters::{parse_parameters, ParameterLexer};

/// Mock lexer for testing parameter parsing
struct MockParameterLexer {
    position: Position,
    chars_consumed: usize,
}

impl MockParameterLexer {
    fn new() -> Self {
        Self {
            position: Position { row: 0, column: 0 },
            chars_consumed: 0,
        }
    }
}

impl ParameterLexer for MockParameterLexer {
    fn current_position(&self) -> Position {
        self.position
    }

    fn peek(&self) -> Option<char> {
        None // Not used in current implementation
    }

    fn advance(&mut self) -> Option<char> {
        self.chars_consumed += 1;
        self.position.column += 1;
        None
    }

    fn is_at_end(&self) -> bool {
        false // Not used in current implementation
    }

    fn get_input(&self) -> &[char] {
        &[] // Mock implementation for testing
    }
}

/// Extract parameter data from tokens for easier testing
fn extract_parameters(tokens: &[ScannerToken]) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for token in tokens {
        if let ScannerToken::Parameter { key, value, .. } = token {
            params.insert(key.clone(), value.clone());
        }
    }
    params
}

#[cfg(test)]
mod basic_parameter_tests {
    use super::*;

    #[test]
    fn simple_key_value_pairs() {
        let test_cases = [
            ("key=value", vec![("key", "value")]),
            ("debug=true", vec![("debug", "true")]),
            ("version=3.11", vec![("version", "3.11")]),
            ("style=minimal", vec![("style", "minimal")]),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn multiple_parameters() {
        let test_cases = [
            (
                "key1=value1,key2=value2",
                vec![("key1", "value1"), ("key2", "value2")],
            ),
            (
                "debug=true,version=3.11,style=minimal",
                vec![("debug", "true"), ("version", "3.11"), ("style", "minimal")],
            ),
            (
                "a=1,b=2,c=3,d=4",
                vec![("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")],
            ),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn boolean_shorthand() {
        let test_cases = [
            ("debug", vec![("debug", "true")]),
            ("verbose", vec![("verbose", "true")]),
            (
                "debug,verbose",
                vec![("debug", "true"), ("verbose", "true")],
            ),
            (
                "debug,version=3.11,verbose",
                vec![("debug", "true"), ("version", "3.11"), ("verbose", "true")],
            ),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }
}

#[cfg(test)]
mod quoted_string_tests {
    use super::*;

    #[test]
    fn basic_quoted_strings() {
        let test_cases = [
            (r#"title="My Document""#, vec![("title", "My Document")]),
            (
                r#"path="/home/user/docs""#,
                vec![("path", "/home/user/docs")],
            ),
            (r#"tags="red,blue,green""#, vec![("tags", "red,blue,green")]),
            (r#"note=" important ""#, vec![("note", " important ")]),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn escaped_sequences() {
        let test_cases = [
            (
                r#"message="She said, \"Hello!\"""#,
                vec![("message", r#"She said, "Hello!""#)],
            ),
            (
                r#"path="C:\\Users\\Name\\Documents""#,
                vec![("path", r"C:\Users\Name\Documents")],
            ),
            (
                r#"multiline="Line 1\nLine 2""#,
                vec![("multiline", "Line 1\nLine 2")],
            ),
            (
                r#"tab="Column1\tColumn2""#,
                vec![("tab", "Column1\tColumn2")],
            ),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn mixed_quoted_unquoted() {
        let test_cases = [
            (
                r#"debug=true,title="My Document",version=3.11"#,
                vec![
                    ("debug", "true"),
                    ("title", "My Document"),
                    ("version", "3.11"),
                ],
            ),
            (
                r#"verbose,message="Hello, World!",timeout=30"#,
                vec![
                    ("verbose", "true"),
                    ("message", "Hello, World!"),
                    ("timeout", "30"),
                ],
            ),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }
}

#[cfg(test)]
mod namespaced_key_tests {
    use super::*;

    #[test]
    fn namespaced_keys() {
        let test_cases = [
            (
                "org.example.metadata=value",
                vec![("org.example.metadata", "value")],
            ),
            (
                "company.product.feature=enabled",
                vec![("company.product.feature", "enabled")],
            ),
            (
                "txxt.internal.parser=strict",
                vec![("txxt.internal.parser", "strict")],
            ),
            (
                "user.name.preference=dark",
                vec![("user.name.preference", "dark")],
            ),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn mixed_namespaced_and_simple() {
        let test_cases = [
            (
                "debug=true,org.example.version=2.0,style=minimal",
                vec![
                    ("debug", "true"),
                    ("org.example.version", "2.0"),
                    ("style", "minimal"),
                ],
            ),
            (
                "company.auth.enabled,timeout=30,user.theme.dark=true",
                vec![
                    ("company.auth.enabled", "true"),
                    ("timeout", "30"),
                    ("user.theme.dark", "true"),
                ],
            ),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn empty_and_whitespace() {
        let test_cases = [
            ("", vec![]),
            ("   ", vec![]),
            ("key=", vec![("key", "")]),
            (r#"key="""#, vec![("key", "")]),
            ("  key=value  ", vec![("key", "value")]),
            (
                "key1=value1,  key2=value2",
                vec![("key1", "value1"), ("key2", "value2")],
            ),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn special_characters_in_values() {
        let test_cases = [
            (
                "url=https://example.com",
                vec![("url", "https://example.com")],
            ),
            ("pattern=*.txt", vec![("pattern", "*.txt")]),
            ("expression=a+b-c", vec![("expression", "a+b-c")]),
            ("range=1-10", vec![("range", "1-10")]),
        ];

        for (input, expected) in test_cases {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, input);
            let params = extract_parameters(&tokens);

            let expected_map: HashMap<String, String> = expected
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            assert_eq!(params, expected_map, "Failed for input: '{}'", input);
        }
    }
}

#[cfg(test)]
mod property_based_tests {
    use super::*;

    // Generate valid parameter keys (identifier starting with letter, not ending with period)
    prop_compose! {
        fn valid_key()(
            first in "[a-zA-Z_]",
            rest in "[a-zA-Z0-9_-]*"
        ) -> String {
            let mut key = format!("{}{}", first, rest);
            // Ensure we don't end with period
            while key.ends_with('.') {
                key.pop();
            }
            // Ensure we have at least one character
            if key.is_empty() {
                key = first.to_string();
            }
            key
        }
    }

    // Generate valid unquoted values (no spaces, commas, quotes)
    prop_compose! {
        fn valid_unquoted_value()(
            value in "[a-zA-Z0-9+\\-*/:.#@]+",
        ) -> String {
            value
        }
    }

    proptest! {
        #[test]
        fn single_parameter_roundtrip(
            key in valid_key(),
            value in valid_unquoted_value()
        ) {
            let input = format!("{}={}", key, value);
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, &input);
            let params = extract_parameters(&tokens);

            prop_assert_eq!(params.len(), 1);
            prop_assert_eq!(params.get(&key), Some(&value));
        }

        #[test]
        fn boolean_shorthand_roundtrip(key in valid_key()) {
            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, &key);
            let params = extract_parameters(&tokens);

            prop_assert_eq!(params.len(), 1);
            let expected = "true".to_string();
            prop_assert_eq!(params.get(&key), Some(&expected));
        }

        #[test]
        fn multiple_parameters_preserve_count(
            params in prop::collection::vec((valid_key(), valid_unquoted_value()), 1..5)
        ) {
            // Create unique keys by appending index
            let unique_params: Vec<_> = params
                .into_iter()
                .enumerate()
                .map(|(i, (key, value))| (format!("{}_{}", key, i), value))
                .collect();

            let input = unique_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");

            let mut lexer = MockParameterLexer::new();
            let tokens = parse_parameters(&mut lexer, &input);
            let parsed_params = extract_parameters(&tokens);

            prop_assert_eq!(parsed_params.len(), unique_params.len());

            for (expected_key, expected_value) in unique_params {
                prop_assert_eq!(parsed_params.get(&expected_key), Some(&expected_value));
            }
        }
    }
}

#[cfg(test)]
mod token_span_tests {
    use super::*;

    #[test]
    fn parameter_tokens_have_valid_spans() {
        let input = "key1=value1,key2=value2";
        let mut lexer = MockParameterLexer::new();
        let tokens = parse_parameters(&mut lexer, input);

        assert_eq!(tokens.len(), 2);

        for token in &tokens {
            if let ScannerToken::Parameter { span, .. } = token {
                // All spans should have valid positions
                assert!(span.end.column >= span.start.column);
                assert_eq!(span.start.row, span.end.row); // Single line parameters
            } else {
                panic!("Expected Parameter token, got: {:?}", token);
            }
        }
    }

    #[test]
    fn parameter_token_content_matches() {
        let input = "debug=true";
        let mut lexer = MockParameterLexer::new();
        let tokens = parse_parameters(&mut lexer, input);

        assert_eq!(tokens.len(), 1);

        if let ScannerToken::Parameter { key, value, .. } = &tokens[0] {
            assert_eq!(key, "debug");
            assert_eq!(value, "true");
        } else {
            panic!("Expected Parameter token");
        }
    }
}
