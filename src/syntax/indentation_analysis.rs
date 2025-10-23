//! Indentation Analysis Functions
//!
//! Pure functions for analyzing and extracting indentation information.
//! These extracted functions improve testability and maintainability
//! per the progressive-quality-improvements plan (Phase 2, section 3.2).
//!
//! See: docs/proposals/progressive-quality-improvements.txxt

use crate::cst::ScannerToken;

/// Extract leading whitespace from a sequence of scanner tokens
///
/// This function identifies "wall" whitespace - the structural indentation padding
/// that appears at the start of a line after Indent/Dedent/BlankLine/Newline tokens.
///
/// # Arguments
/// * `tokens` - Scanner token sequence to analyze
/// * `position` - Current position in the token sequence
///
/// # Returns
/// * `Option<String>` - The whitespace content if found, None otherwise
///
/// # Detection Rules
/// Leading whitespace is detected when:
/// - Current token is a Whitespace token
/// - Position is 0 (start of document), OR
/// - Previous token is Indent, BlankLine, Dedent, or Newline
///
/// # Examples
/// ```text
/// Tokens: [Indent, Whitespace("    "), Text("hello")]
/// Position: 1
/// Output: Some("    ")
/// ```
pub fn extract_leading_whitespace_from_tokens(
    tokens: &[ScannerToken],
    position: usize,
) -> Option<String> {
    if position >= tokens.len() {
        return None;
    }

    // Check if current token is whitespace
    if let ScannerToken::Whitespace { content, .. } = &tokens[position] {
        // Check if this whitespace is at line start (after structural tokens)
        let is_at_line_start = position == 0
            || matches!(
                tokens.get(position - 1),
                Some(ScannerToken::Indent { .. })
                    | Some(ScannerToken::BlankLine { .. })
                    | Some(ScannerToken::Dedent { .. })
                    | Some(ScannerToken::Newline { .. })
            );

        if is_at_line_start {
            return Some(content.clone());
        }
    }

    None
}

/// Calculate indentation level from whitespace string
///
/// Converts physical whitespace characters to a numeric indentation level.
/// Tabs are converted to spaces using the standard tab width (4 spaces).
///
/// # Arguments
/// * `whitespace` - The whitespace string to analyze
///
/// # Returns
/// * `usize` - Number of spaces (tabs counted as 4 spaces each)
///
/// # Examples
/// ```text
/// Input: "    "     → Output: 4
/// Input: "\t"       → Output: 4
/// Input: "  \t  "   → Output: 8 (2 + 4 + 2)
/// Input: ""         → Output: 0
/// ```
pub fn calculate_indentation_level(whitespace: &str) -> usize {
    let mut level = 0;
    for ch in whitespace.chars() {
        match ch {
            ' ' => level += 1,
            '\t' => level += 4, // Standard tab width
            _ => break,         // Stop at first non-whitespace
        }
    }
    level
}

/// Check if all lines in a collection maintain consistent indentation
///
/// Validates that all lines either:
/// - Match the expected indentation level exactly, OR
/// - Are blank lines (which don't affect indentation consistency)
///
/// # Arguments
/// * `lines` - Collection of (indentation_level, is_blank) tuples
/// * `expected_level` - The indentation level that should be maintained
///
/// # Returns
/// * `true` if all non-blank lines match expected_level
/// * `false` if any non-blank line has different indentation
///
/// # Examples
/// ```text
/// Lines: [(4, false), (4, false), (0, true), (4, false)]
/// Expected: 4
/// Output: true (blank line at position 2 doesn't break consistency)
///
/// Lines: [(4, false), (8, false)]
/// Expected: 4
/// Output: false (line 2 has wrong indentation)
/// ```
pub fn is_consistently_indented(lines: &[(usize, bool)], expected_level: usize) -> bool {
    lines
        .iter()
        .all(|(level, is_blank)| *is_blank || *level == expected_level)
}

