//! TXXT Lexer - Character-precise tokenization for new AST
//!
//! Converts TXXT source text into Token enum variants with precise SourceSpan
//! positioning for language server support.

use crate::ast::reference_types::ReferenceClassifier;
use crate::ast::tokens::{Position, SourceSpan, Token};
use crate::tokenizer::inline::{
    parse_parameters, read_inline_delimiter, InlineDelimiterLexer, ParameterLexer,
};
use crate::tokenizer::markers::{read_sequence_marker, SequenceMarkerLexer};
use crate::tokenizer::verbatim_scanner::{VerbatimBlock, VerbatimScanner};
use regex::Regex;

/// Pattern type for :: tokens on the current line
#[derive(Debug, Clone, PartialEq)]
enum ColonPattern {
    /// :: label :: pattern (annotation)
    Annotation,
    /// term :: pattern (definition)
    Definition,
    /// standalone :: pattern
    Standalone,
}

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
    // Regex patterns for :: detection
    annotation_pattern: Regex,
    definition_pattern: Regex,
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
            // Regex for :: label :: pattern (annotation) - flexible spacing
            annotation_pattern: Regex::new(r"::\s*\w+.*?\s*::").unwrap(),
            // Regex for content :: pattern (definition) - ensure :: is exactly at end
            definition_pattern: Regex::new(r"\w+.*?::\s*$").unwrap(),
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

            if let Some(token) = self.read_definition_marker() {
                tokens.push(token);
            } else if let Some(token) = self.read_annotation_marker() {
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

        // Post-process annotation and definition parameters
        tokens = self.process_annotation_parameters(tokens);
        tokens = self.process_definition_parameters(tokens);

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

    /// Read a definition marker token (term ::)
    fn read_definition_marker(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        // Check for "::"
        if self.peek() == Some(':') {
            let saved_position = self.position;
            let saved_row = self.row;
            let saved_column = self.column;

            self.advance(); // First ':'

            if self.peek() == Some(':') {
                self.advance(); // Second ':'

                // Immediately check if this :: is part of an invalid sequence
                if let Some(next_ch) = self.peek() {
                    if next_ch == ':' {
                        // This is ":::" or longer, not a valid definition marker
                        self.position = saved_position;
                        self.row = saved_row;
                        self.column = saved_column;
                        return None;
                    }
                }

                // Also check if this :: is preceded by a colon (making it part of ":::...")
                if start_pos.column > 0 {
                    let line = self.get_current_line();
                    let col_idx = start_pos.column;
                    if col_idx > 0 {
                        if let Some(prev_char) = line.chars().nth(col_idx - 1) {
                            if prev_char == ':' {
                                // This :: is preceded by ':', making it part of ":::" sequence
                                self.position = saved_position;
                                self.row = saved_row;
                                self.column = saved_column;
                                return None;
                            }
                        }
                    }
                }

                // Use regex-based pattern detection to determine context
                let pattern = self.detect_colon_pattern();

                match pattern {
                    ColonPattern::Definition => {
                        // This is definitely a definition marker
                        return Some(Token::DefinitionMarker {
                            content: "::".to_string(),
                            span: SourceSpan {
                                start: start_pos,
                                end: self.current_position(),
                            },
                        });
                    }
                    ColonPattern::Annotation => {
                        // This line contains annotation pattern, so this :: is not definition
                        self.position = saved_position;
                        self.row = saved_row;
                        self.column = saved_column;
                        return None;
                    }
                    ColonPattern::Standalone => {
                        // Check if followed by whitespace/end (could be definition at end of line)
                        if let Some(next_ch) = self.peek() {
                            if next_ch == '\n'
                                || next_ch == '\r'
                                || next_ch == ' '
                                || next_ch == '\t'
                            {
                                // Check if there's content before this :: (making it a definition)
                                if !self.is_start_of_annotation_pattern(start_pos) {
                                    return Some(Token::DefinitionMarker {
                                        content: "::".to_string(),
                                        span: SourceSpan {
                                            start: start_pos,
                                            end: self.current_position(),
                                        },
                                    });
                                }
                            }
                        } else if self.is_at_end() {
                            // At end of input, check if there's content before
                            if !self.is_start_of_annotation_pattern(start_pos) {
                                return Some(Token::DefinitionMarker {
                                    content: "::".to_string(),
                                    span: SourceSpan {
                                        start: start_pos,
                                        end: self.current_position(),
                                    },
                                });
                            }
                        }
                    }
                }
            }

            // Not a definition marker, backtrack
            self.position = saved_position;
            self.row = saved_row;
            self.column = saved_column;
        }

        None
    }

    /// Check if this :: is at the start of an annotation pattern by looking backwards
    fn is_start_of_annotation_pattern(&self, _start_pos: Position) -> bool {
        // Definition pattern: "term ::" - has non-whitespace before ::
        // Annotation pattern: ":: label ::" - has only whitespace before first ::

        // Check if we're at the beginning of the line or only whitespace precedes
        let start_absolute = self.position - 2; // We advanced past the ::, so go back

        // Look backwards from the start of :: to see what came before on this line
        let mut pos = start_absolute;
        while pos > 0 {
            pos -= 1;
            if let Some(&ch) = self.input.get(pos) {
                if ch == '\n' || ch == '\r' {
                    // Hit newline, everything before :: was whitespace - this is annotation start
                    return true;
                } else if ch != ' ' && ch != '\t' {
                    // Found non-whitespace content before ::, this is NOT annotation start
                    return false;
                }
            }
        }

        // Reached start of input with only whitespace - this is annotation start
        true
    }

    /// Get the current line from the input
    fn get_current_line(&self) -> String {
        let mut line_start = self.position;
        let mut line_end = self.position;

        // Find start of current line
        while line_start > 0 {
            if let Some(&ch) = self.input.get(line_start - 1) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                line_start -= 1;
            } else {
                break;
            }
        }

        // Find end of current line
        while line_end < self.input.len() {
            if let Some(&ch) = self.input.get(line_end) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                line_end += 1;
            } else {
                break;
            }
        }

        self.input[line_start..line_end].iter().collect()
    }

    /// Detect what type of :: pattern this line contains
    fn detect_colon_pattern(&self) -> ColonPattern {
        let line = self.get_current_line();

        // First check for invalid sequences (:::, ::::, etc.)
        if line.contains(":::") {
            return ColonPattern::Standalone; // Treat invalid sequences as standalone (will be rejected)
        }

        // Check for annotation pattern first (:: label ::)
        if self.annotation_pattern.is_match(&line) {
            return ColonPattern::Annotation;
        }

        // Check for definition pattern (term ::)
        if self.definition_pattern.is_match(&line) {
            return ColonPattern::Definition;
        }

        // Default to standalone
        ColonPattern::Standalone
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

                // Also check if this :: is preceded by a colon (making it part of ":::...")
                if start_pos.column > 0 {
                    let line = self.get_current_line();
                    let col_idx = start_pos.column;
                    if col_idx > 0 {
                        if let Some(prev_char) = line.chars().nth(col_idx - 1) {
                            if prev_char == ':' {
                                // This :: is preceded by ':', making it part of ":::" sequence
                                self.position = saved_position;
                                self.row = saved_row;
                                self.column = saved_column;
                                return None;
                            }
                        }
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

    /// Process annotation parameters by finding :: label:params :: patterns and splitting them
    fn process_annotation_parameters(&mut self, tokens: Vec<Token>) -> Vec<Token> {
        let mut processed_tokens = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // Look for annotation pattern: AnnotationMarker + content + AnnotationMarker
            if let Some(Token::AnnotationMarker { .. }) = tokens.get(i) {
                if let Some(annotation_tokens) = self.extract_annotation_content(&tokens, i) {
                    // Found a complete annotation pattern
                    let (start_idx, end_idx, content_tokens) = annotation_tokens;

                    // Add the opening AnnotationMarker
                    processed_tokens.push(tokens[start_idx].clone());

                    // Process the content tokens for parameters
                    let processed_content = self.process_annotation_content(content_tokens);
                    processed_tokens.extend(processed_content);

                    // Add the closing AnnotationMarker
                    processed_tokens.push(tokens[end_idx].clone());

                    // Skip to after the processed tokens
                    i = end_idx + 1;
                } else {
                    // Not a complete annotation pattern, keep the token as is
                    processed_tokens.push(tokens[i].clone());
                    i += 1;
                }
            } else {
                // Not an annotation marker, keep the token as is
                processed_tokens.push(tokens[i].clone());
                i += 1;
            }
        }

        processed_tokens
    }

    /// Extract content tokens between two AnnotationMarker tokens
    fn extract_annotation_content(
        &self,
        tokens: &[Token],
        start_idx: usize,
    ) -> Option<(usize, usize, Vec<Token>)> {
        // Find the closing AnnotationMarker
        let mut content_tokens = Vec::new();
        let mut end_idx = None;

        for (i, token) in tokens.iter().enumerate().skip(start_idx + 1) {
            match token {
                Token::AnnotationMarker { .. } => {
                    // Found closing marker
                    end_idx = Some(i);
                    break;
                }
                Token::Eof { .. } => {
                    // Hit EOF before finding closing marker
                    break;
                }
                _ => {
                    // Content token
                    content_tokens.push(token.clone());
                }
            }
        }

        end_idx.map(|end_idx| (start_idx, end_idx, content_tokens))
    }

    /// Process annotation content tokens to split label from parameters
    fn process_annotation_content(&mut self, content_tokens: Vec<Token>) -> Vec<Token> {
        if content_tokens.is_empty() {
            return content_tokens;
        }

        // Reconstruct the full content string from tokens
        let full_content = self.reconstruct_content_string(&content_tokens);

        // Check if content contains parameters (has a colon)
        if let Some(colon_pos) = full_content.find(':') {
            // Split label from parameters
            let label = &full_content[..colon_pos];
            let params_str = &full_content[colon_pos + 1..];

            let mut result_tokens = Vec::new();

            // Create clean label token (reuse span from first content token)
            if let Some(first_token) = content_tokens.first() {
                result_tokens.push(Token::Text {
                    content: label.to_string(),
                    span: first_token.span().clone(),
                });
            }

            // Parse and add parameter tokens (reuse span from last content token for positioning)
            if !params_str.trim().is_empty() {
                let param_tokens = parse_parameters(self, params_str);
                result_tokens.extend(param_tokens);
            }

            result_tokens
        } else {
            // No parameters, return original content tokens
            content_tokens
        }
    }

    /// Reconstruct full content string from tokens (for parameter parsing)
    fn reconstruct_content_string(&self, tokens: &[Token]) -> String {
        if tokens.is_empty() {
            return String::new();
        }

        // Get the span covering all tokens
        let start_span = tokens.first().unwrap().span();
        let end_span = tokens.last().unwrap().span();

        // Extract the original text directly from the input
        let _start_pos = start_span.start.row * 1000 + start_span.start.column; // Simple position calculation
        let _end_pos = end_span.end.row * 1000 + end_span.end.column;

        // Reconstruct from the actual input characters
        if start_span.start.row == end_span.end.row {
            // Same line - extract substring
            let line = self.get_current_line_from_row(start_span.start.row);
            if start_span.start.column < line.len() && end_span.end.column <= line.len() {
                return line[start_span.start.column..end_span.end.column]
                    .iter()
                    .collect();
            }
        }

        // Fallback: concatenate token contents (less accurate but safe)
        let mut content = String::new();
        for token in tokens {
            match token {
                Token::Text { content: text, .. } => {
                    if !content.is_empty() && !text.is_empty() {
                        // Add missing punctuation between tokens (heuristic)
                        let last_char = content.chars().last().unwrap_or(' ');
                        let first_char = text.chars().next().unwrap_or(' ');

                        if last_char.is_alphanumeric() && first_char.is_alphanumeric() {
                            // Look at the positions to determine what character should be between
                            content.push(':'); // For now, assume colon (this is a heuristic)
                        }
                    }
                    content.push_str(text);
                }
                Token::Identifier { content: text, .. } => {
                    content.push_str(text);
                }
                _ => {
                    // Handle other token types
                    if let Some(token_content) = self.get_token_text_content(token) {
                        content.push_str(&token_content);
                    }
                }
            }
        }

        content
    }

    /// Get the current line from input at a specific row
    fn get_current_line_from_row(&self, row: usize) -> Vec<char> {
        let input_string: String = self.input.iter().collect();
        let lines: Vec<&str> = input_string.lines().collect();

        if row < lines.len() {
            lines[row].chars().collect()
        } else {
            Vec::new()
        }
    }

    /// Get text content from various token types for reconstruction
    fn get_token_text_content(&self, token: &Token) -> Option<String> {
        match token {
            Token::Text { content, .. } => Some(content.clone()),
            Token::Identifier { content, .. } => Some(content.clone()),
            _ => None,
        }
    }

    /// Process definition parameters by finding term:params :: patterns and splitting them
    fn process_definition_parameters(&mut self, tokens: Vec<Token>) -> Vec<Token> {
        let mut processed_tokens = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // Look for definition pattern: Text tokens followed by DefinitionMarker
            if self.is_definition_pattern(&tokens, i) {
                if let Some(definition_tokens) = self.extract_definition_content(&tokens, i) {
                    // Found a definition pattern with potential parameters
                    let (_term_start_idx, _term_end_idx, def_marker_idx, term_tokens) =
                        definition_tokens;

                    // Process the term tokens for parameters
                    let processed_term = self.process_definition_term_content(term_tokens);
                    processed_tokens.extend(processed_term);

                    // Add the DefinitionMarker
                    processed_tokens.push(tokens[def_marker_idx].clone());

                    // Skip to after the processed tokens
                    i = def_marker_idx + 1;
                } else {
                    // Not a definition pattern with parameters, keep the token as is
                    processed_tokens.push(tokens[i].clone());
                    i += 1;
                }
            } else {
                // Not part of a definition pattern, keep the token as is
                processed_tokens.push(tokens[i].clone());
                i += 1;
            }
        }

        processed_tokens
    }

    /// Check if current position starts a definition pattern
    fn is_definition_pattern(&self, tokens: &[Token], start_idx: usize) -> bool {
        // Look for Text/Identifier tokens followed by DefinitionMarker
        // Skip non-text tokens and look ahead for DefinitionMarker
        for token in tokens.iter().skip(start_idx) {
            match token {
                Token::DefinitionMarker { .. } => return true,
                Token::Text { .. } | Token::Identifier { .. } => continue,
                Token::Newline { .. } | Token::BlankLine { .. } => continue, // Allow line breaks in definition terms
                _ => return false, // Hit something else, not a definition pattern
            }
        }
        false
    }

    /// Extract definition term tokens before DefinitionMarker
    fn extract_definition_content(
        &self,
        tokens: &[Token],
        start_idx: usize,
    ) -> Option<(usize, usize, usize, Vec<Token>)> {
        let mut term_tokens = Vec::new();
        let mut def_marker_idx = None;
        let mut term_start_idx = start_idx;
        let mut term_end_idx = start_idx;

        // Find the DefinitionMarker and collect term tokens before it
        for (i, token) in tokens.iter().enumerate().skip(start_idx) {
            match token {
                Token::DefinitionMarker { .. } => {
                    def_marker_idx = Some(i);
                    break;
                }
                Token::Text { .. } | Token::Identifier { .. } => {
                    if term_tokens.is_empty() {
                        term_start_idx = i;
                    }
                    term_tokens.push(token.clone());
                    term_end_idx = i;
                }
                Token::Newline { .. } | Token::BlankLine { .. } => {
                    // Skip whitespace but don't include in term tokens
                    continue;
                }
                _ => {
                    // Hit something else, this breaks the definition pattern
                    return None;
                }
            }
        }

        if let Some(def_idx) = def_marker_idx {
            if !term_tokens.is_empty() {
                Some((term_start_idx, term_end_idx, def_idx, term_tokens))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Process definition term tokens to split term from parameters
    fn process_definition_term_content(&mut self, term_tokens: Vec<Token>) -> Vec<Token> {
        if term_tokens.is_empty() {
            return term_tokens;
        }

        // Reconstruct the full term string from tokens
        let full_term = self.reconstruct_content_string(&term_tokens);

        // Check if term contains parameters (has a colon)
        if let Some(colon_pos) = full_term.find(':') {
            // Split term from parameters
            let term = &full_term[..colon_pos];
            let params_str = &full_term[colon_pos + 1..];

            let mut result_tokens = Vec::new();

            // Create clean term token (reuse span from first term token)
            if let Some(first_token) = term_tokens.first() {
                result_tokens.push(Token::Text {
                    content: term.to_string(),
                    span: first_token.span().clone(),
                });
            }

            // Parse and add parameter tokens
            if !params_str.trim().is_empty() {
                let param_tokens = parse_parameters(self, params_str);
                result_tokens.extend(param_tokens);
            }

            result_tokens
        } else {
            // No parameters, return original term tokens
            term_tokens
        }
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
}
