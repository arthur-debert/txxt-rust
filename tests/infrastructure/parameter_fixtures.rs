//! Test fixtures and helpers for parameter testing
//!
//! Provides reusable fixtures for creating and validating parameters across all
//! test suites. This ensures that parameter format changes only require updates
//! in one place.

#![allow(dead_code)] // Functions will be used when tests are migrated (Task 9)

use std::collections::HashMap;
use txxt::ast::elements::components::parameters::Parameters as AstParameters;
use txxt::cst::{
    HighLevelToken, HighLevelTokenBuilder, Position, ScannerToken, ScannerTokenSequence,
};

/// Create AST Parameters node from a map (test fixture)
pub fn create_ast_parameters(map: HashMap<String, String>) -> AstParameters {
    AstParameters {
        map,
        tokens: ScannerTokenSequence::new(),
    }
}

/// Create AST Parameters node from key-value pairs (test fixture)
pub fn create_ast_parameters_from_pairs(pairs: Vec<(&str, &str)>) -> AstParameters {
    let map: HashMap<String, String> = pairs
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    create_ast_parameters(map)
}

/// Extract parameters from scanner tokens (test helper)
///
/// Takes scanner tokens and attempts to build a Parameters high-level token,
/// returning the extracted HashMap.
pub fn extract_parameters_from_scanner_tokens(
    scanner_tokens: &[ScannerToken],
) -> Option<HashMap<String, String>> {
    if let Some(HighLevelToken::Parameters { params, .. }) =
        HighLevelTokenBuilder::parameters_from_scanner_tokens(scanner_tokens)
    {
        Some(params)
    } else {
        None
    }
}

/// Parse parameter string and extract HashMap (test helper)
pub fn parse_parameter_string(input: &str) -> HashMap<String, String> {
    let start_pos = Position { row: 0, column: 0 };
    let scanner_tokens = txxt::cst::scan_parameter_string(input, start_pos);
    extract_parameters_from_scanner_tokens(&scanner_tokens).unwrap_or_default()
}

/// Extract parameters from any token sequence by finding parameter-like patterns
///
/// This scans through tokens looking for Identifier-Equals-Value patterns and
/// extracts them into a parameter HashMap. Useful for integration tests.
/// Parameters are extracted after the label (first text token after initial marker).
pub fn extract_parameters_from_tokens(tokens: &[ScannerToken]) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let mut i = 0;

    // Skip initial TxxtMarker and whitespace
    while i < tokens.len() {
        match &tokens[i] {
            ScannerToken::TxxtMarker { .. } | ScannerToken::Whitespace { .. } => {
                i += 1;
            }
            _ => break,
        }
    }

    // Skip the label (first Text/Identifier token)
    if i < tokens.len()
        && matches!(
            &tokens[i],
            ScannerToken::Text { .. } | ScannerToken::Identifier { .. }
        )
    {
        i += 1;
    }

    // Skip whitespace after label
    while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. }) {
        i += 1;
    }

    // Now extract parameters
    while i < tokens.len() {
        // Stop at structural tokens
        match &tokens[i] {
            ScannerToken::TxxtMarker { .. } | ScannerToken::Newline { .. } => break,
            _ => {}
        }

        // Try to find Identifier/Text tokens
        let key = match &tokens[i] {
            ScannerToken::Identifier { content, .. } | ScannerToken::Text { content, .. } => {
                content.clone()
            }
            _ => {
                i += 1;
                continue;
            }
        };
        i += 1;

        // Skip whitespace
        while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. }) {
            i += 1;
        }

        // Look for equals
        if i < tokens.len() && matches!(&tokens[i], ScannerToken::Equals { .. }) {
            i += 1;

            // Skip whitespace
            while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. }) {
                i += 1;
            }

            // Get value
            if i < tokens.len() {
                let value = match &tokens[i] {
                    ScannerToken::Text { content, .. } => content.clone(),
                    ScannerToken::QuotedString { content, .. } => content.clone(),
                    ScannerToken::Identifier { content, .. } => content.clone(),
                    _ => {
                        i += 1;
                        continue;
                    }
                };

                params.insert(key, value);
                i += 1;
            }
        } else {
            // Boolean shorthand - key without value (only if next is comma or structural token)
            if i < tokens.len() {
                match &tokens[i] {
                    ScannerToken::Comma { .. }
                    | ScannerToken::TxxtMarker { .. }
                    | ScannerToken::Whitespace { .. } => {
                        params.insert(key, "true".to_string());
                    }
                    _ => {}
                }
            } else {
                // End of tokens - treat as boolean
                params.insert(key, "true".to_string());
            }
        }
    }

    params
}

/// Check if tokens contain a specific parameter key-value pair
pub fn tokens_contain_parameter(
    tokens: &[ScannerToken],
    expected_key: &str,
    expected_value: &str,
) -> bool {
    let params = extract_parameters_from_tokens(tokens);
    params.get(expected_key) == Some(&expected_value.to_string())
}

/// Count parameter key-value pairs in token sequence
pub fn count_parameters_in_tokens(tokens: &[ScannerToken]) -> usize {
    extract_parameters_from_tokens(tokens).len()
}

/// Assert that parameters match expected values (test helper)
pub fn assert_parameters_match(actual: &HashMap<String, String>, expected: &[(&str, &str)]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "Parameter count mismatch. Actual: {:?}, Expected: {:?}",
        actual,
        expected
    );

    for (key, value) in expected {
        assert_eq!(
            actual.get(*key),
            Some(&value.to_string()),
            "Parameter '{}' mismatch. Actual: {:?}, Expected: {}",
            key,
            actual,
            value
        );
    }
}

/// Extract parameters from a verbatim label string (e.g., "python:version=3.9,style=pep8")
///
/// At the scanner level, verbatim labels include parameters in the label string.
/// This function splits the label at the first colon and parses the parameter portion.
pub fn extract_parameters_from_verbatim_label(label_raw: &str) -> HashMap<String, String> {
    // Find the first colon to split label from parameters
    if let Some(colon_pos) = label_raw.find(':') {
        let param_string = &label_raw[colon_pos + 1..];
        parse_parameter_string(param_string)
    } else {
        HashMap::new()
    }
}
