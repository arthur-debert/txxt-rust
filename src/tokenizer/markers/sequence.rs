//! Sequence marker parsing for lists
//!
//! Handles detection and parsing of list sequence markers as defined in the
//! TXXT specification, including:
//! - Plain markers: "- "
//! - Numerical markers: "1. ", "42. "
//! - Alphabetical markers: "a. ", "Z) "
//! - Roman numeral markers: "i. ", "III) "

use crate::ast::tokens::{Position, SourceSpan, Token};

/// Read a sequence marker token (list markers like "1. ", "a) ", "- ")
///
/// This function takes a lexer-like interface and attempts to parse a sequence marker
/// at the current position. It handles all four types of sequence markers defined
/// in the TXXT specification.
pub fn read_sequence_marker<L>(lexer: &mut L) -> Option<Token>
where
    L: SequenceMarkerLexer,
{
    let start_pos = lexer.current_position();
    let saved_state = lexer.save_state();

    // Try to match sequence marker patterns

    // 1. Plain dash marker: "- "
    if lexer.peek() == Some('-') {
        lexer.advance();
        if lexer.peek() == Some(' ') {
            lexer.advance();
            return Some(Token::SequenceMarker {
                content: "-".to_string(),
                span: SourceSpan {
                    start: start_pos,
                    end: Position {
                        row: start_pos.row,
                        column: start_pos.column + 1,
                    },
                },
            });
        } else {
            // Not a valid marker, backtrack
            lexer.restore_state(saved_state.clone());
            return None;
        }
    }

    // 2. Numbered markers: "1. ", "42. "
    let mut number_str = String::new();
    while let Some(ch) = lexer.peek() {
        if ch.is_ascii_digit() {
            number_str.push(ch);
            lexer.advance();
        } else {
            break;
        }
    }

    if !number_str.is_empty() {
        if lexer.peek() == Some('.') {
            lexer.advance();
            if lexer.peek() == Some(' ') {
                lexer.advance();
                let marker = format!("{}.", number_str);
                return Some(Token::SequenceMarker {
                    content: marker.clone(),
                    span: SourceSpan {
                        start: start_pos,
                        end: Position {
                            row: start_pos.row,
                            column: start_pos.column + marker.len(),
                        },
                    },
                });
            }
        }
        // Not a valid marker, backtrack
        lexer.restore_state(saved_state.clone());
        return None;
    }

    // 3. Alphabetical markers: "a. ", "b) ", "A. ", "Z) "
    if let Some(ch) = lexer.peek() {
        if ch.is_ascii_alphabetic() {
            let letter = ch;
            lexer.advance();

            // Check for . or )
            if let Some(punct) = lexer.peek() {
                if punct == '.' || punct == ')' {
                    lexer.advance();
                    if lexer.peek() == Some(' ') {
                        lexer.advance();
                        let marker = format!("{}{}", letter, punct);
                        return Some(Token::SequenceMarker {
                            content: marker.clone(),
                            span: SourceSpan {
                                start: start_pos,
                                end: Position {
                                    row: start_pos.row,
                                    column: start_pos.column + marker.len(),
                                },
                            },
                        });
                    }
                }
            }
        }
    }

    // 4. Roman numeral markers: "i. ", "ii) ", "I. ", "III) "
    lexer.restore_state(saved_state.clone());

    let roman_patterns = [
        "iii", "ii", "i", "iv", "v", "vi", "vii", "viii", "ix", "x", "III", "II", "I", "IV", "V",
        "VI", "VII", "VIII", "IX", "X",
    ];

    for pattern in &roman_patterns {
        if lexer.matches_string(pattern) {
            // Advance past the roman numeral
            for _ in 0..pattern.len() {
                lexer.advance();
            }

            // Check for . or )
            if let Some(punct) = lexer.peek() {
                if punct == '.' || punct == ')' {
                    lexer.advance();
                    if lexer.peek() == Some(' ') {
                        lexer.advance();
                        let marker = format!("{}{}", pattern, punct);
                        return Some(Token::SequenceMarker {
                            content: marker.clone(),
                            span: SourceSpan {
                                start: start_pos,
                                end: Position {
                                    row: start_pos.row,
                                    column: start_pos.column + marker.len(),
                                },
                            },
                        });
                    }
                }
            }

            // Not a valid marker, backtrack and try next pattern
            lexer.restore_state(saved_state.clone());
        }
    }

    None
}

/// Trait that defines the interface required for sequence marker parsing
///
/// This trait abstracts the lexer interface needed for sequence marker parsing,
/// making the parsing logic testable and reusable.
pub trait SequenceMarkerLexer {
    /// Type for saving/restoring lexer state
    type State: Clone;

    /// Get the current position in the input
    fn current_position(&self) -> Position;

    /// Peek at the current character without advancing
    fn peek(&self) -> Option<char>;

    /// Advance to the next character
    fn advance(&mut self) -> Option<char>;

    /// Check if the current position matches a given string
    fn matches_string(&self, s: &str) -> bool;

    /// Save the current lexer state for backtracking
    fn save_state(&self) -> Self::State;

    /// Restore a previously saved lexer state
    fn restore_state(&mut self, state: Self::State);
}