/// Helper function to extract indentation information from a line string
///
/// Combines indentation calculation with blank line detection.
///
/// # Arguments
/// * `line` - The line string to analyze
///
/// # Returns
/// * `(usize, bool)` - Tuple of (indentation_level, is_blank)
pub fn analyze_line_indentation(line: &str) -> (usize, bool) {
    let is_blank = line.trim().is_empty();
    let level = if is_blank {
        0 // Blank lines considered as level 0
    } else {
        calculate_indentation_level(line)
    };
    (level, is_blank)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    fn make_whitespace(content: &str, row: usize) -> ScannerToken {
        ScannerToken::Whitespace {
            content: content.to_string(),
            span: SourceSpan {
                start: Position { row, column: 0 },
                end: Position {
                    row,
                    column: content.len(),
                },
            },
        }
    }

    fn make_indent(row: usize) -> ScannerToken {
        ScannerToken::Indent {
            span: SourceSpan {
                start: Position { row, column: 0 },
                end: Position { row, column: 4 },
            },
        }
    }

    fn make_text(content: &str, row: usize) -> ScannerToken {
        ScannerToken::Text {
            content: content.to_string(),
            span: SourceSpan {
                start: Position { row, column: 0 },
                end: Position {
                    row,
                    column: content.len(),
                },
            },
        }
    }

    fn make_newline(row: usize) -> ScannerToken {
        ScannerToken::Newline {
            span: SourceSpan {
                start: Position { row, column: 0 },
                end: Position { row, column: 1 },
            },
        }
    }

    #[test]
    fn test_extract_leading_whitespace_at_start() {
        let tokens = vec![make_whitespace("    ", 0), make_text("hello", 0)];
        let ws = extract_leading_whitespace_from_tokens(&tokens, 0);
        assert_eq!(ws, Some("    ".to_string()));
    }

    #[test]
    fn test_extract_leading_whitespace_after_indent() {
        let tokens = vec![
            make_indent(0),
            make_whitespace("    ", 0),
            make_text("hello", 0),
        ];
        let ws = extract_leading_whitespace_from_tokens(&tokens, 1);
        assert_eq!(ws, Some("    ".to_string()));
    }

    #[test]
    fn test_extract_leading_whitespace_after_newline() {
        let tokens = vec![
            make_text("line1", 0),
            make_newline(0),
            make_whitespace("  ", 1),
            make_text("line2", 1),
        ];
        let ws = extract_leading_whitespace_from_tokens(&tokens, 2);
        assert_eq!(ws, Some("  ".to_string()));
    }

    #[test]
    fn test_extract_leading_whitespace_mid_line() {
        let tokens = vec![
            make_text("hello", 0),
            make_whitespace("  ", 0),
            make_text("world", 0),
        ];
        let ws = extract_leading_whitespace_from_tokens(&tokens, 1);
        assert_eq!(ws, None); // Not at line start
    }

    #[test]
    fn test_extract_leading_whitespace_no_whitespace() {
        let tokens = vec![make_text("hello", 0)];
        let ws = extract_leading_whitespace_from_tokens(&tokens, 0);
        assert_eq!(ws, None);
    }

    #[test]
    fn test_calculate_indentation_level_spaces() {
        assert_eq!(calculate_indentation_level(""), 0);
        assert_eq!(calculate_indentation_level("    "), 4);
        assert_eq!(calculate_indentation_level("        "), 8);
        assert_eq!(calculate_indentation_level("  "), 2);
    }

    #[test]
    fn test_calculate_indentation_level_tabs() {
        assert_eq!(calculate_indentation_level("\t"), 4);
        assert_eq!(calculate_indentation_level("\t\t"), 8);
    }

    #[test]
    fn test_calculate_indentation_level_mixed() {
        assert_eq!(calculate_indentation_level("  \t"), 6); // 2 spaces + 1 tab (4)
        assert_eq!(calculate_indentation_level("\t  "), 6); // 1 tab (4) + 2 spaces
        assert_eq!(calculate_indentation_level(" \t "), 6); // 1 + 4 + 1
    }

    #[test]
    fn test_calculate_indentation_level_stops_at_content() {
        assert_eq!(calculate_indentation_level("  hello"), 2);
        assert_eq!(calculate_indentation_level("\thello"), 4);
    }

    #[test]
    fn test_is_consistently_indented_all_same() {
        let lines = vec![(4, false), (4, false), (4, false)];
        assert!(is_consistently_indented(&lines, 4));
    }

    #[test]
    fn test_is_consistently_indented_with_blanks() {
        let lines = vec![(4, false), (0, true), (4, false)];
        assert!(is_consistently_indented(&lines, 4));
    }

    #[test]
    fn test_is_consistently_indented_inconsistent() {
        let lines = vec![(4, false), (8, false), (4, false)];
        assert!(!is_consistently_indented(&lines, 4));
    }

    #[test]
    fn test_is_consistently_indented_empty() {
        let lines: Vec<(usize, bool)> = vec![];
        assert!(is_consistently_indented(&lines, 4));
    }

    #[test]
    fn test_is_consistently_indented_all_blanks() {
        let lines = vec![(0, true), (0, true), (0, true)];
        assert!(is_consistently_indented(&lines, 4)); // All blank = consistent
    }

    #[test]
    fn test_analyze_line_indentation_normal() {
        assert_eq!(analyze_line_indentation("hello"), (0, false));
        assert_eq!(analyze_line_indentation("    hello"), (4, false));
        assert_eq!(analyze_line_indentation("\tworld"), (4, false));
    }

    #[test]
    fn test_analyze_line_indentation_blank() {
        assert_eq!(analyze_line_indentation(""), (0, true));
        assert_eq!(analyze_line_indentation("   "), (0, true));
        assert_eq!(analyze_line_indentation("\t  \t"), (0, true));
    }
}
