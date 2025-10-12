//! Math span tokenization for TXXT (#content#)
//!
//! Handles parsing of math expressions wrapped in hash delimiters according to
//! the TXXT specification. Math spans preserve their content literally and
//! are processed for mathematical rendering separately.

use crate::ast::tokens::{Position, SourceSpan, Token};

/// Trait for lexer state that can parse math spans
pub trait MathSpanLexer {
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

/// Read a complete math span (#content#) if present at current position
pub fn read_math_span<L: MathSpanLexer>(lexer: &mut L) -> Option<Token> {
    let start_pos = lexer.current_position();

    // Must start with #
    if lexer.peek() != Some('#') {
        return None;
    }

    // Save position for potential backtracking
    let saved_position = lexer.position();
    let saved_row = lexer.row();
    let saved_column = lexer.column();

    lexer.advance(); // Consume opening #

    let mut content = String::new();
    let mut found_closing = false;

    // Read content until closing # or end of line
    while let Some(ch) = lexer.peek() {
        if ch == '#' {
            lexer.advance(); // Consume closing #
            found_closing = true;
            break;
        } else if ch == '\n' || ch == '\r' {
            // Math spans cannot cross line boundaries
            break;
        } else {
            content.push(ch);
            lexer.advance();
        }
    }

    if !found_closing || content.is_empty() {
        // Not a valid math span, backtrack
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }

    Some(Token::MathSpan {
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
    use crate::tokenizer::infrastructure::lexer::Lexer;

    #[test]
    fn test_simple_math_span() {
        let mut lexer = Lexer::new("#x+y#");
        let token = read_math_span(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::MathSpan { content, span } => {
                assert_eq!(content, "x+y");
                assert_eq!(span.start.row, 0);
                assert_eq!(span.start.column, 0);
                assert_eq!(span.end.row, 0);
                assert_eq!(span.end.column, 5);
            }
            _ => panic!("Expected MathSpan token"),
        }
    }

    #[test]
    fn test_math_span_with_spaces() {
        let mut lexer = Lexer::new("# x + y #");
        let token = read_math_span(&mut lexer);

        assert!(token.is_some());
        match token.unwrap() {
            Token::MathSpan { content, .. } => {
                assert_eq!(content, " x + y ");
            }
            _ => panic!("Expected MathSpan token"),
        }
    }

    #[test]
    fn test_math_span_empty() {
        let mut lexer = Lexer::new("##");
        let token = read_math_span(&mut lexer);

        // Empty math spans should be invalid
        assert!(token.is_none());
    }

    #[test]
    fn test_math_span_unclosed() {
        let mut lexer = Lexer::new("#formula");
        let token = read_math_span(&mut lexer);

        // Unclosed math spans should be invalid
        assert!(token.is_none());
        // Should backtrack to original position
        assert_eq!(lexer.position(), 0);
    }

    #[test]
    fn test_math_span_with_internal_hash() {
        let mut lexer = Lexer::new("#x#y#");
        let token = read_math_span(&mut lexer);

        // Should parse the first valid math span #x#
        assert!(token.is_some());
        match token.unwrap() {
            Token::MathSpan { content, .. } => {
                assert_eq!(content, "x");
            }
            _ => panic!("Expected MathSpan token"),
        }

        // After parsing #x#, the lexer should be positioned at 'y'
        assert_eq!(lexer.position(), 3);
    }

    #[test]
    fn test_math_span_with_unescaped_internal_hash() {
        let mut lexer = Lexer::new("#a#b#c#");
        let token = read_math_span(&mut lexer);

        // This should parse #a# first
        assert!(token.is_some());
        match token.unwrap() {
            Token::MathSpan { content, .. } => {
                assert_eq!(content, "a");
            }
            _ => panic!("Expected MathSpan token"),
        }
    }

    #[test]
    fn test_math_span_line_break() {
        let mut lexer = Lexer::new("#x+\ny#");
        let token = read_math_span(&mut lexer);

        // Math spans cannot cross lines
        assert!(token.is_none());
    }

    #[test]
    fn test_standalone_hash() {
        let mut lexer = Lexer::new("#");
        let token = read_math_span(&mut lexer);

        // Standalone # should not be a math span
        assert!(token.is_none());
    }
}
