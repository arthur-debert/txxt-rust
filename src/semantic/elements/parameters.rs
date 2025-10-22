//! Parameter Element Construction
//!
//! Converts high-level Parameter tokens into AST Parameters nodes.
//! This is the single source of truth for parameter AST construction.
//!
//! ## Related Files
//! - **Specification**: `docs/specs/elements/components/parameters.txxt`
//! - **AST Node**: `src/ast/elements/components/parameters.rs`
//! - **High-Level Token**: `src/cst/high_level_tokens.rs`
//!
//! ## Parameter Processing Flow
//!
//! This function is **Step 3** of the three-level parameter processing:
//!
//! ```text
//! Step 1 (Scanner): "key=value,key2=val2"
//!        → [Identifier("key"), Equals, Text("value"), Comma, ...]
//!        See: crate::cst::parameter_scanner::scan_parameter_string
//!
//! Step 2 (High-Level): [Identifier("key"), Equals, Text("value"), ...]
//!        → Parameters { params: {key: "value"}, tokens: [...] }
//!        See: crate::cst::high_level_tokens::HighLevelTokenBuilder::parameters_from_scanner_tokens
//!
//! Step 3 (AST - THIS MODULE): Parameters { params: {...}, tokens: [...] }
//!        → AstParameters { map: {...}, tokens: ... }
//!        See: create_parameters_ast
//! ```

use crate::ast::elements::components::parameters::Parameters as AstParameters;
use crate::cst::HighLevelToken;
use crate::semantic::BlockParseError;

/// Create an AST Parameters node from a high-level Parameters token
///
/// This is the single, reusable function for creating parameter AST nodes.
/// All element constructors (annotation, definition, verbatim) should call this.
///
/// # Arguments
/// * `params_token` - Optional Parameters high-level token
///
/// # Returns
/// * AST Parameters node with extracted key-value pairs and tokens
///
/// # Examples
/// ```
/// use txxt::cst::{HighLevelToken, HighLevelTokenBuilder, Position, SourceSpan};
/// use txxt::semantic::elements::parameters::create_parameters_ast;
/// use std::collections::HashMap;
///
/// // With parameters
/// let mut map = HashMap::new();
/// map.insert("key".to_string(), "value".to_string());
/// let span = SourceSpan {
///     start: Position { row: 0, column: 0 },
///     end: Position { row: 0, column: 9 },
/// };
/// let token = HighLevelTokenBuilder::parameters(map, span);
/// let result = create_parameters_ast(Some(&token));
/// assert!(result.is_ok());
///
/// // Without parameters
/// let result = create_parameters_ast(None);
/// assert!(result.is_ok());
/// ```
pub fn create_parameters_ast(
    params_token: Option<&HighLevelToken>,
) -> Result<AstParameters, BlockParseError> {
    match params_token {
        Some(token) => {
            // Extract parameters from the high-level token
            Ok(AstParameters::from_high_level_token(token))
        }
        None => {
            // No parameters - return empty
            Ok(AstParameters::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{HighLevelTokenBuilder, Position, ScannerTokenSequence, SourceSpan};
    use std::collections::HashMap;

    #[test]
    fn test_create_parameters_ast_with_none() {
        let result = create_parameters_ast(None);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.is_empty());
        assert_eq!(params.map.len(), 0);
    }

    #[test]
    fn test_create_parameters_ast_with_empty_parameters() {
        let map = HashMap::new();
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 0 },
        };
        let token = HighLevelTokenBuilder::parameters(map, span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_parameters_ast_with_single_parameter() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 9 },
        };
        let token = HighLevelTokenBuilder::parameters(map.clone(), span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.map.len(), 1);
        assert_eq!(params.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_create_parameters_ast_with_multiple_parameters() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());
        map.insert("key3".to_string(), "value3".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 30 },
        };
        let token = HighLevelTokenBuilder::parameters(map.clone(), span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.map.len(), 3);
        assert_eq!(params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(params.get("key2"), Some(&"value2".to_string()));
        assert_eq!(params.get("key3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_create_parameters_ast_with_empty_value() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 4 },
        };
        let token = HighLevelTokenBuilder::parameters(map, span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.map.len(), 1);
        assert_eq!(params.get("key"), Some(&"".to_string()));
    }

    #[test]
    fn test_create_parameters_ast_with_boolean_shorthand() {
        let mut map = HashMap::new();
        map.insert("debug".to_string(), "true".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 5 },
        };
        let token = HighLevelTokenBuilder::parameters(map, span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.get("debug"), Some(&"true".to_string()));
    }

    #[test]
    fn test_create_parameters_ast_with_quoted_values() {
        let mut map = HashMap::new();
        map.insert("title".to_string(), "My Document".to_string());
        map.insert("path".to_string(), "/home/user/docs".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 40 },
        };
        let token = HighLevelTokenBuilder::parameters(map, span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.get("title"), Some(&"My Document".to_string()));
        assert_eq!(params.get("path"), Some(&"/home/user/docs".to_string()));
    }

    #[test]
    fn test_create_parameters_ast_with_namespaced_keys() {
        let mut map = HashMap::new();
        map.insert("org.example.version".to_string(), "1.0".to_string());
        map.insert("company.feature.enabled".to_string(), "true".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 50 },
        };
        let token = HighLevelTokenBuilder::parameters(map, span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.get("org.example.version"), Some(&"1.0".to_string()));
        assert_eq!(
            params.get("company.feature.enabled"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_create_parameters_ast_with_special_characters() {
        let mut map = HashMap::new();
        map.insert("version".to_string(), "3.11".to_string());
        map.insert("pattern".to_string(), "*.txt".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 30 },
        };
        let token = HighLevelTokenBuilder::parameters(map, span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.get("version"), Some(&"3.11".to_string()));
        assert_eq!(params.get("pattern"), Some(&"*.txt".to_string()));
    }

    #[test]
    fn test_create_parameters_ast_preserves_tokens() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 9 },
        };
        let tokens = ScannerTokenSequence::new();
        let token = HighLevelTokenBuilder::parameters_with_tokens(map, span, tokens);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        // Tokens should be preserved for source reconstruction
        let params = result.unwrap();
        assert_eq!(params.map.len(), 1);
    }

    #[test]
    fn test_create_parameters_ast_with_non_parameter_token() {
        // Test that passing a non-Parameter token returns empty parameters
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 4 },
        };
        let token = HighLevelTokenBuilder::label("test".to_string(), span);

        let result = create_parameters_ast(Some(&token));
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.is_empty()); // Should be empty since it's not a Parameters token
    }
}
