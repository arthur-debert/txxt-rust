//! TXXT Lexer - Character-precise tokenization for new AST
//!
//! Converts TXXT source text into Token enum variants with precise SourceSpan
//! positioning for language server support.

use crate::ast::reference_types::ReferenceClassifier;
use crate::ast::tokens::{Position, SourceSpan, Token};
use crate::tokenizer::inline::{
    read_citation_ref, read_inline_delimiter, read_math_span, CitationRefLexer, MathSpanLexer,
};
use crate::tokenizer::markers::{
    integrate_annotation_parameters, integrate_definition_parameters, read_annotation_marker,
    read_definition_marker, read_sequence_marker, ReferenceLexer,
};
use crate::tokenizer::verbatim_scanner::{VerbatimLexer, VerbatimScanner};

/// Saved lexer state for backtracking
#[derive(Debug, Clone)]
pub struct LexerState {
    pub(crate) position: usize,
    pub(crate) row: usize,
    pub(crate) column: usize,
}

/// Main tokenizer that produces new AST Token enum variants
pub struct Lexer {
    pub(crate) input: Vec<char>,
    pub(crate) position: usize,
    pub(crate) row: usize,
    pub(crate) column: usize,
    // Reference classifier for basic validation
    ref_classifier: ReferenceClassifier,
}

