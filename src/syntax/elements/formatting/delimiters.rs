//! Inline formatting delimiter parsing
//!
//! Handles detection and parsing of inline formatting delimiters as defined
//! in the TXXT specification:
//! - Bold delimiters: *
//! - Italic delimiters: _
//! - Code delimiters: `
//! - Math delimiters: #

use crate::cst::{Position, ScannerToken, SourceSpan};
use crate::syntax::tokenization::Lexer;

/// Trait for lexer state that can parse inline delimiters
pub trait InlineDelimiterLexer {
    /// Get the current position in the input
    fn current_position(&self) -> Position;

    /// Peek at the current character without advancing
    fn peek(&self) -> Option<char>;

    /// Peek at character at offset from current position
    fn peek_at(&self, offset: usize) -> Option<char>;

    /// Advance to the next character and return it
    fn advance(&mut self) -> Option<char>;
}

/// Check if an underscore is likely part of an identifier rather than a formatting delimiter
fn is_likely_identifier_underscore<L: InlineDelimiterLexer>(lexer: &L) -> bool {
    // Get current character (should be '_')
    if lexer.peek() != Some('_') {
        return false;
    }

    // Look at what comes after the underscore
    let next_char = lexer.peek_at(1);

    match next_char {
        Some('_') => {
            // __ pattern - could be identifier or double delimiter
            // Look at what comes after the second underscore
            match lexer.peek_at(2) {
                Some(ch) if ch.is_alphanumeric() => {
                    // __letter - likely identifier like __test
                    true
                }
                _ => {
                    // __ at end or followed by non-alphanumeric - likely double delimiter
                    false
                }
            }
        }
        Some(ch) if ch.is_alphanumeric() => {
            // _letter or _digit - could be identifier start
            // Look further ahead to see if there's a matching underscore (suggesting delimiter pair)
            let mut pos = 2;
            while let Some(ch) = lexer.peek_at(pos) {
                if ch == '_' {
                    // Found another underscore - this suggests delimiter pair like _text_
                    return false;
                } else if ch.is_alphanumeric() {
                    pos += 1;
                    continue;
                } else {
                    // Hit non-alphanumeric, non-underscore - likely end of identifier
                    break;
                }
            }
            // No matching underscore found, likely identifier
            true
        }
        Some(' ') | Some('\t') | Some('\n') | Some('\r') | Some(':') | None => {
            // Underscore followed by whitespace, colon, or end
            // This could be either identifier end or standalone delimiter
            // Need to look at context to decide
            // For now, let's be conservative and treat as delimiter unless we have strong evidence it's an identifier
            false
        }
        _ => {
            // Underscore followed by other punctuation - likely formatting delimiter
            false
        }
    }
}

/// Read inline formatting delimiters (*, _, `, #)
pub fn read_inline_delimiter<L: InlineDelimiterLexer>(lexer: &mut L) -> Option<ScannerToken> {
    let start_pos = lexer.current_position();

    match lexer.peek()? {
        '*' => {
            lexer.advance();
            Some(ScannerToken::BoldDelimiter {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            })
        }
        '_' => {
            // Be more conservative about underscores - check if this might be part of an identifier
            if is_likely_identifier_underscore(lexer) {
                None // Let text/identifier handler take it
            } else {
                lexer.advance();
                Some(ScannerToken::ItalicDelimiter {
                    span: SourceSpan {
                        start: start_pos,
                        end: lexer.current_position(),
                    },
                })
            }
        }
        '`' => {
            lexer.advance();
            Some(ScannerToken::CodeDelimiter {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            })
        }
        '#' => {
            lexer.advance();
            Some(ScannerToken::MathDelimiter {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            })
        }
        _ => None,
    }
}

impl InlineDelimiterLexer for Lexer {
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
}
