//! Test fixtures and helpers for parameter testing
//!
//! Provides reusable fixtures for creating and validating parameters across all
//! test suites. This ensures that parameter format changes only require updates
//! in one place.

use std::collections::HashMap;
use txxt::ast::elements::components::parameters::Parameters as AstParameters;
use txxt::cst::{HighLevelToken, HighLevelTokenBuilder, Position, ScannerToken, ScannerTokenSequence};

/// Create AST Parameters node from a map (test fixture)
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// let mut params = HashMap::new();
/// params.insert("key".to_string(), "value".to_string());
/// let ast_params = create_ast_parameters(params);
/// ```
pub fn create_ast_parameters(map: HashMap<String, String>) -> AstParameters {
    AstParameters {
        map,
        tokens: ScannerTokenSequence::new(),
    }
}

/// Create AST Parameters node from key-value pairs (test fixture)
///
/// # Example
/// ```
/// let ast_params = create_ast_parameters_from_pairs(vec![
///     ("key", "value"),
///     ("debug", "true"),
/// ]);
/// ```
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

/// Extract parameters from any token sequence by finding parameter-like patterns
///
/// This scans through tokens looking for Identifier-Equals-Value patterns and
/// extracts them into a parameter HashMap. Useful for integration tests.
pub fn extract_parameters_from_tokens(tokens: &[ScannerToken]) -> HashMap<String, String> {
    // Look for sequences: Identifier, Equals, (Text|QuotedString|Identifier)
    let mut params = HashMap::new();
    let mut i = 0;
    
    while i < tokens.len() {
        // Skip non-parameter tokens
        while i < tokens.len() {
            match &tokens[i] {
                ScannerToken::Identifier { .. } => break,
                ScannerToken::Colon { .. } => {
                    // Colon might precede parameters, skip it
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
///
/// Convenience function for assertions in tests.
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

/// Parse parameter string and extract HashMap (test helper)
///
/// Convenience function that scans a parameter string and extracts the HashMap.
pub fn parse_parameter_string(input: &str) -> HashMap<String, String> {
    let start_pos = Position { row: 0, column: 0 };
    let scanner_tokens = txxt::cst::scan_parameter_string(input, start_pos);
    extract_parameters_from_scanner_tokens(&scanner_tokens).unwrap_or_default()
}

/// Check if scanner tokens contain a parameter with given key-value (test helper)
///
/// This is useful for integration tests that scan entire documents and need to
/// verify parameter extraction.
pub fn has_parameter_in_tokens(
    tokens: &[ScannerToken],
    expected_key: &str,
    expected_value: &str,
) -> bool {
    // Find all parameter-related tokens and build parameters
    let param_indices = find_parameter_token_sequences(tokens);
    
    for (start, end) in param_indices {
        if let Some(params) = extract_parameters_from_scanner_tokens(&tokens[start..=end]) {
            if params.get(expected_key) == Some(&expected_value.to_string()) {
                return true;
            }
        }
    }
    
    false
}

/// Find sequences of tokens that form parameters (test helper)
///
/// Scans for patterns like: Identifier, Equals, (Text|QuotedString)
/// Returns ranges (start_idx, end_idx) of parameter sequences.
fn find_parameter_token_sequences(tokens: &[ScannerToken]) -> Vec<(usize, usize)> {
    let mut sequences = Vec::new();
    let mut i = 0;
    
    while i < tokens.len() {
        // Skip to potential parameter start (Identifier after colon or at start)
        while i < tokens.len() {
            if matches!(&tokens[i], ScannerToken::Identifier { .. }) {
                // Check if this could be a parameter key
                let mut j = i + 1;
                
                // Skip whitespace
                while j < tokens.len() && matches!(&tokens[j], ScannerToken::Whitespace { .. }) {
                    j += 1;
                }
                
                // Look for equals
                if j < tokens.len() && matches!(&tokens[j], ScannerToken::Equals { .. }) {
                    // This is a parameter - find the end of the parameter sequence
                    let start = i;
                    let end = find_parameter_sequence_end(tokens, i);
                    sequences.push((start, end));
                    i = end + 1;
                    break;
                } else if j < tokens.len() && matches!(&tokens[j], ScannerToken::Comma { .. }) {
                    // Boolean shorthand
                    sequences.push((i, j));
                    i = j + 1;
                    break;
                }
            }
            i += 1;
        }
    }
    
    sequences
}

/// Find the end of a parameter token sequence starting at given index
fn find_parameter_sequence_end(tokens: &[ScannerToken], start: usize) -> usize {
    let mut i = start;
    
    while i < tokens.len() {
        match &tokens[i] {
            ScannerToken::Comma { .. } => {
                // End of this parameter, but might be more parameters after
                if i + 1 < tokens.len() {
                    // Skip whitespace
                    let mut j = i + 1;
                    while j < tokens.len() && matches!(&tokens[j], ScannerToken::Whitespace { .. }) {
                        j += 1;
                    }
                    
                    // Check if next is another identifier (more parameters)
                    if j < tokens.len() && matches!(&tokens[j], ScannerToken::Identifier { .. }) {
                        i += 1;
                        continue;
                    }
                }
                return i - 1;
            }
            ScannerToken::TxxtMarker { .. } | ScannerToken::Newline { .. } => {
                // End of parameter sequence
                return if i > start { i - 1 } else { start };
            }
            _ => i += 1,
        }
    }
    
    tokens.len() - 1
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
