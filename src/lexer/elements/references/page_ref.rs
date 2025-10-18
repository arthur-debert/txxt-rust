//! Page reference tokenization for TXXT ([p.123])
//!
//! Handles parsing of page references wrapped in square brackets with p. prefix
//! according to the TXXT specification. Page references are used to reference
//! specific pages or page ranges in documents.

use crate::ast::scanner_tokens::{Position, SourceSpan, ScannerToken};

/// Trait for lexer state that can parse page references
pub trait PageRefLexer {
    /// Get the current position in the input
    fn current_position(&self) -> Position;

    /// Peek at the current character without advancing
    fn peek(&self) -> Option<char>;

    /// Peek at character at offset from current position
    fn peek_at(&self, offset: usize) -> Option<char>;

    /// Advance to the next character and return it
    fn advance(&mut self) -> Option<char>;

    /// Get current row (line number)
    fn row(&self) -> usize;

    /// Get current column
    fn column(&self) -> usize;

    /// Get current position index
    fn position(&self) -> usize;

    /// Backtrack to a saved position
    fn backtrack(&mut self, position: usize, row: usize, column: usize);
}

/// Read a complete page reference ([p.123] or [p.123-125]) if present at current position
pub fn read_page_ref<L: PageRefLexer>(lexer: &mut L) -> Option<ScannerToken> {
    let start_pos = lexer.current_position();

    // Must start with [
    if lexer.peek() != Some('[') {
        return None;
    }

    // Save position for potential backtracking
    let saved_position = lexer.position();
    let saved_row = lexer.row();
    let saved_column = lexer.column();

    lexer.advance(); // Consume opening [

    // Must have p next
    if lexer.peek() != Some('p') {
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }

    lexer.advance(); // Consume p

    // Must have . next
    if lexer.peek() != Some('.') {
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }

    lexer.advance(); // Consume .

    let mut content = String::new();
    let mut found_closing = false;

    // Read content until closing ] or end of line
    while let Some(ch) = lexer.peek() {
        if ch == ']' {
            lexer.advance(); // Consume closing ]
            found_closing = true;
            break;
        } else if ch == '\n' || ch == '\r' {
            // Page references cannot cross line boundaries
            break;
        } else if ch == '[' {
            // Invalid character inside page reference
            break;
        } else if ch.is_ascii_digit() || ch == '-' {
            // Valid characters for page numbers and ranges
            content.push(ch);
            lexer.advance();
        } else {
            // Invalid character
            break;
        }
    }

    if !found_closing || content.is_empty() || !is_valid_page_content(&content) {
        // Not a valid page reference, backtrack
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }

    Some(ScannerToken::PageRef {
        content,
        span: SourceSpan {
            start: start_pos,
            end: lexer.current_position(),
        },
    })
}

/// Validate page reference content
fn is_valid_page_content(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    // Check for valid page number patterns: "123", "123-125"
    if content.contains('-') {
        // Page range: should be "start-end" where both are numbers
        let parts: Vec<&str> = content.split('-').collect();
        if parts.len() != 2 {
            return false;
        }

        // Both parts must be valid numbers
        parts[0].chars().all(|c| c.is_ascii_digit())
            && parts[1].chars().all(|c| c.is_ascii_digit())
            && !parts[0].is_empty()
            && !parts[1].is_empty()
    } else {
        // Single page: must be all digits
        content.chars().all(|c| c.is_ascii_digit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::core::lexer::Lexer;

    #[test]
    fn test_simple_page_ref() {
        let mut lexer = Lexer::new("[p.123]");
        let token = read_page_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::PageRef { content, span } => {
                assert_eq!(content, "123");
                assert_eq!(span.start.row, 0);
                assert_eq!(span.start.column, 0);
                assert_eq!(span.end.row, 0);
                assert_eq!(span.end.column, 7);
            }
            _ => panic!("Expected PageRef token"),
        }
    }

    #[test]
    fn test_page_range_ref() {
        let mut lexer = Lexer::new("[p.123-125]");
        let token = read_page_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::PageRef { content, .. } => {
                assert_eq!(content, "123-125");
            }
            _ => panic!("Expected PageRef token"),
        }
    }

    #[test]
    fn test_page_ref_single_digit() {
        let mut lexer = Lexer::new("[p.5]");
        let token = read_page_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::PageRef { content, .. } => {
                assert_eq!(content, "5");
            }
            _ => panic!("Expected PageRef token"),
        }
    }

    #[test]
    fn test_page_ref_empty() {
        let mut lexer = Lexer::new("[p.]");
        let token = read_page_ref(&mut lexer);

        // Empty page references should be invalid
        assert!(token.is_none());
    }

    #[test]
    fn test_page_ref_unclosed() {
        let mut lexer = Lexer::new("[p.123");
        let token = read_page_ref(&mut lexer);

        // Unclosed page references should be invalid
        assert!(token.is_none());
        // Should backtrack to original position
        assert_eq!(lexer.position(), 0);
    }

    #[test]
    fn test_page_ref_invalid_chars() {
        let mut lexer = Lexer::new("[p.12a3]");
        let token = read_page_ref(&mut lexer);

        // Page refs with letters should be invalid
        assert!(token.is_none());
    }

    #[test]
    fn test_page_ref_line_break() {
        let mut lexer = Lexer::new("[p.12\n3]");
        let token = read_page_ref(&mut lexer);

        // Page references cannot cross lines
        assert!(token.is_none());
    }

    #[test]
    fn test_not_page_ref() {
        let mut lexer = Lexer::new("[text]");
        let token = read_page_ref(&mut lexer);

        // Should not match without p.
        assert!(token.is_none());
    }

    #[test]
    fn test_page_ref_missing_dot() {
        let mut lexer = Lexer::new("[p123]");
        let token = read_page_ref(&mut lexer);

        // Should not match without dot after p
        assert!(token.is_none());
    }

    #[test]
    fn test_page_ref_invalid_range() {
        let mut lexer = Lexer::new("[p.123-]");
        let token = read_page_ref(&mut lexer);

        // Invalid range should be rejected
        assert!(token.is_none());
    }

    #[test]
    fn test_page_ref_multiple_dashes() {
        let mut lexer = Lexer::new("[p.123-125-127]");
        let token = read_page_ref(&mut lexer);

        // Multiple dashes should be invalid
        assert!(token.is_none());
    }

    #[test]
    fn test_valid_page_content() {
        assert!(is_valid_page_content("123"));
        assert!(is_valid_page_content("1"));
        assert!(is_valid_page_content("123-125"));
        assert!(is_valid_page_content("1-999"));

        assert!(!is_valid_page_content(""));
        assert!(!is_valid_page_content("abc"));
        assert!(!is_valid_page_content("123-"));
        assert!(!is_valid_page_content("-123"));
        assert!(!is_valid_page_content("123-125-127"));
        assert!(!is_valid_page_content("12a3"));
    }
}
