//! Paragraph element tokenization
//!
//! Implements tokenization for paragraph elements as defined in
//! docs/specs/elements/paragraph.txxt
//!
//! Paragraphs are the default block elements for text content

use crate::ast::tokens::{Position, SourceSpan, Token};

/// Identify if a line represents paragraph content
pub fn is_paragraph_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Empty lines are not paragraph content
    if trimmed.is_empty() {
        return false;
    }

    // Lines starting with special markers are not paragraphs
    if trimmed.starts_with("::") || // annotations
       trimmed.starts_with("- ") || // lists
       trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) && trimmed.contains(". ") || // numbered lists
       trimmed.ends_with(" ::")
    // definitions
    {
        return false;
    }

    // Otherwise, it's paragraph content
    true
}

/// Create a text token for paragraph content
pub fn create_paragraph_text_token(content: &str, start_pos: Position) -> Token {
    Token::Text {
        content: content.to_string(),
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
    fn test_paragraph_identification() {
        assert!(is_paragraph_line("This is regular text"));
        assert!(is_paragraph_line("  Indented text"));
        assert!(!is_paragraph_line(""));
        assert!(!is_paragraph_line("   "));
        assert!(!is_paragraph_line(":: annotation ::"));
        assert!(!is_paragraph_line("- list item"));
        assert!(!is_paragraph_line("1. numbered item"));
        assert!(!is_paragraph_line("term ::"));
    }

    #[test]
    fn test_paragraph_text_token() {
        let token = create_paragraph_text_token("hello world", Position { row: 0, column: 0 });

        match token {
            Token::Text { content, span } => {
                assert_eq!(content, "hello world");
                assert_eq!(span.start.row, 0);
                assert_eq!(span.start.column, 0);
                assert_eq!(span.end.column, 11);
            }
            _ => panic!("Expected Text token"),
        }
    }
}