impl Lexer {
    /// Create a new lexer for the given input text
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            row: 0,
            column: 0,
            // Reference classifier for basic validation only
            ref_classifier: ReferenceClassifier::new(),
        }
    }

    /// Tokenize the input text into Token enum variants
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        // First, pre-scan for verbatim blocks
        let input_text: String = self.input.iter().collect();
        let verbatim_scanner = VerbatimScanner::new();
        let verbatim_blocks = verbatim_scanner.scan(&input_text);

        while !self.is_at_end() {
            // Check if we're at the start of a verbatim block
            if let Some(verbatim_tokens) =
                VerbatimLexer::read_verbatim_block(self, &verbatim_blocks)
            {
                tokens.extend(verbatim_tokens);
                continue;
            }

            // Try to read sequence marker only at column 0 (start of line)
            if self.column == 0 {
                if let Some(token) = read_sequence_marker(self) {
                    tokens.push(token);
                    continue;
                }
            }

            // Try to read blank lines when at column 0 (start of line)
            if self.column == 0 {
                if let Some(token) = self.read_blankline() {
                    // Check if the last token was also a BlankLine and merge them
                    if let Some(Token::BlankLine { span: last_span }) = tokens.last_mut() {
                        // Extend the span of the existing BlankLine to include this new one
                        last_span.end = token.span().end;
                    } else {
                        // This is the first BlankLine or follows a different token type
                        tokens.push(token);
                    }
                    continue;
                }
            }

            // Handle newlines (they're significant tokens)
            if let Some(ch) = self.peek() {
                if ch == '\n' || ch == '\r' {
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

            if let Some(token) = read_definition_marker(&mut *self) {
                tokens.push(token);
            } else if let Some(token) = read_annotation_marker(&mut *self) {
                tokens.push(token);
            } else if let Some(token) = read_math_span(self) {
                tokens.push(token);
            } else if let Some(token) = read_citation_ref(self) {
                tokens.push(token);
            } else if let Some(token) = ReferenceLexer::read_ref_marker(self) {
                tokens.push(token);
            } else if let Some(token) = read_inline_delimiter(self) {
                tokens.push(token);
            } else if let Some(token) = self.read_dash() {
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

        // Simple parameter integration - just detect and split label:params patterns
        tokens = integrate_annotation_parameters(tokens, self);
        tokens = integrate_definition_parameters(tokens, self);

        // Add EOF token
        tokens.push(Token::Eof {
            span: SourceSpan {
                start: self.current_position(),
                end: self.current_position(),
            },
        });

        tokens
    }

    /// Read a text token (alphanumeric characters and underscores that are part of words)
    fn read_text(&mut self) -> Option<Token> {
        let start_pos = self.current_position();
        let mut content = String::new();

        // Don't start text with delimiter characters or dash
        if let Some(ch) = self.peek() {
            if ch == '*' || ch == '_' || ch == '`' || ch == '#' || ch == '-' {
                return None;
            }
        }

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() {
                content.push(ch);
                self.advance();
            } else if ch == '_' {
                // Only include underscore if it's followed by alphanumeric (not delimiter)
                let next_pos = self.position + 1;
                if let Some(&next_ch) = self.input.get(next_pos) {
                    if next_ch.is_alphanumeric() {
                        content.push(ch);
                        self.advance();
                    } else {
                        // Next char is not alphanumeric, stop here to let delimiter handler take it
                        break;
                    }
                } else {
                    // At end of input, stop here
                    break;
                }
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

    /// Read a dash token (standalone -)
    fn read_dash(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some('-') {
            // Check if this is part of a sequence marker (already handled earlier)
            // We only want standalone dashes, not "- " sequence markers
            let next_pos = self.position + 1;
            if let Some(&next_ch) = self.input.get(next_pos) {
                if next_ch == ' ' {
                    // This is a sequence marker, not a standalone dash
                    return None;
                }
            }

            self.advance();
            return Some(Token::Dash {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read an identifier token (alphanumeric starting with letter or underscore)
    fn read_identifier(&mut self) -> Option<Token> {
        let start_pos = self.current_position();
        let mut content = String::new();

        // Must start with letter or underscore (but only if underscore is part of a longer identifier)
        if let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                content.push(ch);
                self.advance();
            } else if ch == '_' {
                // Only start with underscore if followed by alphanumeric (not standalone delimiter)
                let next_pos = self.position + 1;
                if let Some(&next_ch) = self.input.get(next_pos) {
                    if next_ch.is_ascii_alphabetic() || next_ch.is_ascii_digit() || next_ch == '_' {
                        content.push(ch);
                        self.advance();
                    } else {
                        return None; // Standalone underscore should be handled as delimiter
                    }
                } else {
                    return None; // Underscore at end of input should be delimiter
                }
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

    /// Get the current position in the input (internal method)
    fn current_position(&self) -> Position {
        Position {
            row: self.row,
            column: self.column,
        }
    }

    /// Advance to the next character and update position tracking (internal method)
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
                }

                return Some(Token::Newline {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
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

    /// Read a blank line token (line containing only whitespace, NOT including line break)  
    fn read_blankline(&mut self) -> Option<Token> {
        // Only try to read blank lines when we're at column 0 (start of line)
        if self.column != 0 {
            return None;
        }

        // Don't read blank lines at the very start of input - that should be a newline
        if self.row == 0 {
            return None;
        }

        let start_pos = self.current_position();
        let saved_position = self.position;
        let saved_row = self.row;
        let saved_column = self.column;

        // Collect any whitespace on this line
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }

        // Check if this line ends with a newline (making it a blank line)
        // Consume the newline as part of the BlankLine token
        if let Some(ch) = self.peek() {
            if ch == '\n' {
                // This is a blank line - consume the newline
                self.advance(); // Consume \n
                return Some(Token::BlankLine {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
            } else if ch == '\r' {
                // Handle CRLF
                self.advance(); // Consume \r
                if self.peek() == Some('\n') {
                    self.advance(); // Consume \n
                }
                return Some(Token::BlankLine {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
            }
        }

        // Also handle end of file after whitespace-only content
        if self.is_at_end() && self.position > saved_position {
            return Some(Token::BlankLine {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        // Not a blank line, backtrack
        self.position = saved_position;
        self.row = saved_row;
        self.column = saved_column;
        None
    }

    /// Check if we're at the end of input
    pub fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Peek at the current character without advancing
    pub fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    // Debug methods (for testing)
    pub fn position(&self) -> usize {
        self.position
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn test_read_newline(&mut self) -> Option<Token> {
        self.read_newline()
    }

    pub fn test_advance(&mut self) -> Option<char> {
        self.advance()
    }
}

impl VerbatimLexer for Lexer {
    fn row(&self) -> usize {
        self.row
    }

    fn column(&self) -> usize {
        self.column
    }

    fn get_absolute_position(&self) -> usize {
        self.position
    }
}

impl ReferenceLexer for Lexer {
    fn current_position(&self) -> Position {
        Position {
            row: self.row,
            column: self.column,
        }
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

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
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

    fn input(&self) -> &[char] {
        &self.input
    }

    fn ref_classifier(&self) -> &ReferenceClassifier {
        &self.ref_classifier
    }

    fn backtrack(&mut self, position: usize, row: usize, column: usize) {
        self.position = position;
        self.row = row;
        self.column = column;
    }
}

impl MathSpanLexer for Lexer {
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

impl CitationRefLexer for Lexer {
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
