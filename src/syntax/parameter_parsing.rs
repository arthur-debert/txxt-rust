//! Parameter Parsing Functions
//!
//! Pure functions for parsing and validating txxt parameters.
//! These extracted functions improve testability and maintainability
//! per the progressive-quality-improvements plan (Phase 2, section 3.6).
//!
//! See: docs/proposals/progressive-quality-improvements.txxt
//! See: docs/specs/elements/components/parameters.txxt
//!
//! ## Parameter Syntax
//!
//! Parameters follow the format: `key=value,key2=value2`
//! - Keys: Letters, digits, underscore, dash, period (for namespaces)
//! - Values: Unquoted (simple) or quoted (with spaces/special chars)
//! - Boolean shorthand: `debug` (implies `debug=true`)
//! - Quoted values support escape sequences: `\"`, `\\`, `\n`

use crate::cst::ScannerToken;
use std::collections::HashMap;

/// Validate a parameter key according to txxt rules
///
/// Valid keys must:
/// - Start with a letter (a-z, A-Z)
/// - Contain only letters, digits, underscore, dash, period
/// - Use period only for namespace separation (not at start/end, no doubles)
/// - Be at least one character long
///
/// # Arguments
/// * `key` - The parameter key to validate
///
/// # Returns
/// * `true` if key is valid, `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::parameter_parsing::*;
/// assert!(is_valid_parameter_key("version"));
/// assert!(is_valid_parameter_key("org.example.key"));
/// assert!(is_valid_parameter_key("line-numbers"));
/// assert!(is_valid_parameter_key("_private"));
///
/// assert!(!is_valid_parameter_key("123invalid")); // Starts with digit
/// assert!(!is_valid_parameter_key("-invalid"));   // Starts with dash
/// assert!(!is_valid_parameter_key("key..name"));  // Double period
/// assert!(!is_valid_parameter_key("key."));       // Ends with period
/// assert!(!is_valid_parameter_key(".key"));       // Starts with period
/// assert!(!is_valid_parameter_key(""));           // Empty
/// ```
pub fn is_valid_parameter_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }

    let chars: Vec<char> = key.chars().collect();

    // Must start with letter or underscore
    if !chars[0].is_ascii_alphabetic() && chars[0] != '_' {
        return false;
    }

    // Cannot end with period
    if chars[chars.len() - 1] == '.' {
        return false;
    }

    // Check each character and look for invalid patterns
    let mut prev_was_period = false;
    for &ch in &chars {
        // Allow letters, digits, underscore, dash, period
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.' {
            return false;
        }

        // No double periods
        if ch == '.' {
            if prev_was_period {
                return false;
            }
            prev_was_period = true;
        } else {
            prev_was_period = false;
        }
    }

    true
}

/// Parse a single parameter pair from scanner tokens
///
/// Parses one `key=value` or `key` (boolean shorthand) from the token stream.
///
/// # Arguments
/// * `tokens` - Scanner token sequence
/// * `start` - Starting position in token sequence
///
/// # Returns
/// * `Some((key, value, consumed))` - Key, value, and number of tokens consumed
/// * `None` - If no valid parameter pair found at this position
///
/// # Examples
/// ```
/// # use txxt::syntax::parameter_parsing::*;
/// # use txxt::cst::{ScannerToken, SourceSpan, Position};
/// let tokens = vec![
///     ScannerToken::Identifier {
///         content: "debug".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 0 },
///             end: Position { row: 0, column: 5 },
///         },
///     },
///     ScannerToken::Equals {
///         span: SourceSpan {
///             start: Position { row: 0, column: 5 },
///             end: Position { row: 0, column: 6 },
///         },
///     },
///     ScannerToken::Text {
///         content: "true".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 6 },
///             end: Position { row: 0, column: 10 },
///         },
///     },
/// ];
///
/// let result = parse_parameter_pair(&tokens, 0);
/// assert_eq!(result, Some(("debug".to_string(), "true".to_string(), 3)));
/// ```
pub fn parse_parameter_pair(
    tokens: &[ScannerToken],
    start: usize,
) -> Option<(String, String, usize)> {
    if start >= tokens.len() {
        return None;
    }

    let mut i = start;

    // Skip leading whitespace and colons
    while i < tokens.len()
        && (matches!(&tokens[i], ScannerToken::Whitespace { .. })
            || matches!(&tokens[i], ScannerToken::Colon { .. }))
    {
        i += 1;
    }

    if i >= tokens.len() {
        return None;
    }

    // Extract key (accept Identifier or Text)
    let key = match &tokens[i] {
        ScannerToken::Identifier { content, .. } => content.clone(),
        ScannerToken::Text { content, .. } => content.clone(),
        _ => return None,
    };
    i += 1;

    // Skip whitespace after key
    while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. }) {
        i += 1;
    }

    // Check for equals sign
    if i < tokens.len() && matches!(&tokens[i], ScannerToken::Equals { .. }) {
        i += 1; // Skip equals

        // Skip whitespace after equals
        while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. }) {
            i += 1;
        }

        // Extract value (or empty string if no value)
        let value = if i < tokens.len() {
            match &tokens[i] {
                ScannerToken::Text { content, .. } => {
                    i += 1;
                    content.clone()
                }
                ScannerToken::QuotedString { content, .. } => {
                    i += 1;
                    content.clone()
                }
                ScannerToken::Identifier { content, .. } => {
                    i += 1;
                    content.clone()
                }
                _ => String::new(), // Empty value (key= with nothing after)
            }
        } else {
            String::new() // Empty value (key= at end)
        };

        Some((key, value, i - start))
    } else {
        // Boolean shorthand - key without value means "true"
        Some((key, "true".to_string(), i - start))
    }
}

