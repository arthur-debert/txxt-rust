//! Sequence marker parsing for lists
//!
//! Handles detection and parsing of list sequence markers as defined in the
//! TXXT specification, including:
//! - Plain markers: "- "
//! - Numerical markers: "1. ", "42. "
//! - Alphabetical markers: "a. ", "Z) "
//! - Roman numeral markers: "i. ", "III) "

use crate::ast::tokens::{Position, SequenceMarkerType, SourceSpan, Token};
use crate::tokenizer::core::lexer::{Lexer, LexerState};

/// Convert Roman numeral string to number
fn roman_to_number(roman: &str) -> u64 {
    match roman.to_lowercase().as_str() {
        "i" => 1,
        "ii" => 2,
        "iii" => 3,
        "iv" => 4,
        "v" => 5,
        "vi" => 6,
        "vii" => 7,
        "viii" => 8,
        "ix" => 9,
        "x" => 10,
        "xi" => 11,
        "xii" => 12,
        "xiii" => 13,
        _ => 0, // Default for unknown patterns
    }
}

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
                marker_type: SequenceMarkerType::Plain("-".to_string()),
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
        // Check for both . and ) endings
        if let Some(punct) = lexer.peek() {
            if punct == '.' || punct == ')' {
                lexer.advance();
                if lexer.peek() == Some(' ') {
                    lexer.advance();
                    let marker = format!("{}{}", number_str, punct);
                    let number = number_str.parse::<u64>().unwrap_or(0);
                    return Some(Token::SequenceMarker {
                        marker_type: SequenceMarkerType::Numerical(number, marker.clone()),
                        span: SourceSpan {
                            start: start_pos,
                            end: Position {
                                row: start_pos.row,
                                column: start_pos.column + marker.chars().count(),
                            },
                        },
                    });
                }
            }
        }
        // Not a valid marker, backtrack
        lexer.restore_state(saved_state.clone());
        return None;
    }

    // 3. Roman numeral markers: "i. ", "ii) ", "I. ", "III) " - check before alphabetical
    lexer.restore_state(saved_state.clone());

    let roman_patterns = [
        // Longer patterns first to avoid partial matches
        "xiii", "xii", "xi", "viii", "vii", "iii", "ii", "iv", "vi", "ix", "i", "v", "x", "XIII",
        "XII", "XI", "VIII", "VII", "III", "II", "IV", "VI", "IX", "I", "V", "X",
    ];

    for pattern in &roman_patterns {
        if lexer.matches_string(pattern) {
            // Advance past the roman numeral
            for _ in 0..pattern.chars().count() {
                lexer.advance();
            }

            // Check for . or )
            if let Some(punct) = lexer.peek() {
                if punct == '.' || punct == ')' {
                    lexer.advance();
                    if lexer.peek() == Some(' ') {
                        lexer.advance();
                        let marker = format!("{}{}", pattern, punct);
                        let roman_value = roman_to_number(pattern);
                        return Some(Token::SequenceMarker {
                            marker_type: SequenceMarkerType::Roman(roman_value, marker.clone()),
                            span: SourceSpan {
                                start: start_pos,
                                end: Position {
                                    row: start_pos.row,
                                    column: start_pos.column + marker.chars().count(),
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

    // 4. Alphabetical markers: "a. ", "b) ", "A. ", "Z) " - check after roman numerals
    lexer.restore_state(saved_state.clone());
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
                            marker_type: SequenceMarkerType::Alphabetical(letter, marker.clone()),
                            span: SourceSpan {
                                start: start_pos,
                                end: Position {
                                    row: start_pos.row,
                                    column: start_pos.column + marker.chars().count(),
                                },
                            },
                        });
                    }
                }
            }

            // Not a valid alphabetical marker, backtrack
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

impl SequenceMarkerLexer for Lexer {
    type State = LexerState;

    fn current_position(&self) -> Position {
        Position {
            row: self.row,
            column: self.column,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
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

    fn matches_string(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, &expected_char) in chars.iter().enumerate() {
            if let Some(actual_char) = self.input.get(self.position + i) {
                if *actual_char != expected_char {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn save_state(&self) -> Self::State {
        LexerState {
            position: self.position,
            row: self.row,
            column: self.column,
        }
    }

    fn restore_state(&mut self, state: Self::State) {
        self.position = state.position;
        self.row = state.row;
        self.column = state.column;
    }
}
