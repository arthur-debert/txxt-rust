//! Verbatim Scanner Test Runner
//!
//! Runs tests from .txxt files with embedded test directives:
//! - #EXPECTED: [[start,end,"type"], ...] for successful parsing
//! - #ERROR: "message" for expected errors
//!
//! Each test file contains:
//! 1. First line: test directive (#EXPECTED or #ERROR)  
//! 2. Remaining lines: actual TXXT content to test

use std::fs;
use std::path::Path;
use txxt::syntax::verbatim_scanning::{VerbatimScanner, VerbatimType};

/// Test expectation parsed from test file directive
#[derive(Debug, Clone, PartialEq)]
enum TestExpectation {
    /// Expected verbatim blocks: [(content_start, content_end, type), ...]
    Blocks(Vec<(usize, usize, VerbatimType)>),
    /// Expected error message
    Error(String),
}

/// Parse test directive from first line of test file
fn parse_test_directive(line: &str) -> Result<TestExpectation, String> {
    if let Some(expected_part) = line.strip_prefix("Expected: ") {
        // Parse [[start,end,"type"], ...]
        if expected_part.trim() == "[]" {
            return Ok(TestExpectation::Blocks(Vec::new()));
        }

        // Simple parser for the expected format
        let expected_part = expected_part.trim();
        if !expected_part.starts_with('[') || !expected_part.ends_with(']') {
            return Err(format!("Invalid EXPECTED format: {}", expected_part));
        }

        let inner = &expected_part[1..expected_part.len() - 1]; // Remove outer []
        let mut blocks = Vec::new();

        if !inner.is_empty() {
            // Split by "], [" to get individual blocks
            let block_strs: Vec<&str> = if inner.contains("], [") {
                inner.split("], [").collect()
            } else {
                vec![inner]
            };

            for (i, block_str) in block_strs.iter().enumerate() {
                let mut block_str = *block_str;

                // Clean up brackets
                if i == 0 && block_str.starts_with('[') {
                    block_str = &block_str[1..];
                }
                if i == block_strs.len() - 1 && block_str.ends_with(']') {
                    block_str = &block_str[..block_str.len() - 1];
                }

                // Parse [start,end,"type"]
                let parts: Vec<&str> = block_str.split(',').map(|s| s.trim()).collect();
                if parts.len() != 3 {
                    return Err(format!("Invalid block format: {}", block_str));
                }

                let start: usize = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid start: {}", parts[0]))?;
                let end: usize = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid end: {}", parts[1]))?;
                let type_str = parts[2].trim_matches('"');
                let verbatim_type = match type_str {
                    "normal" => VerbatimType::Normal,
                    "stretched" => VerbatimType::Stretched,
                    "empty" => VerbatimType::Empty,
                    _ => return Err(format!("Invalid type: {}", type_str)),
                };

                blocks.push((start, end, verbatim_type));
            }
        }

        Ok(TestExpectation::Blocks(blocks))
    } else if let Some(error_part) = line.strip_prefix("Error: ") {
        let error_msg = error_part.trim_matches('"');
        Ok(TestExpectation::Error(error_msg.to_string()))
    } else {
        Err(format!("Unknown test directive: {}", line))
    }
}

