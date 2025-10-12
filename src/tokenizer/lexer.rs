//! TXXT Lexer - Character-precise tokenization for new AST
//!
//! Converts TXXT source text into Token enum variants with precise SourceSpan
//! positioning for language server support.

use crate::ast::reference_types::ReferenceClassifier;
use crate::ast::tokens::{Position, SourceSpan, Token};
use crate::tokenizer::inline::{
    parse_parameters, read_inline_delimiter, InlineDelimiterLexer, ParameterLexer,
};
use crate::tokenizer::markers::{
    detect_colon_pattern, integrate_annotation_parameters, integrate_definition_parameters,
    is_start_of_annotation_pattern, read_annotation_marker, read_definition_marker,
    read_sequence_marker, ColonPattern, SequenceMarkerLexer, TxxtMarkerLexer,
};
use crate::tokenizer::patterns::get_current_line;
use crate::tokenizer::verbatim_scanner::{VerbatimBlock, VerbatimScanner};
use regex::Regex;

/// Saved lexer state for backtracking
#[derive(Debug, Clone)]
pub struct LexerState {
    position: usize,
    row: usize,
    column: usize,
}

/// Main tokenizer that produces new AST Token enum variants
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    row: usize,
    column: usize,
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
            if let Some(verbatim_tokens) = self.read_verbatim_block(&verbatim_blocks) {
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
            } else if let Some(token) = self.read_ref_marker() {
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

    /// Read reference markers ([target], [@citation], [#section], [1])
    fn read_ref_marker(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        // Must start with [
        if self.peek() != Some('[') {
            return None;
        }

        let saved_position = self.position;
        let saved_row = self.row;
        let saved_column = self.column;

        self.advance(); // Consume [

        let mut content = String::new();
        let mut found_closing = false;

        // Read content until ] or end of line
        while let Some(ch) = self.peek() {
            if ch == ']' {
                self.advance(); // Consume ]
                found_closing = true;
                break;
            } else if ch == '\n' || ch == '\r' {
                // Reference markers cannot span lines
                break;
            } else {
                content.push(ch);
                self.advance();
            }
        }

        if !found_closing || content.is_empty() {
            // Not a valid reference marker, backtrack
            self.position = saved_position;
            self.row = saved_row;
            self.column = saved_column;
            return None;
        }

        // Validate content patterns
        if self.is_valid_ref_content(&content) {
            Some(Token::RefMarker {
                content,
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            })
        } else {
            // Invalid reference content, backtrack
            self.position = saved_position;
            self.row = saved_row;
            self.column = saved_column;
            None
        }
    }

    /// Check if reference content is valid (basic alphanumeric validation only)
    fn is_valid_ref_content(&self, content: &str) -> bool {
        // Only basic validation - at least one alphanumeric character
        // Detailed type classification happens during parsing phase
        self.ref_classifier.is_valid_reference_content(content)
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

    /// Read verbatim block if current position matches a verbatim block start
    fn read_verbatim_block(&mut self, verbatim_blocks: &[VerbatimBlock]) -> Option<Vec<Token>> {
        let current_line = self.row;
        let current_char_pos = self.get_absolute_position();

        // Find a verbatim block that starts at this position
        for block in verbatim_blocks {
            if self.is_at_verbatim_block_start(block, current_line, current_char_pos) {
                return Some(self.tokenize_verbatim_block(block));
            }
        }

        None
    }

    /// Check if current position is at the start of the given verbatim block
    fn is_at_verbatim_block_start(
        &self,
        block: &VerbatimBlock,
        current_line: usize,
        _current_char_pos: usize,
    ) -> bool {
        // Check if we're at the correct line for block start (1-based to 0-based conversion)
        (block.block_start - 1) == current_line && self.is_at_line_start_for_verbatim(block)
    }

    /// Check if we're at the start of a line that should be part of a verbatim block
    fn is_at_line_start_for_verbatim(&self, _block: &VerbatimBlock) -> bool {
        // For now, just check if we're at the start of a line
        self.column == 0
    }

    /// Tokenize a verbatim block into VerbatimTitle and VerbatimContent tokens
    fn tokenize_verbatim_block(&mut self, block: &VerbatimBlock) -> Vec<Token> {
        let mut tokens = Vec::new();

        // Create VerbatimTitle token for the title line
        let title_start_pos = self.current_position();

        // Advance through the title line to get its content (excluding the trailing colon)
        let mut title_content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            if ch == ':' {
                // Don't include the colon - it's a structural marker, not content
                self.advance();
                break;
            }
            title_content.push(ch);
            self.advance();
        }

        // Advance past the newline
        if let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                self.advance();
                if ch == '\r' && self.peek() == Some('\n') {
                    self.advance(); // Handle CRLF
                }
            }
        }

        tokens.push(Token::VerbatimTitle {
            content: title_content,
            span: SourceSpan {
                start: title_start_pos,
                end: self.current_position(),
            },
        });

        // Create VerbatimContent token for the block content
        let content_start_pos = self.current_position();
        let mut content = String::new();

        // Advance through all content lines until terminator
        let mut current_line = self.row;
        while current_line < (block.block_end - 1) {
            // Convert 1-based to 0-based
            // Read the entire line
            while let Some(ch) = self.peek() {
                if ch == '\n' || ch == '\r' {
                    content.push(ch);
                    self.advance();
                    if ch == '\r' && self.peek() == Some('\n') {
                        content.push('\n');
                        self.advance(); // Handle CRLF
                    }
                    break;
                } else {
                    content.push(ch);
                    self.advance();
                }
            }
            current_line = self.row;
        }

        // Don't include the terminator line in content
        // Remove trailing newline if it exists
        if content.ends_with('\n') {
            content.pop();
            if content.ends_with('\r') {
                content.pop();
            }
        }

        if !content.is_empty() {
            tokens.push(Token::VerbatimContent {
                content,
                span: SourceSpan {
                    start: content_start_pos,
                    end: self.current_position(),
                },
            });
        }

        // Parse the terminator line to extract label and parameters
        let terminator_start_pos = self.current_position();
        let mut terminator_content = String::new();

        // Read the entire terminator line content
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            terminator_content.push(ch);
            self.advance();
        }

        // Extract just the label+params portion (without :: prefix)
        if !terminator_content.trim().is_empty() {
            // Use the same regex pattern as the verbatim scanner
            let verbatim_end_re =
                Regex::new(r"^\s*::\s+([a-zA-Z_][a-zA-Z0-9._-]*(?::[^:\s].*)?)\s*$").unwrap();

            if let Some(captures) = verbatim_end_re.captures(&terminator_content) {
                if let Some(label_and_params) = captures.get(1) {
                    let label_and_params_str = label_and_params.as_str();

                    // Split label from parameters at the first colon
                    if let Some(colon_pos) = label_and_params_str.find(':') {
                        // There are parameters - split them
                        let label = &label_and_params_str[..colon_pos];
                        let params_str = &label_and_params_str[colon_pos + 1..];

                        // Add the clean verbatim label
                        tokens.push(Token::VerbatimLabel {
                            content: label.to_string(),
                            span: SourceSpan {
                                start: terminator_start_pos,
                                end: self.current_position(),
                            },
                        });

                        // Parse and add individual parameter tokens
                        let mut param_tokens = parse_parameters(self, params_str);
                        tokens.append(&mut param_tokens);
                    } else {
                        // No parameters - just the label
                        tokens.push(Token::VerbatimLabel {
                            content: label_and_params_str.to_string(),
                            span: SourceSpan {
                                start: terminator_start_pos,
                                end: self.current_position(),
                            },
                        });
                    }
                }
            } else {
                // Fallback: if regex doesn't match, use the full content (shouldn't happen)
                tokens.push(Token::VerbatimLabel {
                    content: terminator_content,
                    span: SourceSpan {
                        start: terminator_start_pos,
                        end: self.current_position(),
                    },
                });
            }
        }

        // Advance past the newline at end of terminator
        if let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                self.advance();
                if ch == '\r' && self.peek() == Some('\n') {
                    self.advance(); // Handle CRLF
                }
            }
        }

        tokens
    }

    /// Get absolute character position in input
    fn get_absolute_position(&self) -> usize {
        self.position
    }
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

impl ParameterLexer for Lexer {
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

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn get_input(&self) -> &[char] {
        &self.input
    }
}

impl TxxtMarkerLexer for Lexer {
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

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn get_current_line(&self) -> String {
        get_current_line(&self.input, self.position, self.row, self.column)
    }

    fn detect_colon_pattern(&self) -> ColonPattern {
        detect_colon_pattern(self)
    }

    fn is_start_of_annotation_pattern(&self, start_pos: Position) -> bool {
        is_start_of_annotation_pattern(self, start_pos)
    }

    fn annotation_pattern(&self) -> &Regex {
        // Create a static regex for annotation pattern
        static ANNOTATION_PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        ANNOTATION_PATTERN.get_or_init(|| Regex::new(r"::\s*\w+.*?\s*::").unwrap())
    }

    fn definition_pattern(&self) -> &Regex {
        // Create a static regex for definition pattern
        static DEFINITION_PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        DEFINITION_PATTERN.get_or_init(|| Regex::new(r"\w+.*?::\s*$").unwrap())
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
