//! TXXT Lexer - Character-precise tokenization for new AST
//!
//! Converts TXXT source text into Token enum variants with precise SourceSpan
//! positioning for language server support.

use crate::ast::tokens::{Position, SourceSpan, Token};

/// Main tokenizer that produces new AST Token enum variants
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    row: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given input text
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            row: 0,
            column: 0,
        }
    }

    /// Tokenize the input text into Token enum variants
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            // Try to read sequence marker only at column 0 (start of line)
            if self.column == 0 {
                if let Some(token) = self.read_sequence_marker() {
                    tokens.push(token);
                    continue;
                }
            }

            // Handle newlines first (they're significant tokens)
            if let Some(ch) = self.peek() {
                if ch == '\n' {
                    if let Some(token) = self.read_newline() {
                        tokens.push(token);
                        continue;
                    }
                } else if ch == '\r' {
                    // Handle CRLF sequences
                    if let Some(token) = self.read_newline() {
                        tokens.push(token);
                        continue;
                    }
                }
            }

            // Handle other whitespace (spaces and tabs)
            if let Some(ch) = self.peek() {
                if ch == ' ' || ch == '\t' {
                    self.advance();
                    continue;
                }
            }

            if self.is_at_end() {
                break;
            }

            if let Some(token) = self.read_annotation_marker() {
                tokens.push(token);
            } else if let Some(token) = self.read_text() {
                tokens.push(token);
            } else if let Some(token) = self.read_identifier() {
                tokens.push(token);
            } else {
                // Skip unrecognized character for now
                if let Some(_ch) = self.peek() {
                    self.advance();
                }
            }
        }

        // Add EOF token
        tokens.push(Token::Eof {
            span: SourceSpan {
                start: self.current_position(),
                end: self.current_position(),
            },
        });

        tokens
    }

    /// Read a text token (alphanumeric and underscore characters)
    fn read_text(&mut self) -> Option<Token> {
        let start_pos = self.current_position();
        let mut content = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                content.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if content.is_empty() {
            None
        } else {
            Some(Token::Text {
                content,
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            })
        }
    }

    /// Read a sequence marker token (list markers like "1. ", "a) ", "- ")
    fn read_sequence_marker(&mut self) -> Option<Token> {
        let start_pos = self.current_position();
        let saved_position = self.position;
        let saved_row = self.row;
        let saved_column = self.column;

        // Try to match sequence marker patterns

        // 1. Plain dash marker: "- "
        if self.peek() == Some('-') {
            self.advance();
            if self.peek() == Some(' ') {
                self.advance();
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
                self.position = saved_position;
                self.row = saved_row;
                self.column = saved_column;
                return None;
            }
        }

        // 2. Numbered markers: "1. ", "42. "
        let mut number_str = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if !number_str.is_empty() {
            if self.peek() == Some('.') {
                self.advance();
                if self.peek() == Some(' ') {
                    self.advance();
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
            self.position = saved_position;
            self.row = saved_row;
            self.column = saved_column;
            return None;
        }

        // 3. Alphabetical markers: "a. ", "b) ", "A. ", "Z) "
        if let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                let letter = ch;
                self.advance();

                // Check for . or )
                if let Some(punct) = self.peek() {
                    if punct == '.' || punct == ')' {
                        self.advance();
                        if self.peek() == Some(' ') {
                            self.advance();
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
        self.position = saved_position;
        self.row = saved_row;
        self.column = saved_column;

        let roman_patterns = [
            "iii", "ii", "i", "iv", "v", "vi", "vii", "viii", "ix", "x", "III", "II", "I", "IV",
            "V", "VI", "VII", "VIII", "IX", "X",
        ];

        for pattern in &roman_patterns {
            if self.matches_string(pattern) {
                // Advance past the roman numeral
                for _ in 0..pattern.len() {
                    self.advance();
                }

                // Check for . or )
                if let Some(punct) = self.peek() {
                    if punct == '.' || punct == ')' {
                        self.advance();
                        if self.peek() == Some(' ') {
                            self.advance();
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
                self.position = saved_position;
                self.row = saved_row;
                self.column = saved_column;
            }
        }

        None
    }

    /// Read an annotation marker token (::)
    fn read_annotation_marker(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        // Check for "::"
        if self.peek() == Some(':') {
            let saved_position = self.position;
            let saved_row = self.row;
            let saved_column = self.column;

            self.advance(); // First ':'

            if self.peek() == Some(':') {
                self.advance(); // Second ':'

                // Check that this is not part of a longer sequence like ":::"
                // Annotation markers should be exactly "::"
                if let Some(next_ch) = self.peek() {
                    if next_ch == ':' {
                        // This is ":::" or longer, not a valid annotation marker
                        self.position = saved_position;
                        self.row = saved_row;
                        self.column = saved_column;
                        return None;
                    }
                }

                return Some(Token::AnnotationMarker {
                    content: "::".to_string(),
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
            } else {
                // Not an annotation marker, backtrack
                self.position = saved_position;
                self.row = saved_row;
                self.column = saved_column;
            }
        }

        None
    }

    /// Read an identifier token (alphanumeric starting with letter or underscore)
    fn read_identifier(&mut self) -> Option<Token> {
        let start_pos = self.current_position();
        let mut content = String::new();

        // Must start with letter or underscore
        if let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                content.push(ch);
                self.advance();
            } else {
                return None;
            }
        } else {
            return None;
        }

        // Continue with alphanumeric or underscore
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                content.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Some(Token::Identifier {
            content,
            span: SourceSpan {
                start: start_pos,
                end: self.current_position(),
            },
        })
    }

    /// Check if the current position matches a given string
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

    /// Get the current position in the input
    fn current_position(&self) -> Position {
        Position {
            row: self.row,
            column: self.column,
        }
    }

    /// Peek at the current character without advancing
    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    /// Advance to the next character and update position tracking
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

    /// Read a newline token (\n or \r\n)
    fn read_newline(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if let Some(ch) = self.peek() {
            if ch == '\r' {
                // Handle CRLF sequence
                self.advance(); // Consume \r
                if self.peek() == Some('\n') {
                    self.advance(); // Consume \n
                    return Some(Token::Newline {
                        span: SourceSpan {
                            start: start_pos,
                            end: self.current_position(),
                        },
                    });
                } else {
                    // Just \r - treat as newline
                    return Some(Token::Newline {
                        span: SourceSpan {
                            start: start_pos,
                            end: self.current_position(),
                        },
                    });
                }
            } else if ch == '\n' {
                // Handle LF
                self.advance(); // Consume \n
                return Some(Token::Newline {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
            }
        }

        None
    }

    /// Check if we're at the end of input
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}
