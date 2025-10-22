//! TXXT Lexer - Character-precise tokenization for new AST
//!
//! Converts TXXT source text into Token enum variants with precise SourceSpan
//! positioning for language server support.

use crate::cst::{Position, ScannerToken, SourceSpan};
use crate::syntax::core::indentation::IndentationTracker;
use crate::syntax::elements::components::sequence::read_sequence_marker;
use crate::syntax::elements::formatting::read_inline_delimiter;
use crate::syntax::elements::references::{
    citations::read_citation_ref, footnote_ref::read_footnote_ref, page_ref::read_page_ref,
    session_ref::read_session_ref,
};
use crate::syntax::verbatim_scanning::{VerbatimLexer, VerbatimScanner};

/// Check if a character is a special delimiter that should terminate text tokens
fn is_special_delimiter(ch: char) -> bool {
    matches!(
        ch,
        '*' | '_' | '`' | '#' | '-' | '[' | ']' | ':' | '(' | ')' | '=' | ',' | '.' | '\\'
    )
}

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
    pub fn tokenize(&mut self) -> Vec<ScannerToken> {
        let mut tokens = Vec::new();

        // First, pre-scan for verbatim block boundaries (NEW - Issue #132)
        let input_text: String = self.input.iter().collect();
        let verbatim_scanner = VerbatimScanner::new();
        let verbatim_boundaries = verbatim_scanner.scan_boundaries(&input_text);

        while !self.is_at_end() {
            // NEW: Check if we're at a verbatim boundary (title or terminator line)
            let current_line = self.row + 1; // 1-based line numbers
            if let Some(boundary_token) =
                self.try_read_verbatim_boundary(current_line, &verbatim_boundaries)
            {
                tokens.push(boundary_token);
                continue;
            }

            // NEW: Check if we're on a verbatim content line
            if let Some(content_token) =
                self.try_read_verbatim_content_line(current_line, &verbatim_boundaries)
            {
                tokens.push(content_token);
                continue;
            }

            // Process indentation at column 0 (start of line), but skip verbatim content lines
            if self.column == 0 {
                // Check if this line is part of verbatim content that should be skipped
                let is_verbatim_line =
                    self.is_inside_verbatim_block(current_line, &verbatim_boundaries);

                if !is_verbatim_line {
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

            // Try to read sequence marker at start of line or after indentation
            if self.column == 0 || self.is_at_line_start_after_indent(&tokens) {
                if let Some(token) = read_sequence_marker(self) {
                    tokens.push(token);
                    continue;
                }
            }

            // Try to read blank lines when at column 0 (start of line)
            if self.column == 0 {
                if let Some(token) = self.read_blankline() {
                    // Don't merge blank lines - each one should be preserved separately
                    // to maintain exact whitespace content
                    tokens.push(token);
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

            if let Some(token) = self.read_txxt_marker() {
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
                crate::syntax::elements::references::ReferenceLexer::read_ref_marker(self)
            {
                tokens.push(token);
            } else if let Some(token) = self.read_left_bracket() {
                tokens.push(token);
            } else if let Some(token) = self.read_right_bracket() {
                tokens.push(token);
            } else if let Some(token) = self.read_left_paren() {
                tokens.push(token);
            } else if let Some(token) = self.read_right_paren() {
                tokens.push(token);
            } else if let Some(token) = self.read_colon() {
                tokens.push(token);
            } else if let Some(token) = self.read_equals() {
                tokens.push(token);
            } else if let Some(token) = self.read_comma() {
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

        // Parameters are now handled at the semantic analysis phase
        // The scanner emits basic tokens (Text, Colon, Equals, etc.) and
        // semantic analysis uses scan_parameter_string to parse parameter regions

        // Finalize indentation processing (emit remaining dedents)
        let final_indent_tokens = self.indent_tracker.finalize();
        tokens.extend(final_indent_tokens);

        // Add EOF token
        tokens.push(ScannerToken::Eof {
            span: SourceSpan {
                start: self.current_position(),
                end: self.current_position(),
            },
        });

        tokens
    }

    // ========== NEW VERBATIM BOUNDARY METHODS (Issue #132) ==========

    /// Check if we're at a verbatim boundary line (title or terminator) and emit appropriate token
    fn try_read_verbatim_boundary(
        &mut self,
        current_line: usize,
        boundaries: &[crate::syntax::verbatim_scanning::VerbatimBoundary],
    ) -> Option<ScannerToken> {
        // Must be at start of line
        if self.column != 0 {
            return None;
        }

        // Find boundary that matches current line
        for boundary in boundaries {
            // Check if this is the title line
            if boundary.title_line == current_line {
                return self.read_verbatim_block_start(boundary);
            }

            // Check if this is the terminator line
            if boundary.terminator_line == current_line {
                return self.read_verbatim_block_end(boundary);
            }
        }

        None
    }

    /// Read VerbatimBlockStart token for title line
    fn read_verbatim_block_start(
        &mut self,
        boundary: &crate::syntax::verbatim_scanning::VerbatimBoundary,
    ) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        // Advance through the entire title line
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                self.advance(); // Consume newline
                if ch == '\r' && self.peek() == Some('\n') {
                    self.advance(); // Handle CRLF
                }
                break;
            }
            self.advance();
        }

        let end_pos = self.current_position();

        Some(ScannerToken::VerbatimBlockStart {
            title: boundary.title.clone(),
            wall_type: boundary.wall_type.clone(),
            span: SourceSpan {
                start: start_pos,
                end: end_pos,
            },
        })
    }

    /// Read VerbatimBlockEnd token for terminator line
    fn read_verbatim_block_end(
        &mut self,
        boundary: &crate::syntax::verbatim_scanning::VerbatimBoundary,
    ) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        // Advance through the entire terminator line
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                self.advance(); // Consume newline
                if ch == '\r' && self.peek() == Some('\n') {
                    self.advance(); // Handle CRLF
                }
                break;
            }
            self.advance();
        }

        let end_pos = self.current_position();

        Some(ScannerToken::VerbatimBlockEnd {
            label_raw: boundary.label_raw.clone(),
            span: SourceSpan {
                start: start_pos,
                end: end_pos,
            },
        })
    }

    /// Read VerbatimContentLine token for content line
    fn try_read_verbatim_content_line(
        &mut self,
        current_line: usize,
        boundaries: &[crate::syntax::verbatim_scanning::VerbatimBoundary],
    ) -> Option<ScannerToken> {
        // Must be at start of line
        if self.column != 0 {
            return None;
        }

        // Check if this line is verbatim content
        for boundary in boundaries {
            if let (Some(start), Some(end)) = (boundary.content_start, boundary.content_end) {
                if current_line >= start && current_line <= end {
                    return self.read_verbatim_content_line(boundary);
                }
            }
        }

        None
    }

    /// Read a single verbatim content line
    fn read_verbatim_content_line(
        &mut self,
        _boundary: &crate::syntax::verbatim_scanning::VerbatimBoundary,
    ) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        // Capture leading indentation
        let mut indentation = String::new();
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                indentation.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Capture rest of line as content
        let mut content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                self.advance(); // Consume newline
                if ch == '\r' && self.peek() == Some('\n') {
                    self.advance(); // Handle CRLF
                }
                break;
            }
            content.push(ch);
            self.advance();
        }

        let end_pos = self.current_position();

        Some(ScannerToken::VerbatimContentLine {
            content,
            indentation,
            span: SourceSpan {
                start: start_pos,
                end: end_pos,
            },
        })
    }

    /// Check if current line is inside a verbatim block (title, content, or terminator)
    fn is_inside_verbatim_block(
        &self,
        current_line: usize,
        boundaries: &[crate::syntax::verbatim_scanning::VerbatimBoundary],
    ) -> bool {
        for boundary in boundaries {
            // Check if on title line
            if boundary.title_line == current_line {
                return true;
            }

            // Check if on terminator line
            if boundary.terminator_line == current_line {
                return true;
            }

            // Check if on content line
            if let (Some(start), Some(end)) = (boundary.content_start, boundary.content_end) {
                if current_line >= start && current_line <= end {
                    return true;
                }
            }
        }
        false
    }

    // ========== END NEW VERBATIM BOUNDARY METHODS ==========

    /// Read a text token (any non-whitespace, non-delimiter characters)
    fn read_text(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();
        let mut content = String::new();

        // Don't start text with delimiter characters or dash (unless it's a backslash)
        if let Some(ch) = self.peek() {
            if ch == '*' || ch == '_' || ch == '`' || ch == '#' || ch == '-' {
                return None;
            }
        }

        while let Some(ch) = self.peek() {
            if ch == '\\' {
                // Handle escape sequences
                let next_pos = self.position + 1;
                if let Some(&next_ch) = self.input.get(next_pos) {
                    // Check if next character is a special character that should be escaped
                    if next_ch == '*'
                        || next_ch == '_'
                        || next_ch == '`'
                        || next_ch == '#'
                        || next_ch == '-'
                        || next_ch == '\\'
                        || next_ch == '['
                        || next_ch == ']'
                    {
                        // Include both backslash and the escaped character
                        content.push(ch);
                        self.advance();
                        content.push(next_ch);
                        self.advance();
                    } else {
                        // Backslash not followed by special character, just include it
                        content.push(ch);
                        self.advance();
                    }
                } else {
                    // Backslash at end of input, include it
                    content.push(ch);
                    self.advance();
                }
            } else if (!ch.is_whitespace() && !is_special_delimiter(ch)) || ch == '^' {
                // Include any non-whitespace, non-delimiter character, plus caret
                content.push(ch);
                self.advance();
            } else if ch == '_' {
                // Only include underscore if it's followed by alphanumeric (not delimiter)
                let next_pos = self.position + 1;
                if let Some(&next_ch) = self.input.get(next_pos) {
                    if next_ch.is_alphanumeric()
                        || (!next_ch.is_whitespace() && !is_special_delimiter(next_ch))
                    {
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
                    if next_ch.is_numeric()
                        && !content.is_empty()
                        && content.chars().last().unwrap().is_numeric()
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
            Some(ScannerToken::Text {
                content,
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            })
        }
    }

    /// Read a dash token (standalone -)
    fn read_dash(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some('-') {
            self.advance();
            return Some(ScannerToken::Dash {
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
    fn read_period(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some('.') {
            // Check context to determine if this should be a standalone period token
            // Periods within numeric text (like "2.0") should not be tokenized separately

            // Look at previous character if available
            let prev_is_digit = if self.position > 0 {
                self.input
                    .get(self.position - 1)
                    .is_some_and(|c| c.is_numeric())
            } else {
                false
            };

            // Look at next character
            let next_is_digit = self
                .input
                .get(self.position + 1)
                .is_some_and(|c| c.is_numeric());

            // If surrounded by digits, this is part of a decimal number - don't tokenize
            if prev_is_digit && next_is_digit {
                return None;
            }

            self.advance();
            return Some(ScannerToken::Period {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a left bracket token ([)
    fn read_left_bracket(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some('[') {
            self.advance();
            return Some(ScannerToken::LeftBracket {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a right bracket token (])
    fn read_right_bracket(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some(']') {
            self.advance();
            return Some(ScannerToken::RightBracket {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a left parenthesis token (()
    fn read_left_paren(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some('(') {
            self.advance();
            return Some(ScannerToken::LeftParen {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a right parenthesis token ())
    fn read_right_paren(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some(')') {
            self.advance();
            return Some(ScannerToken::RightParen {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a colon token (:)
    fn read_colon(&mut self) -> Option<ScannerToken> {
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

            self.advance();
            return Some(ScannerToken::Colon {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read an equals token (=)
    fn read_equals(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some('=') {
            self.advance();
            return Some(ScannerToken::Equals {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a comma token (,)
    fn read_comma(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some(',') {
            self.advance();
            return Some(ScannerToken::Comma {
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            });
        }

        None
    }

    /// Read a TxxtMarker token (::)
    fn read_txxt_marker(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if self.peek() == Some(':') {
            let saved_state = self.save_state();
            self.advance(); // First ':'

            if self.peek() == Some(':') {
                self.advance(); // Second ':'

                // Check that this is not part of a longer sequence like ":::"
                if let Some(next_ch) = self.peek() {
                    if next_ch == ':' {
                        // This is ":::" or longer, not a valid TxxtMarker
                        self.restore_state(saved_state);
                        return None;
                    }
                }

                return Some(ScannerToken::TxxtMarker {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
            } else {
                // Not a TxxtMarker, backtrack
                self.restore_state(saved_state);
            }
        }

        None
    }

    /// Save current lexer state for backtracking
    fn save_state(&self) -> LexerState {
        LexerState {
            position: self.position,
            row: self.row,
            column: self.column,
        }
    }

    /// Restore previous lexer state
    fn restore_state(&mut self, state: LexerState) {
        self.position = state.position;
        self.row = state.row;
        self.column = state.column;
    }

    /// Read an identifier token (alphanumeric starting with letter or underscore)
    fn read_identifier(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();
        let mut content = String::new();

        // Must start with letter or underscore (but only if underscore is part of a longer identifier)
        if let Some(ch) = self.peek() {
            if ch.is_alphabetic() {
                content.push(ch);
                self.advance();
            } else if ch == '_' {
                // Only start with underscore if followed by alphanumeric (not standalone delimiter)
                let next_pos = self.position + 1;
                if let Some(&next_ch) = self.input.get(next_pos) {
                    if next_ch.is_alphabetic() || next_ch.is_numeric() || next_ch == '_' {
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

        Some(ScannerToken::Identifier {
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
    fn read_newline(&mut self) -> Option<ScannerToken> {
        let start_pos = self.current_position();

        if let Some(ch) = self.peek() {
            if ch == '\r' {
                // Handle CRLF sequence
                self.advance(); // Consume \r
                if self.peek() == Some('\n') {
                    self.advance(); // Consume \n
                }

                return Some(ScannerToken::Newline {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
            } else if ch == '\n' {
                // Handle LF
                self.advance(); // Consume \n

                return Some(ScannerToken::Newline {
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
    fn read_whitespace(&mut self) -> Option<ScannerToken> {
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
            Some(ScannerToken::Whitespace {
                content,
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            })
        }
    }

    /// Read a blank line token (line containing only whitespace, NOT including line break)  
    fn read_blankline(&mut self) -> Option<ScannerToken> {
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
        let mut whitespace_content = String::new();

        // Collect any whitespace on this line
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                whitespace_content.push(ch);
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
                return Some(ScannerToken::BlankLine {
                    whitespace: whitespace_content,
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
                return Some(ScannerToken::BlankLine {
                    whitespace: whitespace_content,
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                });
            }
        }

        // Also handle end of file after whitespace-only content
        if self.is_at_end() && self.position > saved_position {
            return Some(ScannerToken::BlankLine {
                whitespace: whitespace_content,
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

    /// Check if we're at the start of line content after proper indentation
    fn is_at_line_start_after_indent(&self, tokens: &[ScannerToken]) -> bool {
        use crate::syntax::core::indentation::is_valid_indentation_level;

        // Look back at recent tokens to see if we just processed proper indentation
        if tokens.is_empty() {
            return false;
        }

        // First check: if we're at a valid indentation level (not column 0),
        // and the most recent token is whitespace, we might be at a valid position
        if self.column > 0 && is_valid_indentation_level(self.column) {
            if let Some(last_token) = tokens.last() {
                if matches!(last_token, ScannerToken::Whitespace { .. }) {
                    // We're at a valid indentation level with whitespace - this is a valid position
                    return true;
                }
            }
        }

        // We need to handle two cases:
        // 1. Just after an Indent token (possibly with following whitespace)
        // 2. After Newline + valid indentation whitespace (for continued indented lines)

        // Look at the last few tokens in reverse order
        let mut tokens_checked = 0;
        let mut saw_whitespace = false;

        for token in tokens.iter().rev() {
            tokens_checked += 1;
            if tokens_checked > 5 {
                break; // Don't look too far back
            }

            match token {
                ScannerToken::Whitespace { .. } => {
                    if !saw_whitespace {
                        saw_whitespace = true;
                        // If this is the first token we see (most recent),
                        // continue to check what's before it
                        continue;
                    }
                }
                ScannerToken::Indent { span, .. } => {
                    // We found an Indent token
                    // Check if the indent itself is valid (multiple of 4)
                    let indent_level = span.end.column - span.start.column;
                    if is_valid_indentation_level(indent_level) {
                        // Valid if we've only seen at most one whitespace token since
                        return tokens_checked <= 2;
                    }
                    return false;
                }
                ScannerToken::Newline { .. } => {
                    // We found a newline
                    // Valid if the next token (going forward) is whitespace with valid indentation
                    if saw_whitespace {
                        // Get the whitespace token that came after this newline
                        if let Some(ScannerToken::Whitespace { content, .. }) =
                            tokens.iter().rev().nth(tokens_checked - 2)
                        {
                            return is_valid_indentation_level(content.len());
                        }
                    }
                    return false;
                }
                _ => {
                    // Any other token means we're not at a valid position
                    return false;
                }
            }
        }

        false
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

    pub fn test_read_newline(&mut self) -> Option<ScannerToken> {
        self.read_newline()
    }

    pub fn test_advance(&mut self) -> Option<char> {
        self.advance()
    }
}

impl VerbatimLexer for Lexer {
    fn current_position(&self) -> Position {
        Position {
            row: self.row,
            column: self.column,
        }
    }

    fn row(&self) -> usize {
        self.row
    }

    fn column(&self) -> usize {
        self.column
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

    fn get_absolute_position(&self) -> usize {
        self.position
    }
}

// Trait implementations for reference lexing
use crate::syntax::elements::references::{CitationRefLexer, PageRefLexer};

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

impl crate::syntax::elements::references::session_ref::SessionRefLexer for Lexer {
    fn current_position(&self) -> crate::cst::Position {
        crate::cst::Position {
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

impl crate::syntax::elements::references::ReferenceLexer for Lexer {
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

    fn ref_classifier(
        &self,
    ) -> &crate::ast::elements::references::reference_types::ReferenceClassifier {
        // Create a static classifier instance
        static CLASSIFIER: std::sync::OnceLock<
            crate::ast::elements::references::reference_types::ReferenceClassifier,
        > = std::sync::OnceLock::new();
        CLASSIFIER.get_or_init(
            crate::ast::elements::references::reference_types::ReferenceClassifier::new,
        )
    }

    fn backtrack(&mut self, position: usize, row: usize, column: usize) {
        self.position = position;
        self.row = row;
        self.column = column;
    }
}

impl crate::syntax::elements::references::footnote_ref::FootnoteRefLexer for Lexer {
    fn current_position(&self) -> crate::cst::Position {
        crate::cst::Position {
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
