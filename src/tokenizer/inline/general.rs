//! General inline element tokenization
//!
//! Implements the foundation for inline elements as defined in
//! docs/specs/elements/inlines/inlines-general.txxt
//!
//! Provides common functionality for all inline element types

use crate::ast::tokens::Position;

/// Common interface for inline element detection
pub trait InlineElementLexer {
    /// Get current position
    fn current_position(&self) -> Position;

    /// Peek at current character
    fn peek(&self) -> Option<char>;

    /// Advance to next character
    fn advance(&mut self) -> Option<char>;

    /// Check if at end of input
    fn is_at_end(&self) -> bool;
}

/// Detect if a character sequence represents an inline element boundary
pub fn is_inline_boundary(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '\t' | '\n' | '\r' | '*' | '_' | '`' | '#' | '[' | ']' | ':'
    )
}

/// Skip whitespace and return the next non-whitespace character
pub fn skip_whitespace<L: InlineElementLexer>(lexer: &mut L) -> Option<char> {
    while let Some(ch) = lexer.peek() {
        if !ch.is_whitespace() {
            return Some(ch);
        }
        lexer.advance();
    }
    None
}

/// Check if a character is valid for inline content
pub fn is_valid_inline_char(ch: char) -> bool {
    // Most characters are valid inline content except for structural markers
    !matches!(ch, '\n' | '\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_boundary_detection() {
        assert!(is_inline_boundary(' '));
        assert!(is_inline_boundary('*'));
        assert!(is_inline_boundary('_'));
        assert!(is_inline_boundary('['));
        assert!(!is_inline_boundary('a'));
        assert!(!is_inline_boundary('1'));
    }

    #[test]
    fn test_valid_inline_char() {
        assert!(is_valid_inline_char('a'));
        assert!(is_valid_inline_char('1'));
        assert!(is_valid_inline_char(' '));
        assert!(is_valid_inline_char('*'));
        assert!(!is_valid_inline_char('\n'));
        assert!(!is_valid_inline_char('\r'));
    }
}
