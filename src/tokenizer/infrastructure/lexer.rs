//! TXXT Lexer - Character-precise tokenization for new AST
//!
//! Converts TXXT source text into Token enum variants with precise SourceSpan
//! positioning for language server support.

use crate::ast::tokens::{Position, SourceSpan, Token};
use crate::tokenizer::indentation::IndentationTracker;
use crate::tokenizer::infrastructure::markers::{
    sequence::read_sequence_marker,
    txxt_marker::{
        integrate_annotation_parameters, integrate_definition_parameters, read_annotation_marker,
        read_definition_marker,
    },
};
use crate::tokenizer::inline::read_inline_delimiter;
use crate::tokenizer::inline::references::{
    citations::read_citation_ref, footnote_ref::read_footnote_ref, page_ref::read_page_ref,
    session_ref::read_session_ref,
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
    pub(crate) indent_tracker: IndentationTracker,
}

impl Lexer {
    /// Create a new lexer for the given input text
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            row: 0,
            column: 0,
            indent_tracker: IndentationTracker::new(),
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

            // Process indentation at column 0 (start of line), but skip verbatim content lines
            if self.column == 0 {
                // Check if this line is part of verbatim content that should be skipped
                let current_line = self.row + 1; // Convert to 1-based line numbers for verbatim scanner
                let is_verbatim_content =
                    verbatim_scanner.is_verbatim_content(current_line, &verbatim_blocks);

                if !is_verbatim_content {
                    // Get the current line for indentation processing
                    if let Some(line) = self.get_current_line() {
                        // Update indentation tracker position
                        self.indent_tracker.set_position(self.current_position());

                        // Process line indentation and emit Indent/Dedent tokens
                        let indent_tokens = self.indent_tracker.process_line_indentation(&line);
                        tokens.extend(indent_tokens);
                    }
                }
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
                    if let Some(token) = self.read_whitespace() {
                        tokens.push(token);
                        continue;
                    }
                }
            }

            if self.is_at_end() {
                break;
            }

            if let Some(token) = read_definition_marker(&mut *self) {
                tokens.push(token);
            } else if let Some(token) = read_annotation_marker(&mut *self) {
                tokens.push(token);
            // TODO: Update these to work with atomic tokens from parser level
            } else if let Some(token) = read_citation_ref(self) {
                tokens.push(token);
            } else if let Some(token) = read_page_ref(self) {
                tokens.push(token);
            } else if let Some(token) = read_session_ref(self) {
                tokens.push(token);
            } else if let Some(token) = read_footnote_ref(self) {
                tokens.push(token);
            } else if let Some(token) =
                crate::tokenizer::inline::references::ReferenceLexer::read_ref_marker(self)
            {
                tokens.push(token);
            } else if let Some(token) = self.read_left_bracket() {
                tokens.push(token);
            } else if let Some(token) = self.read_right_bracket() {
                tokens.push(token);
            } else if let Some(token) = self.read_at_sign() {
                tokens.push(token);
            } else if let Some(token) = self.read_left_paren() {
                tokens.push(token);
            } else if let Some(token) = self.read_right_paren() {
                tokens.push(token);
            } else if let Some(token) = self.read_colon() {
                tokens.push(token);
            } else if let Some(token) = read_inline_delimiter(self) {
                tokens.push(token);
            } else if let Some(token) = self.read_dash() {
                tokens.push(token);
            } else if let Some(token) = self.read_period() {
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
        // Use original integration for now until we fix the new one
        tokens = integrate_annotation_parameters(tokens, self);
        tokens = integrate_definition_parameters(tokens, self);

        // Finalize indentation processing (emit remaining dedents)
        let final_indent_tokens = self.indent_tracker.finalize();
        tokens.extend(final_indent_tokens);

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
            } else if ch == '.' {
                // Include period if it's part of decimal numbers (e.g., "2.0")
                let next_pos = self.position + 1;
                if let Some(&next_ch) = self.input.get(next_pos) {
                    if next_ch.is_ascii_digit()
                        && !content.is_empty()
                        && content.chars().last().unwrap().is_ascii_digit()
                    {
                        content.push(ch);
                        self.advance();
                    } else {
                        // Period is structural, not part of text
                        break;
                    }
                } else {
                    // Period at end of input, stop here
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

    /// Read a period token (standalone .)
    /// Only tokenizes periods that are structural markers, not those within text content
    fn read_period(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some('.') {
            // Check context to determine if this should be a standalone period token
            // Periods within numeric text (like "2.0") should not be tokenized separately

            // Look at previous character if available
            let prev_is_digit = if self.position > 0 {
                self.input
                    .get(self.position - 1)
                    .is_some_and(|c| c.is_ascii_digit())
            } else {
                false
            };

            // Look at next character
            let next_is_digit = self
                .input
                .get(self.position + 1)
                .is_some_and(|c| c.is_ascii_digit());

            // If surrounded by digits, this is part of a decimal number - don't tokenize
            if prev_is_digit && next_is_digit {
                return None;
            }

            self.advance();
            return Some(Token::Period {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a left bracket token ([)
    fn read_left_bracket(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some('[') {
            self.advance();
            return Some(Token::LeftBracket {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a right bracket token (])
    fn read_right_bracket(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some(']') {
            self.advance();
            return Some(Token::RightBracket {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read an at-sign token (@)
    fn read_at_sign(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some('@') {
            self.advance();
            return Some(Token::AtSign {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a left parenthesis token (()
    fn read_left_paren(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some('(') {
            self.advance();
            return Some(Token::LeftParen {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a right parenthesis token ())
    fn read_right_paren(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some(')') {
            self.advance();
            return Some(Token::RightParen {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a colon token (:)
    /// Only tokenizes structural colons, not those used in parameter syntax
    fn read_colon(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        if self.peek() == Some(':') {
            // Check if this is part of a double colon (::)
            let next_pos = self.position + 1;
            if let Some(&next_ch) = self.input.get(next_pos) {
                if next_ch == ':' {
                    // This is part of a double colon, don't tokenize as single colon
                    return None;
                }
            }

            // Check if this colon is likely part of parameter syntax
            // Look backwards for alphanumeric content (term before colon)
            if self.position > 0 {
                if let Some(&prev_ch) = self.input.get(self.position - 1) {
                    if prev_ch.is_alphanumeric() {
                        // Look ahead for parameter-like content (key=value)
                        let mut lookahead_pos = next_pos;
                        while lookahead_pos < self.input.len() {
                            if let Some(&ch) = self.input.get(lookahead_pos) {
                                if ch == '=' {
                                    // Found = after colon following alphanumeric, likely parameter
                                    return None;
                                } else if ch == ' '
                                    || ch == '\t'
                                    || ch.is_alphanumeric()
                                    || ch == '_'
                                {
                                    lookahead_pos += 1;
                                    continue;
                                } else {
                                    // Hit non-parameter character, break
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            self.advance();
            return Some(Token::Colon {
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

    /// Read whitespace token (spaces and tabs, but not newlines)
    fn read_whitespace(&mut self) -> Option<Token> {
        let start_pos = self.current_position();
        let mut content = String::new();

        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                content.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if content.is_empty() {
            None
        } else {
            Some(Token::Whitespace {
                content,
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            })
        }
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

    /// Get the current line as a string (for indentation processing)
    ///
    /// Returns the entire line that the current position is on, starting from
    /// the beginning of the line to the end of the line (or end of input).
    fn get_current_line(&self) -> Option<String> {
        if self.is_at_end() {
            return None;
        }

        // Find the start of the current line
        let mut line_start = self.position;
        while line_start > 0 && self.input[line_start - 1] != '\n' {
            line_start -= 1;
        }

        // Find the end of the current line
        let mut line_end = self.position;
        while line_end < self.input.len()
            && self.input[line_end] != '\n'
            && self.input[line_end] != '\r'
        {
            line_end += 1;
        }

        // Extract the line
        let line: String = self.input[line_start..line_end].iter().collect();
        Some(line)
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

// Trait implementations for reference lexing
use crate::tokenizer::inline::references::{CitationRefLexer, PageRefLexer};

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
        self.advance()
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

impl PageRefLexer for Lexer {
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
        self.advance()
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

impl crate::tokenizer::inline::references::session_ref::SessionRefLexer for Lexer {
    fn current_position(&self) -> crate::ast::tokens::Position {
        crate::ast::tokens::Position {
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

impl crate::tokenizer::inline::references::ReferenceLexer for Lexer {
    fn current_position(&self) -> Position {
        Position {
            row: self.row,
            column: self.column,
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.advance()
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

    fn ref_classifier(&self) -> &crate::ast::reference_types::ReferenceClassifier {
        // Create a static classifier instance
        static CLASSIFIER: std::sync::OnceLock<crate::ast::reference_types::ReferenceClassifier> =
            std::sync::OnceLock::new();
        CLASSIFIER.get_or_init(crate::ast::reference_types::ReferenceClassifier::new)
    }

    fn backtrack(&mut self, position: usize, row: usize, column: usize) {
        self.position = position;
        self.row = row;
        self.column = column;
    }
}

impl crate::tokenizer::inline::references::footnote_ref::FootnoteRefLexer for Lexer {
    fn current_position(&self) -> crate::ast::tokens::Position {
        crate::ast::tokens::Position {
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

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
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