/// Extract all parameters from scanner token sequence
///
/// Parses a complete parameter sequence into a HashMap.
/// Handles multiple parameters separated by commas.
///
/// # Arguments
/// * `tokens` - Scanner token sequence representing parameters
///
/// # Returns
/// * `HashMap<String, String>` - Parsed key-value pairs
///   (Empty HashMap if no valid parameters found)
///
/// # Examples
/// ```
/// # use txxt::syntax::parameter_parsing::*;
/// # use txxt::cst::{ScannerToken, SourceSpan, Position};
/// let tokens = vec![
///     ScannerToken::Identifier {
///         content: "lang".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 0 },
///             end: Position { row: 0, column: 4 },
///         },
///     },
///     ScannerToken::Equals {
///         span: SourceSpan {
///             start: Position { row: 0, column: 4 },
///             end: Position { row: 0, column: 5 },
///         },
///     },
///     ScannerToken::Text {
///         content: "rust".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 5 },
///             end: Position { row: 0, column: 9 },
///         },
///     },
///     ScannerToken::Comma {
///         span: SourceSpan {
///             start: Position { row: 0, column: 9 },
///             end: Position { row: 0, column: 10 },
///         },
///     },
///     ScannerToken::Identifier {
///         content: "debug".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 10 },
///             end: Position { row: 0, column: 15 },
///         },
///     },
/// ];
///
/// let params = extract_parameters_from_tokens(&tokens);
/// assert_eq!(params.get("lang"), Some(&"rust".to_string()));
/// assert_eq!(params.get("debug"), Some(&"true".to_string()));
/// ```
pub fn extract_parameters_from_tokens(tokens: &[ScannerToken]) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let mut i = 0;

    while i < tokens.len() {
        if let Some((key, value, consumed)) = parse_parameter_pair(tokens, i) {
            params.insert(key, value);
            i += consumed;

            // Skip comma and whitespace before next parameter
            while i < tokens.len()
                && (matches!(&tokens[i], ScannerToken::Whitespace { .. })
                    || matches!(&tokens[i], ScannerToken::Comma { .. }))
            {
                i += 1;
            }
        } else {
            // No valid parameter, skip this token
            i += 1;
        }
    }

    params
}

