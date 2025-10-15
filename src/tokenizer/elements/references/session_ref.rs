//! Session reference tokenization ([#1.2])
//!
//! Handles semantic tokenization of session references that follow the pattern:
//! - `[#1]` - Simple numeric session reference
//! - `[#1.2]` - Hierarchical session reference  
//! - `[#1.2.3]` - Deep hierarchical session reference
//!
//! Session references use the `#` prefix within brackets to reference document
//! sections by their hierarchical numbering. This provides type-safe navigation
//! within documents and better language server support compared to generic
//! reference markers.

use crate::ast::tokens::{Position, SourceSpan, Token};

/// Lexer trait for session reference parsing
pub trait SessionRefLexer {
    fn current_position(&self) -> Position;
    fn peek(&self) -> Option<char>;
    fn peek_at(&self, offset: usize) -> Option<char>;
    fn advance(&mut self) -> Option<char>;
    fn row(&self) -> usize;
    fn column(&self) -> usize;
    fn position(&self) -> usize;
    fn backtrack(&mut self, position: usize, row: usize, column: usize);
}

/// Read a session reference token ([#1.2])
pub fn read_session_ref<L: SessionRefLexer>(lexer: &mut L) -> Option<Token> {
    let start_pos = lexer.current_position();
    let saved_position = lexer.position();
    let saved_row = lexer.row();
    let saved_column = lexer.column();

    // Must start with [
    if lexer.peek() != Some('[') {
        return None;
    }
    lexer.advance();

    // Must be followed by #
    if lexer.peek() != Some('#') {
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }
    lexer.advance();

    // Collect session number content after #
    let mut content = String::new();

    while let Some(ch) = lexer.peek() {
        if ch.is_ascii_digit() || ch == '.' || ch == '-' {
            content.push(ch);
            lexer.advance();
        } else if ch == ']' {
            // Found closing bracket - validate content and create token
            if is_valid_session_content(&content) {
                lexer.advance(); // consume ]

                return Some(Token::SessionRef {
                    content,
                    span: SourceSpan {
                        start: start_pos,
                        end: lexer.current_position(),
                    },
                });
            } else {
                // Invalid content, backtrack
                lexer.backtrack(saved_position, saved_row, saved_column);
                return None;
            }
        } else {
            // Invalid character, backtrack
            lexer.backtrack(saved_position, saved_row, saved_column);
            return None;
        }
    }

    // Reached end of input without finding closing bracket
    lexer.backtrack(saved_position, saved_row, saved_column);
    None
}