/// Run a single test file
fn run_test_file(file_path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Err("Empty test file".to_string());
    }

    // Parse test directive from first line
    let expectation = parse_test_directive(lines[0])?;

    // Use entire file content (scanner will ignore non-TXXT content like Expected: line)
    let txxt_content = content;

    // Run verbatim scanner
    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(&txxt_content);

    // Check results against expectation
    match expectation {
        TestExpectation::Blocks(expected_blocks) => {
            if blocks.len() != expected_blocks.len() {
                return Err(format!(
                    "Block count mismatch: expected {}, got {}",
                    expected_blocks.len(),
                    blocks.len()
                ));
            }

            for (i, block) in blocks.iter().enumerate() {
                let (exp_start, exp_end, exp_type) = &expected_blocks[i];

                // Check block boundaries (start = title line, end = terminator line)
                if block.block_start != *exp_start {
                    return Err(format!(
                        "Block {} start mismatch: expected {}, got {}",
                        i, exp_start, block.block_start
                    ));
                }

                if block.block_end != *exp_end {
                    return Err(format!(
                        "Block {} end mismatch: expected {}, got {}",
                        i, exp_end, block.block_end
                    ));
                }

                if block.block_type != *exp_type {
                    return Err(format!(
                        "Block {} type mismatch: expected {:?}, got {:?}",
                        i, exp_type, block.block_type
                    ));
                }

                // For empty blocks, content_start and content_end should be None
                if *exp_type == VerbatimType::Empty {
                    if block.content_start.is_some() {
                        return Err(format!(
                            "Block {} should be empty but has content_start: {:?}",
                            i, block.content_start
                        ));
                    }
                    if block.content_end.is_some() {
                        return Err(format!(
                            "Block {} should be empty but has content_end: {:?}",
                            i, block.content_end
                        ));
                    }
                }
            }

            Ok(())
        }

        TestExpectation::Error(expected_error) => {
            // For error tests, we expect the scanner to detect some issue
            // The scanner should call finalize_scan and get an error
            // We'll capture stderr to check for error messages
            // We check if we're in an error state by examining blocks and expected error type

            // For unterminated blocks, we expect 0 blocks found
            if expected_error.contains("Unterminated") {
                if !blocks.is_empty() {
                    return Err(format!(
                        "Expected unterminated error but found {} blocks",
                        blocks.len()
                    ));
                }
                // Test passes - scanner correctly didn't find blocks due to unterminated state
                Ok(())
            } else {
                Err(format!(
                    "Error test type not implemented: {}",
                    expected_error
                ))
            }
        }
    }
}

/// Run all test files in a directory
fn run_test_directory(dir_path: &Path, test_type: &str) -> (usize, usize) {
    let mut passed = 0;
    let mut total = 0;

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "txxt" {
                    total += 1;

                    match run_test_file(&entry.path()) {
                        Ok(()) => {
                            println!("✓ {} - {}", test_type, entry.file_name().to_string_lossy());
                            passed += 1;
                        }
                        Err(e) => {
                            println!(
                                "✗ {} - {}: {}",
                                test_type,
                                entry.file_name().to_string_lossy(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    (passed, total)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_directive_empty() {
        let result = parse_test_directive("Expected: []");
        assert!(matches!(result, Ok(TestExpectation::Blocks(blocks)) if blocks.is_empty()));
    }

    #[test]
    fn test_parse_directive_single_block() {
        let result = parse_test_directive("Expected: [[2,3,\"normal\"]]");
        match result {
            Ok(TestExpectation::Blocks(blocks)) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0], (2, 3, VerbatimType::Normal));
            }
            _ => panic!("Expected single block"),
        }
    }

    #[test]
    fn test_parse_directive_multiple_blocks() {
        let result = parse_test_directive("Expected: [[2,3,\"normal\"], [6,8,\"stretched\"]]");
        match result {
            Ok(TestExpectation::Blocks(blocks)) => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(blocks[0], (2, 3, VerbatimType::Normal));
                assert_eq!(blocks[1], (6, 8, VerbatimType::Stretched));
            }
            _ => panic!("Expected multiple blocks"),
        }
    }

    #[test]
    pub fn test_verbatim_scanner_integration() {
        // Run tests on our test files
        let test_base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/verbatim_scanner");

        println!("\n=== Running Verbatim Scanner Tests ===");

        let (correct_passed, correct_total) =
            run_test_directory(&test_base.join("correct"), "CORRECT");
        let (negative_passed, negative_total) =
            run_test_directory(&test_base.join("negative"), "NEGATIVE");
        let (error_passed, error_total) = run_test_directory(&test_base.join("errors"), "ERROR");

        let total_passed = correct_passed + negative_passed + error_passed;
        let total_tests = correct_total + negative_total + error_total;

        println!("\n=== Results ===");
        println!("Correct tests: {}/{}", correct_passed, correct_total);
        println!("Negative tests: {}/{}", negative_passed, negative_total);
        println!("Error tests: {}/{}", error_passed, error_total);
        println!("Total: {}/{}", total_passed, total_tests);

        if total_passed != total_tests {
            panic!("Some verbatim scanner tests failed!");
        }
    }
}