/// Parse a boolean value from string
///
/// Recognizes standard boolean representations:
/// - `true` / `false` (preferred)
/// - `yes` / `no`
/// - `on` / `off`
/// - `1` / `0`
///
/// # Arguments
/// * `value` - String value to parse as boolean
///
/// # Returns
/// * `Some(true)` or `Some(false)` if recognized
/// * `None` if not a boolean value
///
/// # Examples
/// ```
/// # use txxt::syntax::parameter_parsing::*;
/// assert_eq!(parse_boolean_value("true"), Some(true));
/// assert_eq!(parse_boolean_value("false"), Some(false));
/// assert_eq!(parse_boolean_value("yes"), Some(true));
/// assert_eq!(parse_boolean_value("no"), Some(false));
/// assert_eq!(parse_boolean_value("on"), Some(true));
/// assert_eq!(parse_boolean_value("off"), Some(false));
/// assert_eq!(parse_boolean_value("1"), Some(true));
/// assert_eq!(parse_boolean_value("0"), Some(false));
/// assert_eq!(parse_boolean_value("maybe"), None);
/// ```
pub fn parse_boolean_value(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    #[test]
    fn test_is_valid_parameter_key_valid() {
        assert!(is_valid_parameter_key("version"));
        assert!(is_valid_parameter_key("lineNumbers"));
        assert!(is_valid_parameter_key("line-numbers"));
        assert!(is_valid_parameter_key("_private"));
        assert!(is_valid_parameter_key("org.example.key"));
        assert!(is_valid_parameter_key("a"));
    }

    #[test]
    fn test_is_valid_parameter_key_invalid() {
        assert!(!is_valid_parameter_key("")); // Empty
        assert!(!is_valid_parameter_key("123invalid")); // Starts with digit
        assert!(!is_valid_parameter_key("-invalid")); // Starts with dash
        assert!(!is_valid_parameter_key(".key")); // Starts with period
        assert!(!is_valid_parameter_key("key.")); // Ends with period
        assert!(!is_valid_parameter_key("key..name")); // Double period
        assert!(!is_valid_parameter_key("key name")); // Space
        assert!(!is_valid_parameter_key("key=value")); // Equals sign
    }

    #[test]
    fn test_parse_parameter_pair_with_value() {
        let tokens = vec![
            ScannerToken::Identifier {
                content: "lang".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 4 },
                },
            },
            ScannerToken::Equals {
                span: SourceSpan {
                    start: Position { row: 0, column: 4 },
                    end: Position { row: 0, column: 5 },
                },
            },
            ScannerToken::Text {
                content: "rust".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 5 },
                    end: Position { row: 0, column: 9 },
                },
            },
        ];

        let result = parse_parameter_pair(&tokens, 0);
        assert_eq!(result, Some(("lang".to_string(), "rust".to_string(), 3)));
    }

    #[test]
    fn test_parse_parameter_pair_boolean_shorthand() {
        let tokens = vec![ScannerToken::Identifier {
            content: "debug".to_string(),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 5 },
            },
        }];

        let result = parse_parameter_pair(&tokens, 0);
        assert_eq!(result, Some(("debug".to_string(), "true".to_string(), 1)));
    }

    #[test]
    fn test_parse_parameter_pair_quoted_value() {
        let tokens = vec![
            ScannerToken::Identifier {
                content: "title".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 5 },
                },
            },
            ScannerToken::Equals {
                span: SourceSpan {
                    start: Position { row: 0, column: 5 },
                    end: Position { row: 0, column: 6 },
                },
            },
            ScannerToken::QuotedString {
                content: "My Document".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 6 },
                    end: Position { row: 0, column: 19 },
                },
            },
        ];

        let result = parse_parameter_pair(&tokens, 0);
        assert_eq!(
            result,
            Some(("title".to_string(), "My Document".to_string(), 3))
        );
    }

    #[test]
    fn test_extract_parameters_from_tokens_multiple() {
        let tokens = vec![
            ScannerToken::Identifier {
                content: "lang".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 4 },
                },
            },
            ScannerToken::Equals {
                span: SourceSpan {
                    start: Position { row: 0, column: 4 },
                    end: Position { row: 0, column: 5 },
                },
            },
            ScannerToken::Text {
                content: "rust".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 5 },
                    end: Position { row: 0, column: 9 },
                },
            },
            ScannerToken::Comma {
                span: SourceSpan {
                    start: Position { row: 0, column: 9 },
                    end: Position { row: 0, column: 10 },
                },
            },
            ScannerToken::Identifier {
                content: "debug".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 10 },
                    end: Position { row: 0, column: 15 },
                },
            },
        ];

        let params = extract_parameters_from_tokens(&tokens);
        assert_eq!(params.get("lang"), Some(&"rust".to_string()));
        assert_eq!(params.get("debug"), Some(&"true".to_string()));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_parse_boolean_value() {
        assert_eq!(parse_boolean_value("true"), Some(true));
        assert_eq!(parse_boolean_value("TRUE"), Some(true));
        assert_eq!(parse_boolean_value("false"), Some(false));
        assert_eq!(parse_boolean_value("FALSE"), Some(false));
        assert_eq!(parse_boolean_value("yes"), Some(true));
        assert_eq!(parse_boolean_value("no"), Some(false));
        assert_eq!(parse_boolean_value("on"), Some(true));
        assert_eq!(parse_boolean_value("off"), Some(false));
        assert_eq!(parse_boolean_value("1"), Some(true));
        assert_eq!(parse_boolean_value("0"), Some(false));
        assert_eq!(parse_boolean_value("maybe"), None);
        assert_eq!(parse_boolean_value(""), None);
    }
}
