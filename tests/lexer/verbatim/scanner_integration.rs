//! Verbatim Scanner Integration Tests
//!
//! This module provides verbatim scanner integration tests.
//! The main integration test is available in tests/verbatim_scanner_tests.rs

use txxt::lexer::verbatim_scanning::VerbatimScanner;

#[test]
fn test_basic_verbatim_scanner_integration() {
    let scanner = VerbatimScanner::new();
    let text = r#"Simple verbatim:
    print("Hello World")
:: python"#;

    let blocks = scanner.scan(text);
    assert_eq!(blocks.len(), 1, "Should find exactly one verbatim block");

    let block = &blocks[0];
    assert!(block.content_start.is_some(), "Block should have content");
    assert!(block.content_end.is_some(), "Block should have content end");
}
