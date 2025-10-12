//! Label system tokenization
//!
//! Implements tokenization for the label system as defined in
//! docs/specs/elements/labels.txxt
//!
//! Labels are used in annotations and verbatim blocks

use crate::ast::tokens::{Position, SourceSpan};

/// Validate a label identifier
pub fn is_valid_label(label: &str) -> bool {
    if label.is_empty() {
        return false;
    }

    // Label must start with letter or underscore
    let first_char = label.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    // Rest can be letters, digits, underscores, hyphens
    label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Parse a label from a string and return its span
pub fn parse_label(input: &str, start_pos: Position) -> Option<(String, SourceSpan)> {
    let trimmed = input.trim();

    if is_valid_label(trimmed) {
        Some((
            trimmed.to_string(),
            SourceSpan {
                start: start_pos,
                end: Position {
                    row: start_pos.row,
                    column: start_pos.column + trimmed.len(),
                },
            },
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_labels() {
        assert!(is_valid_label("title"));
        assert!(is_valid_label("_private"));
        assert!(is_valid_label("test-case"));
        assert!(is_valid_label("item123"));
        assert!(is_valid_label("a"));
    }

    #[test]
    fn test_invalid_labels() {
        assert!(!is_valid_label(""));
        assert!(!is_valid_label("123invalid"));
        assert!(!is_valid_label("-invalid"));
        assert!(!is_valid_label("invalid.label"));
        assert!(!is_valid_label("invalid label"));
    }
}
