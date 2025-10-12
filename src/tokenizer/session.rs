//! Session element tokenization
//!
//! Implements tokenization for session elements as defined in
//! docs/specs/elements/session.txxt
//!
//! Sessions provide hierarchical document organization

use crate::ast::tokens::{Position, SourceSpan, Token};

/// Detect if a line represents a session title
pub fn is_session_title(line: &str) -> bool {
    let trimmed = line.trim();

    // Session titles are typically at specific indentation levels
    // and don't start with special markers
    !trimmed.is_empty()
        && !trimmed.starts_with("::")
        && !trimmed.starts_with("- ")
        && !trimmed.ends_with(" ::")
}

/// Extract session level from indentation
pub fn get_session_level(line: &str) -> usize {
    // Count leading spaces to determine session level
    line.chars().take_while(|&c| c == ' ').count() / 2 // 2 spaces per level
}

/// Create a session title token
pub fn create_session_title_token(content: &str, level: usize, start_pos: Position) -> Token {
    Token::Text {
        content: format!("Session Level {}: {}", level, content.trim()),
        span: SourceSpan {
            start: start_pos,
            end: Position {
                row: start_pos.row,
                column: start_pos.column + content.len(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_title_detection() {
        assert!(is_session_title("Introduction"));
        assert!(is_session_title("  Subsection"));
        assert!(!is_session_title(""));
        assert!(!is_session_title(":: annotation ::"));
        assert!(!is_session_title("- list item"));
        assert!(!is_session_title("term ::"));
    }

    #[test]
    fn test_session_level_calculation() {
        assert_eq!(get_session_level("Title"), 0);
        assert_eq!(get_session_level("  Subsection"), 1);
        assert_eq!(get_session_level("    Deep Section"), 2);
        assert_eq!(get_session_level("      Very Deep"), 3);
    }

    #[test]
    fn test_session_title_token() {
        let token = create_session_title_token("Introduction", 0, Position { row: 0, column: 0 });

        match token {
            Token::Text { content, .. } => {
                assert_eq!(content, "Session Level 0: Introduction");
            }
            _ => panic!("Expected Text token"),
        }
    }
}
