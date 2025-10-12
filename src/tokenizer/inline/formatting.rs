//! Inline formatting delimiter parsing
//!
//! Handles detection and parsing of inline formatting delimiters as defined
//! in the TXXT specification:
//! - Bold delimiters: *
//! - Italic delimiters: _
//! - Code delimiters: `
//! - Math delimiters: #

use crate::ast::tokens::{Position, SourceSpan, Token};

/// Trait for lexer state that can parse inline delimiters
pub trait InlineDelimiterLexer {
    /// Get the current position in the input
    fn current_position(&self) -> Position;

    /// Peek at the current character without advancing
    fn peek(&self) -> Option<char>;

    /// Advance to the next character and return it
    fn advance(&mut self) -> Option<char>;
}

/// Read inline formatting delimiters (*, _, `, #)
pub fn read_inline_delimiter<L: InlineDelimiterLexer>(lexer: &mut L) -> Option<Token> {
    let start_pos = lexer.current_position();

    match lexer.peek()? {
        '*' => {
            lexer.advance();
            Some(Token::BoldDelimiter {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            })
        }
        '_' => {
            lexer.advance();
            Some(Token::ItalicDelimiter {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            })
        }
        '`' => {
            lexer.advance();
            Some(Token::CodeDelimiter {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            })
        }
        '#' => {
            lexer.advance();
            Some(Token::MathDelimiter {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            })
        }
        _ => None,
    }
}