/// Validate session reference content
///
/// Session numbers follow the pattern: (-1|digit+) (. (-1|digit+))*
/// Examples: "1", "1.2", "1.2.3", "10.20.30", "-1", "1.-1.2", "-1.-1"
fn is_valid_session_content(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    // Split by dots and validate each part
    let parts: Vec<&str> = content.split('.').collect();

    for part in parts {
        if part.is_empty() {
            return false; // No empty parts allowed (e.g., "1..2" or ".1")
        }

        // Each part must be either "-1" or all digits
        if part == "-1" {
            continue; // -1 is valid
        } else if part.chars().all(|c| c.is_ascii_digit()) {
            continue; // All digits is valid
        } else {
            return false; // Invalid part
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tokens::Position;

    struct MockLexer {
        input: Vec<char>,
        position: usize,
        row: usize,
        column: usize,
    }

    impl MockLexer {
        fn new(input: &str) -> Self {
            Self {
                input: input.chars().collect(),
                position: 0,
                row: 0,
                column: 0,
            }
        }
    }

    impl SessionRefLexer for MockLexer {
        fn current_position(&self) -> Position {
            Position {
                row: self.row,
                column: self.column,
            }
        }

        fn peek(&self) -> Option<char> {
            self.input.get(self.position).copied()
        }

        fn peek_at(&self, offset: usize) -> Option<char> {
            self.input.get(self.position + offset).copied()
        }

        fn advance(&mut self) -> Option<char> {
            if let Some(ch) = self.input.get(self.position).copied() {
                self.position += 1;
                if ch == '\n' {
                    self.row += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
                Some(ch)
            } else {
                None
            }
        }

        fn row(&self) -> usize {
            self.row
        }

        fn column(&self) -> usize {
            self.column
        }

        fn position(&self) -> usize {
            self.position
        }

        fn backtrack(&mut self, position: usize, row: usize, column: usize) {
            self.position = position;
            self.row = row;
            self.column = column;
        }
    }

    #[test]
    fn test_simple_session_ref() {
        let mut lexer = MockLexer::new("[#1]");
        let token = read_session_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::SessionRef { content, .. } => {
                assert_eq!(content, "1");
            }
            _ => panic!("Expected SessionRef token"),
        }
    }

    #[test]
    fn test_hierarchical_session_ref() {
        let mut lexer = MockLexer::new("[#1.2]");
        let token = read_session_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::SessionRef { content, .. } => {
                assert_eq!(content, "1.2");
            }
            _ => panic!("Expected SessionRef token"),
        }
    }

    #[test]
    fn test_deep_hierarchical_session_ref() {
        let mut lexer = MockLexer::new("[#1.2.3]");
        let token = read_session_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::SessionRef { content, .. } => {
                assert_eq!(content, "1.2.3");
            }
            _ => panic!("Expected SessionRef token"),
        }
    }

    #[test]
    fn test_multi_digit_session_ref() {
        let mut lexer = MockLexer::new("[#10.20.30]");
        let token = read_session_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::SessionRef { content, .. } => {
                assert_eq!(content, "10.20.30");
            }
            _ => panic!("Expected SessionRef token"),
        }
    }

    #[test]
    fn test_session_ref_empty() {
        let mut lexer = MockLexer::new("[#]");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_session_ref_invalid_chars() {
        let mut lexer = MockLexer::new("[#1.a]");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_session_ref_invalid_double_dot() {
        let mut lexer = MockLexer::new("[#1..2]");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_session_ref_starting_dot() {
        let mut lexer = MockLexer::new("[#.1]");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_session_ref_ending_dot() {
        let mut lexer = MockLexer::new("[#1.]");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_session_ref_unclosed() {
        let mut lexer = MockLexer::new("[#1.2");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_session_ref_line_break() {
        let mut lexer = MockLexer::new("[#1\n2]");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_not_session_ref() {
        let mut lexer = MockLexer::new("[something]");
        let token = read_session_ref(&mut lexer);
        assert!(token.is_none());
    }

    #[test]
    fn test_session_ref_negative_indexing() {
        let mut lexer = MockLexer::new("[#-1]");
        let token = read_session_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::SessionRef { content, .. } => {
                assert_eq!(content, "-1");
            }
            _ => panic!("Expected SessionRef token"),
        }
    }

    #[test]
    fn test_session_ref_mixed_negative() {
        let mut lexer = MockLexer::new("[#1.-1.2]");
        let token = read_session_ref(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::SessionRef { content, .. } => {
                assert_eq!(content, "1.-1.2");
            }
            _ => panic!("Expected SessionRef token"),
        }
    }

    #[test]
    fn test_valid_session_content() {
        assert!(is_valid_session_content("1"));
        assert!(is_valid_session_content("1.2"));
        assert!(is_valid_session_content("1.2.3"));
        assert!(is_valid_session_content("10.20.30"));
        assert!(is_valid_session_content("-1"));
        assert!(is_valid_session_content("1.-1"));
        assert!(is_valid_session_content("-1.-1"));
        assert!(is_valid_session_content("1.-1.2"));

        assert!(!is_valid_session_content(""));
        assert!(!is_valid_session_content("."));
        assert!(!is_valid_session_content("1."));
        assert!(!is_valid_session_content(".1"));
        assert!(!is_valid_session_content("1..2"));
        assert!(!is_valid_session_content("1.a"));
        assert!(!is_valid_session_content("a.1"));
        assert!(!is_valid_session_content("-2")); // Only -1 is valid negative
        assert!(!is_valid_session_content("1.-2"));
    }
}
