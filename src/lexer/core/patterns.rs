//! Shared regex patterns and content extraction utilities for TXXT tokenization
//!
//! This module provides utilities for extracting raw content from source spans
//! and common pattern matching functionality used across different tokenizers.

use crate::ast::scanner_tokens::SourceSpan;

/// Pattern for valid text content (used in property tests)
/// Note: Text cannot start with formatting delimiters like _ * ` # -
pub const TEXT_PATTERN: &str = r"[a-zA-Z0-9][a-zA-Z0-9_]*";

/// Pattern for valid identifiers (used in property tests for annotations)
/// Note: Identifiers starting with _ must be followed by alphanumeric chars
pub const IDENTIFIER_PATTERN: &str = r"[a-zA-Z][a-zA-Z0-9_]*|_[a-zA-Z0-9][a-zA-Z0-9_]*";

/// Extract raw content between two source spans (for annotation content)
pub fn extract_raw_content_between_spans(
    start_span: &SourceSpan,
    end_span: &SourceSpan,
    input: &[char],
) -> String {
    // For annotation: extract content between :: markers
    // Start after the first :: and end before the second ::

    let start_pos = start_span.end; // After the opening ::
    let end_pos = end_span.start; // Before the closing ::

    // Extract from the original input
    if start_pos.row == end_pos.row {
        // Same line - extract substring
        let input_string: String = input.iter().collect();
        let lines: Vec<&str> = input_string.lines().collect();

        if start_pos.row < lines.len() {
            let line = lines[start_pos.row];
            let start_col = start_pos.column.min(line.len());
            let end_col = end_pos.column.min(line.len());

            if start_col < end_col {
                return line[start_col..end_col].trim().to_string();
            }
        }
    }

    // Fallback for multi-line or edge cases
    String::new()
}

/// Extract raw content before a span (for definition terms)
pub fn extract_raw_content_before_span(
    start_span: &SourceSpan,
    end_span: &SourceSpan,
    input: &[char],
) -> String {
    // Extract from start of term to before the :: marker

    let start_pos = start_span.start; // Start of first term token
    let end_pos = end_span.start; // Before the :: marker

    // Extract from the original input
    if start_pos.row == end_pos.row {
        // Same line - extract substring
        let input_string: String = input.iter().collect();
        let lines: Vec<&str> = input_string.lines().collect();

        if start_pos.row < lines.len() {
            let line = lines[start_pos.row];
            let start_col = start_pos.column.min(line.len());
            let end_col = end_pos.column.min(line.len());

            if start_col < end_col {
                return line[start_col..end_col].trim().to_string();
            }
        }
    }

    // Fallback for multi-line or edge cases
    String::new()
}

/// Get the current line from input at a specific position
pub fn get_current_line(input: &[char], position: usize, _row: usize, column: usize) -> String {
    let mut line_start = position - column;
    let mut line_end = position;

    // Find start of current line
    while line_start > 0 {
        if let Some(&ch) = input.get(line_start - 1) {
            if ch == '\n' || ch == '\r' {
                break;
            }
            line_start -= 1;
        } else {
            break;
        }
    }

    // Find end of current line
    while line_end < input.len() {
        if let Some(&ch) = input.get(line_end) {
            if ch == '\n' || ch == '\r' {
                break;
            }
            line_end += 1;
        } else {
            break;
        }
    }

    input[line_start..line_end].iter().collect()
}

/// Get a specific line from input by row number
pub fn get_line_by_row(input: &[char], row: usize) -> String {
    let input_string: String = input.iter().collect();
    let lines: Vec<&str> = input_string.lines().collect();

    if row < lines.len() {
        lines[row].to_string()
    } else {
        String::new()
    }
}
