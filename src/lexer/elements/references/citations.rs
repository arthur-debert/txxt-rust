//! Citation reference tokenization for TXXT ([@key])
//!
//! Handles parsing of citation references wrapped in square brackets with @ prefix
//! according to the TXXT specification. Citation references are used to reference
//! external sources and bibliographic entries.

use crate::cst::{Position, ScannerToken, SourceSpan};

/// Trait for lexer state that can parse citation references
pub trait CitationRefLexer {
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

/// Read a complete citation reference ([@key]) if present at current position
pub fn read_citation_ref<L: CitationRefLexer>(lexer: &mut L) -> Option<ScannerToken> {
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

    // Must have @ next
    if lexer.peek() != Some('@') {
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }

    lexer.advance(); // Consume @

    let mut content = String::new();
    let mut found_closing = false;

    // Read content until closing ] or end of line
    while let Some(ch) = lexer.peek() {
        if ch == ']' {
            lexer.advance(); // Consume closing ]
            found_closing = true;
            break;
        } else if ch == '\n' || ch == '\r' {
            // Citation references cannot cross line boundaries
            break;
        } else if ch == '[' || ch == '@' {
            // Invalid characters inside citation reference
            break;
        } else if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == ':' {
            // Valid characters for citation keys
            content.push(ch);
            lexer.advance();
        } else {
            // Invalid character
            break;
        }
    }

    if !found_closing || content.is_empty() {
        // Not a valid citation reference, backtrack
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }

    Some(ScannerToken::CitationRef {
        content,
        span: SourceSpan {
            start: start_pos,
            end: lexer.current_position(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenization::Lexer;

    #[test]
    fn test_simple_citation_ref() {
        let mut lexer = Lexer::new("[@smith2020]");
        let token = read_citation_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::CitationRef { content, span } => {
                assert_eq!(content, "smith2020");
                assert_eq!(span.start.row, 0);
                assert_eq!(span.start.column, 0);
                assert_eq!(span.end.row, 0);
                assert_eq!(span.end.column, 12);
            }
            _ => panic!("Expected CitationRef token"),
        }
    }

    #[test]
    fn test_citation_ref_with_dots_and_hyphens() {
        let mut lexer = Lexer::new("[@smith-jones.2020]");
        let token = read_citation_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::CitationRef { content, .. } => {
                assert_eq!(content, "smith-jones.2020");
            }
            _ => panic!("Expected CitationRef token"),
        }
    }

    #[test]
    fn test_citation_ref_with_namespace() {
        let mut lexer = Lexer::new("[@author:smith2020]");
        let token = read_citation_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::CitationRef { content, .. } => {
                assert_eq!(content, "author:smith2020");
            }
            _ => panic!("Expected CitationRef token"),
        }
    }

    #[test]
    fn test_citation_ref_empty() {
        let mut lexer = Lexer::new("[@]");
        let token = read_citation_ref(&mut lexer);

        // Empty citation references should be invalid
        assert!(token.is_none());
    }

    #[test]
    fn test_citation_ref_unclosed() {
        let mut lexer = Lexer::new("[@smith2020");
        let token = read_citation_ref(&mut lexer);

        // Unclosed citation references should be invalid
        assert!(token.is_none());
        // Should backtrack to original position
        assert_eq!(lexer.position(), 0);
    }

    #[test]
    fn test_citation_ref_invalid_chars() {
        let mut lexer = Lexer::new("[@smith 2020]");
        let token = read_citation_ref(&mut lexer);

        // Citation refs with spaces should be invalid
        assert!(token.is_none());
    }

    #[test]
    fn test_citation_ref_line_break() {
        let mut lexer = Lexer::new("[@smith\n2020]");
        let token = read_citation_ref(&mut lexer);

        // Citation references cannot cross lines
        assert!(token.is_none());
    }

    #[test]
    fn test_not_citation_ref() {
        let mut lexer = Lexer::new("[text]");
        let token = read_citation_ref(&mut lexer);

        // Should not match without @
        assert!(token.is_none());
    }

    #[test]
    fn test_citation_ref_with_numbers() {
        let mut lexer = Lexer::new("[@ref123]");
        let token = read_citation_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::CitationRef { content, .. } => {
                assert_eq!(content, "ref123");
            }
            _ => panic!("Expected CitationRef token"),
        }
    }

    #[test]
    fn test_citation_ref_with_underscores() {
        let mut lexer = Lexer::new("[@my_ref_2020]");
        let token = read_citation_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            ScannerToken::CitationRef { content, .. } => {
                assert_eq!(content, "my_ref_2020");
            }
            _ => panic!("Expected CitationRef token"),
        }
    }
}
