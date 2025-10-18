//! Integration tests for the TXXT API
//!
//! These tests replace the shell-based tests with proper unit tests that call the API directly.
//! Tests the public API without I/O operations or subprocess calls.

use txxt::api::{process, OutputFormat, ProcessArgs};

#[test]
fn test_verbatim_marks_format() {
    let args = ProcessArgs {
        content: "Some content\n    console.log('test');\n(javascript)".to_string(),
        source_path: "test.txxt".to_string(),
        format: OutputFormat::VerbatimMarks,
    };

    let result = process(args).unwrap();

    // Verify JSON structure
    assert!(result.contains("\"verbatim_blocks\""));
    assert!(result.contains("\"source\": \"test.txxt\""));

    // Parse as JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["verbatim_blocks"].is_array());
}

#[test]
fn test_token_stream_format() {
    let args = ProcessArgs {
        content: "Hello world".to_string(),
        source_path: "test.txxt".to_string(),
        format: OutputFormat::TokenStream,
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
fn test_token_tree_format() {
    let args = ProcessArgs {
        content: "Hello world".to_string(),
        source_path: "test.txxt".to_string(),
        format: OutputFormat::ScannerTokenTree,
    };

    let result = process(args).unwrap();

    // Verify JSON structure
    assert!(result.contains("\"token_tree\""));
    assert!(result.contains("\"source\": \"test.txxt\""));

    // Parse as JSON to verify it's valid
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["token_tree"].is_object());
}

#[test]
fn test_ast_full_json_format() {
    let args = ProcessArgs {
        content: "Hello world".to_string(),
        source_path: "test.txxt".to_string(),
        format: OutputFormat::AstFullJson,
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
        format: OutputFormat::AstFullTreeviz,
    };

    let result = process(args).unwrap();

    // Verify treeviz structure
    assert!(result.contains("⧉ Document: test.txxt"));
    assert!(result.contains("Ψ")); // SessionContainer icon
    assert!(result.contains("¶")); // Paragraph icon
}

#[test]
#[ignore = "Depends on Phase 2.b AST Construction which is not yet implemented"]
fn test_phase_2_formats_implemented() {
    let test_cases = vec![
        OutputFormat::AstNoInlineTreeviz,
        OutputFormat::AstNoInlineJson,
        OutputFormat::AstTreeviz,
        OutputFormat::AstJson,
    ];

    for format in test_cases {
        let args = ProcessArgs {
            content: "test".to_string(),
            source_path: "test.txxt".to_string(),
            format: format.clone(),
        };

        let result = process(args);
        assert!(result.is_ok(), "Format {:?} should be implemented", format);
    }
}

#[test]
fn test_format_parsing() {
    // Test valid formats
    assert_eq!(
        "verbatim-marks".parse::<OutputFormat>().unwrap(),
        OutputFormat::VerbatimMarks
    );
    assert_eq!(
        "token-stream".parse::<OutputFormat>().unwrap(),
        OutputFormat::TokenStream
    );
    assert_eq!(
        "token-tree".parse::<OutputFormat>().unwrap(),
        OutputFormat::ScannerTokenTree
    );
    assert_eq!(
        "ast-full-json".parse::<OutputFormat>().unwrap(),
        OutputFormat::AstFullJson
    );
    assert_eq!(
        "ast-full-treeviz".parse::<OutputFormat>().unwrap(),
        OutputFormat::AstFullTreeviz
    );

    // Test invalid format
    let result = "invalid-format".parse::<OutputFormat>();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown format"));
}

#[test]
fn test_empty_content() {
    let args = ProcessArgs {
        content: "".to_string(),
        source_path: "empty.txxt".to_string(),
        format: OutputFormat::TokenStream,
    };

    let result = process(args).unwrap();

    // Should handle empty content gracefully
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["tokens"].is_array());
}

#[test]
fn test_complex_verbatim_content() {
    let content = r#"
Some text before

    ```python
    def hello():
        print("Hello, world!")
    ```
(python)

More text after
"#;

    let args = ProcessArgs {
        content: content.to_string(),
        source_path: "complex.txxt".to_string(),
        format: OutputFormat::VerbatimMarks,
    };

    let result = process(args).unwrap();

    // Parse and verify structure
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["verbatim_blocks"].is_array());

    // Should detect the verbatim block
    let blocks = &json["verbatim_blocks"];
    if !blocks.as_array().unwrap().is_empty() {
        let first_block = &blocks[0];
        assert!(first_block["block_type"].is_string());
    }
}

#[test]
fn test_multiline_content() {
    let content = "Line 1\nLine 2\nLine 3\n";

    let args = ProcessArgs {
        content: content.to_string(),
        source_path: "multiline.txxt".to_string(),
        format: OutputFormat::TokenStream,
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
            format: OutputFormat::TokenStream,
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
            format: OutputFormat::TokenStream,
        };

        // Should not panic or error on unusual content
        let result = process(args);
        assert!(result.is_ok(), "Failed to process content: {:?}", content);
    }
}
