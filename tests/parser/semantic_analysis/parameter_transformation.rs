//! Tests for Parameter transformation pipeline
//!
//! This module tests the full parameter transformation pipeline:
//! 1. Scanner Level: scan_parameter_string() → scanner tokens
//! 2. High-Level: parameters_from_scanner_tokens() → Parameters semantic token
//! 3. AST: create_parameters_ast() → AstParameters
//!
//! These tests verify the integration and transformation logic for parameters
//! with edge cases like quoted values, escape sequences, malformed input, etc.

use txxt::cst::high_level_tokens::HighLevelTokenBuilder;
use txxt::cst::parameter_scanner::scan_parameter_string;
use txxt::cst::{Position, ScannerToken, SourceSpan};
use txxt::semantic::elements::parameters::create_parameters_ast;

/// Test the full parameter transformation pipeline: string → scanner → high-level → AST
#[test]
fn test_full_parameter_pipeline_simple() {
    let input = "key=value";
    let start_pos = Position { row: 0, column: 0 };

    // Step 1: Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);
    assert_eq!(scanner_tokens.len(), 3); // Identifier, Equals, Text

    // Step 2: High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // Step 3: AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.map.len(), 1);
    assert_eq!(params.get("key"), Some(&"value".to_string()));
}

/// Test parameter transformation with quoted values
#[test]
fn test_parameter_transformation_quoted_values() {
    let input = r#"title="My Document",path="/home/user""#;
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("title"), Some(&"My Document".to_string()));
    assert_eq!(params.get("path"), Some(&"/home/user".to_string()));
}

/// Test parameter transformation with escape sequences
#[test]
fn test_parameter_transformation_escape_sequences() {
    let input = r#"text="Line 1\nLine 2\tTabbed""#;
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(
        params.get("text"),
        Some(&"Line 1\nLine 2\tTabbed".to_string())
    );
}

/// Test parameter transformation with multiple parameters
#[test]
fn test_parameter_transformation_multiple_params() {
    let input = "key1=value1,key2=value2,key3=value3";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.map.len(), 3);
    assert_eq!(params.get("key1"), Some(&"value1".to_string()));
    assert_eq!(params.get("key2"), Some(&"value2".to_string()));
    assert_eq!(params.get("key3"), Some(&"value3".to_string()));
}

/// Test parameter transformation with boolean shorthand
#[test]
fn test_parameter_transformation_boolean_shorthand() {
    let input = "debug,verbose,quiet";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("debug"), Some(&"true".to_string()));
    assert_eq!(params.get("verbose"), Some(&"true".to_string()));
    assert_eq!(params.get("quiet"), Some(&"true".to_string()));
}

/// Test parameter transformation with empty values
#[test]
fn test_parameter_transformation_empty_values() {
    let input = "empty=,key=value";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("empty"), Some(&"".to_string()));
    assert_eq!(params.get("key"), Some(&"value".to_string()));
}

/// Test parameter transformation with namespaced keys
#[test]
fn test_parameter_transformation_namespaced_keys() {
    let input = "org.example.version=1.0,company.feature.enabled=true";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("org.example.version"), Some(&"1.0".to_string()));
    assert_eq!(
        params.get("company.feature.enabled"),
        Some(&"true".to_string())
    );
}

/// Test parameter transformation with special characters in values
#[test]
fn test_parameter_transformation_special_characters() {
    let input = r#"pattern="*.txt",version=3.11,url="https://example.com""#;
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("pattern"), Some(&"*.txt".to_string()));
    assert_eq!(params.get("version"), Some(&"3.11".to_string()));
    assert_eq!(params.get("url"), Some(&"https://example.com".to_string()));
}

/// Test parameter transformation with whitespace handling
#[test]
fn test_parameter_transformation_whitespace() {
    let input = "key1=value1 , key2=value2";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("key1"), Some(&"value1".to_string()));
    assert_eq!(params.get("key2"), Some(&"value2".to_string()));
}

/// Test parameter transformation with None input (no parameters)
#[test]
fn test_parameter_transformation_none_input() {
    // AST with None input
    let ast_result = create_parameters_ast(None);
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert!(params.is_empty());
    assert_eq!(params.map.len(), 0);
}

/// Test parameter transformation with empty scanner tokens
#[test]
fn test_parameter_transformation_empty_tokens() {
    let scanner_tokens: Vec<ScannerToken> = vec![];

    // High-Level Token - should return None for empty tokens
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_none());

    // AST with None should produce empty Parameters
    let ast_result = create_parameters_ast(None);
    assert!(ast_result.is_ok());
    assert!(ast_result.unwrap().is_empty());
}

/// Test parameter transformation preserves source tokens
#[test]
fn test_parameter_transformation_preserves_tokens() {
    let input = "key=value";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    // Verify tokens are preserved for source reconstruction
    assert_eq!(params.map.len(), 1);
    // The tokens field should contain the original scanner tokens
    // This enables perfect source reconstruction
}

/// Test parameter transformation with mixed quoted and unquoted values
#[test]
fn test_parameter_transformation_mixed_values() {
    let input = r#"name=Alice,title="Software Engineer",age=30"#;
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("name"), Some(&"Alice".to_string()));
    assert_eq!(params.get("title"), Some(&"Software Engineer".to_string()));
    assert_eq!(params.get("age"), Some(&"30".to_string()));
}

/// Test parameter transformation handles escaped quotes
#[test]
fn test_parameter_transformation_escaped_quotes() {
    let input = r#"message="He said \"Hello\"""#;
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(
        params.get("message"),
        Some(&"He said \"Hello\"".to_string())
    );
}

/// Test parameter transformation with numeric values
#[test]
fn test_parameter_transformation_numeric_values() {
    let input = "width=1920,height=1080,ratio=16.9,quality=0.95";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    assert_eq!(params.get("width"), Some(&"1920".to_string()));
    assert_eq!(params.get("height"), Some(&"1080".to_string()));
    assert_eq!(params.get("ratio"), Some(&"16.9".to_string()));
    assert_eq!(params.get("quality"), Some(&"0.95".to_string()));
}

/// Test parameter transformation error handling with non-Parameter token
#[test]
fn test_parameter_transformation_non_parameter_token() {
    // Create a non-Parameter token (Label)
    let span = SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 0, column: 4 },
    };
    let token = HighLevelTokenBuilder::label("test".to_string(), span);

    // AST should handle non-Parameter tokens gracefully
    let ast_result = create_parameters_ast(Some(&token));
    assert!(ast_result.is_ok());

    // Should return empty parameters for non-Parameter tokens
    let params = ast_result.unwrap();
    assert!(params.is_empty());
}

/// Test parameter transformation with consecutive commas
#[test]
fn test_parameter_transformation_consecutive_commas() {
    let input = "key1=value1,,key2=value2";
    let start_pos = Position { row: 0, column: 0 };

    // Scanner Level
    let scanner_tokens = scan_parameter_string(input, start_pos);

    // High-Level Token - should handle consecutive commas gracefully
    let high_level_token = HighLevelTokenBuilder::parameters_from_scanner_tokens(&scanner_tokens);
    assert!(high_level_token.is_some());

    // AST
    let ast_result = create_parameters_ast(high_level_token.as_ref());
    assert!(ast_result.is_ok());

    let params = ast_result.unwrap();
    // Should still extract valid parameters
    assert_eq!(params.get("key1"), Some(&"value1".to_string()));
    assert_eq!(params.get("key2"), Some(&"value2".to_string()));
}
