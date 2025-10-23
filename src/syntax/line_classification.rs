//! Line Classification Functions
//!
//! Pure functions for classifying lines and detecting element types.
//! These extracted functions improve testability and maintainability
//! per the progressive-quality-improvements plan.
//!
//! See: docs/proposals/progressive-quality-improvements.txxt

use crate::cst::ScannerToken;

/// Check if tokens represent a definition marker pattern
///
/// After grammar simplification (issue #139), definitions are marked by:
/// - Single colon (`:`) at end of line (not `::`)
/// - Followed immediately by indented content (no blank line)
///
/// This function only checks for the colon pattern, not the indent.
/// The indent check is done at the high-level token stage.
///
/// # Arguments
/// * `tokens` - Sequence of scanner tokens for a single line
///
/// # Returns
/// * `true` if line ends with single colon (definition marker pattern)
/// * `false` otherwise
///
/// # Examples
/// ```text
/// Term:  → true (definition)
/// Term:: → false (old syntax, now annotation)
/// Term   → false (no marker)
/// ```
pub fn is_definition_marker(tokens: &[ScannerToken]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // Look for pattern: ... Colon (Whitespace|Newline|EOF)
    // We need to find a Colon that's NOT followed by another Colon (which would be ::)

    let mut found_trailing_colon = false;
    let mut i = tokens.len();

    // Scan backwards from end to find last non-whitespace/newline token
    while i > 0 {
        i -= 1;
        match &tokens[i] {
            ScannerToken::Whitespace { .. } | ScannerToken::Newline { .. } => {
                // Skip trailing whitespace/newlines
                continue;
            }
            ScannerToken::Colon { .. } => {
                // Found a colon - check it's not part of ::
                if i > 0 {
                    if let ScannerToken::Colon { .. } = &tokens[i - 1] {
                        // This is the second colon in ::, not a definition marker
                        return false;
                    }
                }
                // Check next token isn't also a colon (would make ::)
                if i + 1 < tokens.len() {
                    match &tokens[i + 1] {
                        ScannerToken::Colon { .. } => {
                            // Next is colon, so this is first : in ::
                            return false;
                        }
                        ScannerToken::Whitespace { .. } | ScannerToken::Newline { .. } => {
                            // Colon followed by whitespace/newline - valid definition marker
                            found_trailing_colon = true;
                            break;
                        }
                        _ => {
                            // Colon followed by other content - not a definition marker
                            return false;
                        }
                    }
                } else {
                    // Colon at very end - valid definition marker
                    found_trailing_colon = true;
                    break;
                }
            }
            _ => {
                // Found non-whitespace, non-colon token - no trailing colon
                return false;
            }
        }
    }

    found_trailing_colon
}

/// Check if tokens represent a blank line
///
/// A blank line is:
/// - Empty, OR
/// - Contains only whitespace
///
/// # Arguments
/// * `tokens` - Sequence of scanner tokens
///
/// # Returns
/// * `true` if tokens represent a blank line
pub fn is_blank_line(tokens: &[ScannerToken]) -> bool {
    if tokens.is_empty() {
        return true;
    }

    tokens.iter().all(|token| {
        matches!(
            token,
            ScannerToken::Whitespace { .. }
                | ScannerToken::Newline { .. }
                | ScannerToken::BlankLine { .. }
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    fn make_colon(col: usize) -> ScannerToken {
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + 1,
                },
            },
        }
    }

    fn make_text(content: &str, col: usize) -> ScannerToken {
        ScannerToken::Text {
            content: content.to_string(),
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + content.len(),
                },
            },
        }
    }

    fn make_whitespace(col: usize) -> ScannerToken {
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + 1,
                },
            },
        }
    }

    fn make_newline(col: usize) -> ScannerToken {
        ScannerToken::Newline {
            span: SourceSpan {
                start: Position {
                    row: 0,
                    column: col,
                },
                end: Position {
                    row: 0,
                    column: col + 1,
                },
            },
        }
    }

    #[test]
    fn test_is_definition_marker_simple() {
        // "Term:"
        let tokens = vec![make_text("Term", 0), make_colon(4)];
        assert!(is_definition_marker(&tokens));
    }

    #[test]
    fn test_is_definition_marker_with_trailing_whitespace() {
        // "Term: "
        let tokens = vec![make_text("Term", 0), make_colon(4), make_whitespace(5)];
        assert!(is_definition_marker(&tokens));
    }

    #[test]
    fn test_is_definition_marker_with_newline() {
        // "Term:\n"
        let tokens = vec![make_text("Term", 0), make_colon(4), make_newline(5)];
        assert!(is_definition_marker(&tokens));
    }

    #[test]
    fn test_not_definition_marker_double_colon() {
        // "Term::" (old annotation syntax)
        let tokens = vec![make_text("Term", 0), make_colon(4), make_colon(5)];
        assert!(!is_definition_marker(&tokens));
    }

    #[test]
    fn test_not_definition_marker_no_colon() {
        // "Term"
        let tokens = vec![make_text("Term", 0)];
        assert!(!is_definition_marker(&tokens));
    }

    #[test]
    fn test_not_definition_marker_mid_line_colon() {
        // "Term: text" (colon not at end)
        let tokens = vec![
            make_text("Term", 0),
            make_colon(4),
            make_whitespace(5),
            make_text("text", 6),
        ];
        assert!(!is_definition_marker(&tokens));
    }

    #[test]
    fn test_is_blank_line_empty() {
        let tokens: Vec<ScannerToken> = vec![];
        assert!(is_blank_line(&tokens));
    }

    #[test]
    fn test_is_blank_line_whitespace_only() {
        let tokens = vec![make_whitespace(0), make_whitespace(1)];
        assert!(is_blank_line(&tokens));
    }

    #[test]
    fn test_not_blank_line_has_text() {
        let tokens = vec![make_whitespace(0), make_text("text", 1)];
        assert!(!is_blank_line(&tokens));
    }
}
