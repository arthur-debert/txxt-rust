//! Integration tests for the TXXT API
//!
//! These tests replace the shell-based tests with proper unit tests that call the API directly.
//! Tests the public API without I/O operations or subprocess calls.

use txxt::api::{format_output_unified, process_unified, Format, Stage};

#[test]
fn test_scanner_tokens_format() {
    let output = process_unified(
        "Some content",
        Stage::ScannerTokens,
        Some("test.txxt".to_string()),
    )
    .unwrap();

    let result = format_output_unified(&output, Format::Json, Some("test.txxt")).unwrap();

    // Verify JSON structure
    assert!(result.contains("\"tokens\""));
    assert!(result.contains("\"source\": \"test.txxt\""));

    // Parse as JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["tokens"].is_array());
}

#[test]
fn test_semantic_tokens_format() {
    let output = process_unified(
        ":: note :: Some content",
        Stage::HighLevelTokens,
        Some("test.txxt".to_string()),
    )
    .unwrap();

    let result = format_output_unified(&output, Format::Json, Some("test.txxt")).unwrap();

    // Verify JSON structure
    assert!(result.contains("\"tokens\""));
    assert!(result.contains("\"source\": \"test.txxt\""));
    assert!(result.contains("\"stage\": \"high-level-tokens\""));

    // Parse as JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["tokens"].is_array());
}

#[test]
fn test_ast_full_json_format() {
    let output =
        process_unified("Hello world", Stage::AstFull, Some("test.txxt".to_string())).unwrap();

    let result = format_output_unified(&output, Format::Json, Some("test.txxt")).unwrap();

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
    let output =
        process_unified("Hello world", Stage::AstFull, Some("test.txxt".to_string())).unwrap();

    let result = format_output_unified(&output, Format::TreeViz, Some("test.txxt")).unwrap();

    // Verify treeviz structure
    assert!(result.contains("⧉ Document: test.txxt"));
}

#[test]
#[ignore = "Depends on Phase 2.b AST Construction which is not yet implemented"]
fn test_phase_2_formats_implemented() {
    let test_cases = vec![
        (Stage::AstBlock, Format::Json),
        (Stage::AstBlock, Format::TreeViz),
        (Stage::AstInlines, Format::Json),
        (Stage::AstInlines, Format::TreeViz),
    ];

    for (stage, format) in test_cases {
        let output = process_unified("test", stage, Some("test.txxt".to_string())).unwrap();

        let result = format_output_unified(&output, format, Some("test.txxt"));
        assert!(
            result.is_ok(),
            "Combination {:?} should be implemented",
            (stage, format)
        );
    }
}

#[test]
fn test_empty_content() {
    let output = process_unified("", Stage::ScannerTokens, Some("empty.txxt".to_string())).unwrap();

    let result = format_output_unified(&output, Format::Json, Some("empty.txxt")).unwrap();

    // Should handle empty content gracefully
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["tokens"].is_array());
}

#[test]
fn test_multiline_content() {
    let content = "Line 1\nLine 2\nLine 3\n";

    let output = process_unified(
        content,
        Stage::ScannerTokens,
        Some("multiline.txxt".to_string()),
    )
    .unwrap();

    let result = format_output_unified(&output, Format::Json, Some("multiline.txxt")).unwrap();

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
        let output =
            process_unified("test content", Stage::ScannerTokens, Some(path.to_string())).unwrap();

        let result = format_output_unified(&output, Format::Json, Some(path)).unwrap();

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
        // Should not panic or error on unusual content
        let result = process_unified(content, Stage::ScannerTokens, Some("test.txxt".to_string()));
        assert!(result.is_ok(), "Failed to process content: {:?}", content);
    }
}
