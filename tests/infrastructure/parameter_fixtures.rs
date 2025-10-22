//! Test fixtures and helpers for parameter testing
//!
//! Provides reusable fixtures for creating and validating parameters across all
//! test suites. This ensures that parameter format changes only require updates
//! in one place.

use std::collections::HashMap;
use txxt::ast::elements::components::parameters::Parameters as AstParameters;
use txxt::cst::{HighLevelToken, HighLevelTokenBuilder, Position, ScannerToken, ScannerTokenSequence};

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
pub fn extract_parameters_from_tokens(tokens: &[ScannerToken]) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let mut i = 0;
    
    while i < tokens.len() {
        // Skip to potential parameter tokens
        while i < tokens.len() {
            match &tokens[i] {
                ScannerToken::Identifier { .. } => break,
                ScannerToken::Colon { .. } => {
                    i += 1;
                    continue;
                }
                _ => i += 1,
            }
        }
        
        if i >= tokens.len() {
            break;
        }
        
        // Try to parse Identifier = Value pattern
        if let ScannerToken::Identifier { content: key, .. } = &tokens[i] {
            let key = key.clone();
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
                        _ => continue,
                    };
                    
                    params.insert(key, value);
                    i += 1;
                }
            } else {
                // Boolean shorthand - key without value
                params.insert(key, "true".to_string());
            }
        } else {
            i += 1;
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
pub fn assert_parameters_match(
    actual: &HashMap<String, String>,
    expected: &[(&str, &str)],
) {
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
