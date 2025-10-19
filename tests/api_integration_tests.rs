//! Integration tests for the TXXT API
//!
//! These tests replace the shell-based tests with proper unit tests that call the API directly.
//! Tests the public API without I/O operations or subprocess calls.

use txxt::api::{process, ProcessArgs};

#[test]
fn test_scanner_tokens_format() {
    let args = ProcessArgs {
        content: "Some content".to_string(),
        source_path: "test.txxt".to_string(),
        stage: "scanner-tokens".to_string(),
        format: "json".to_string(),
    };

    let result = process(args).unwrap();

    // Verify JSON structure
    assert!(result.contains("\"tokens\""));
    assert!(result.contains("\"source\": \"test.txxt\""));

    // Parse as JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["tokens"].is_array());
}

#[test]
fn test_semantic_tokens_format() {
    let args = ProcessArgs {
        content: ":: note :: Some content".to_string(),
        source_path: "test.txxt".to_string(),
        stage: "semantic-tokens".to_string(),
        format: "json".to_string(),
    };

    let result = process(args).unwrap();

    // Verify JSON structure
    assert!(result.contains("\"semantic_tokens\""));
    assert!(result.contains("\"source\": \"test.txxt\""));

    // Parse as JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["semantic_tokens"].is_array());
}

#[test]
fn test_ast_full_json_format() {
    let args = ProcessArgs {
        content: "Hello world".to_string(),
        source_path: "test.txxt".to_string(),
        stage: "ast-full".to_string(),
        format: "json".to_string(),
    };

    let result = process(args).unwrap();

    // Verify JSON structure
    assert!(result.contains("\"document\""));
    assert!(result.contains("\"source\": \"test.txxt\""));

    // Parse as JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["document"].is_object());
}

#[test]
#[ignore = "Depends on Phase 2.b AST Construction which is not yet implemented"]
fn test_ast_full_treeviz_format() {
    let args = ProcessArgs {
        content: "Hello world".to_string(),
        source_path: "test.txxt".to_string(),
        stage: "ast-full".to_string(),
        format: "treeviz".to_string(),
    };

    let result = process(args).unwrap();

    // Verify treeviz structure
    assert!(result.contains("⧉ Document: test.txxt"));
}

#[test]
#[ignore = "Depends on Phase 2.b AST Construction which is not yet implemented"]
fn test_phase_2_formats_implemented() {
    let test_cases = vec![
        ("ast-block", "json"),
        ("ast-block", "treeviz"),
        ("ast-inlines", "json"),
        ("ast-inlines", "treeviz"),
    ];

    for (stage, format) in test_cases {
        let args = ProcessArgs {
            content: "test".to_string(),
            source_path: "test.txxt".to_string(),
            stage: stage.to_string(),
            format: format.to_string(),
        };

        let result = process(args);
        assert!(result.is_ok(), "Combination {:?} should be implemented", (stage, format));
    }
}

#[test]
fn test_empty_content() {
    let args = ProcessArgs {
        content: "".to_string(),
        source_path: "empty.txxt".to_string(),
        stage: "scanner-tokens".to_string(),
        format: "json".to_string(),
    };

    let result = process(args).unwrap();

    // Should handle empty content gracefully
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["tokens"].is_array());
}

#[test]
fn test_multiline_content() {
    let content = "Line 1\nLine 2\nLine 3\n";

    let args = ProcessArgs {
        content: content.to_string(),
        source_path: "multiline.txxt".to_string(),
        stage: "scanner-tokens".to_string(),
        format: "json".to_string(),
    };

    let result = process(args).unwrap();

    // Parse and verify tokens include newlines
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    let tokens = json["tokens"].as_array().unwrap();

    // Should have multiple tokens including text and newlines
    assert!(tokens.len() > 1);
}

#[test]
fn test_source_path_preservation() {
    let test_paths = vec![
        "simple.txxt",
        "/absolute/path/file.txxt",
        "../relative/path.txxt",
        "special-chars_file.txxt",
    ];

    for path in test_paths {
        let args = ProcessArgs {
            content: "test content".to_string(),
            source_path: path.to_string(),
            stage: "scanner-tokens".to_string(),
            format: "json".to_string(),
        };

        let result = process(args).unwrap();

        // Verify the source path is preserved in output
        assert!(result.contains(&format!("\"source\": \"{}\"", path)));
    }
}

#[test]
fn test_error_handling() {
    // Test with potentially problematic content
    let problematic_contents = vec![
        "\u{0000}",       // null byte
        "🚀🎉🔥",         // unicode emojis
        "line1\r\nline2", // CRLF
        "\t\t\tdeep tabs",
    ];

    for content in problematic_contents {
        let args = ProcessArgs {
            content: content.to_string(),
            source_path: "test.txxt".to_string(),
            stage: "scanner-tokens".to_string(),
            format: "json".to_string(),
        };

        // Should not panic or error on unusual content
        let result = process(args);
        assert!(result.is_ok(), "Failed to process content: {:?}", content);
    }
}
