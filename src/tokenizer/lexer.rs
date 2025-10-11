//! TXXT Lexer - Character-precise tokenization for new AST
//!
//! Converts TXXT source text into Token enum variants with precise SourceSpan
//! positioning for language server support.

use crate::ast::tokens::{Position, SourceSpan, Token};
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

/// Main tokenizer that produces new AST Token enum variants
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    row: usize,
    column: usize,
    // Regex patterns for :: detection
    annotation_pattern: Regex,
    definition_pattern: Regex,
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
                if let Some(token) = self.read_sequence_marker() {
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
            } else if let Some(token) = self.read_inline_delimiter() {
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

    /// Check if reference content matches valid patterns
    fn is_valid_ref_content(&self, content: &str) -> bool {
        if content.is_empty() {
            return false;
        }

        // Citation pattern: @identifier
        if let Some(identifier) = content.strip_prefix('@') {
            if content.len() <= 1 {
                return false; // @ alone is not valid
            }
            return identifier
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
        }

        // Section pattern: #number or #number.number etc.
        if let Some(section_ref) = content.strip_prefix('#') {
            if content.len() <= 1 {
                return false; // # alone is not valid
            }
            return self.is_valid_section_ref(section_ref);
        }

        // Footnote pattern: just numbers
        if content.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }

        // URL pattern: contains :// or starts with www. or contains @
        if content.contains("://") || content.starts_with("www.") || content.contains('@') {
            return true;
        }

        // File path pattern: contains / or \ or ends with file extension
        if content.contains('/') || content.contains('\\') || self.has_file_extension(content) {
            return true;
        }

        // Plain text references (anchor names, etc.)
        if content
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return true;
        }

        false
    }

    /// Check if content is a valid section reference (numbers and dots)
    fn is_valid_section_ref(&self, content: &str) -> bool {
        if content.is_empty() {
            return false;
        }

        // Split by dots and check each part is a number or -1
        for part in content.split('.') {
            if part.is_empty() {
                return false;
            }
            if part == "-1" {
                continue; // Allow negative indexing
            }
            if !part.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }
        true
    }

    /// Check if content has a common file extension
    fn has_file_extension(&self, content: &str) -> bool {
        let extensions = [
            ".txt", ".md", ".txxt", ".html", ".htm", ".pdf", ".doc", ".docx", ".png", ".jpg",
            ".jpeg", ".gif", ".svg", ".mp4", ".mp3", ".wav", ".js", ".ts", ".py", ".rs", ".go",
            ".java", ".cpp", ".c", ".h",
        ];

        extensions.iter().any(|ext| content.ends_with(ext))
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

    /// Read inline formatting delimiters (*, _, `, #)
    fn read_inline_delimiter(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        match self.peek()? {
            '*' => {
                self.advance();
                Some(Token::BoldDelimiter {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                })
            }
            '_' => {
                self.advance();
                Some(Token::ItalicDelimiter {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                })
            }
            '`' => {
                self.advance();
                Some(Token::CodeDelimiter {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                })
            }
            '#' => {
                self.advance();
                Some(Token::MathDelimiter {
                    span: SourceSpan {
                        start: start_pos,
                        end: self.current_position(),
                    },
                })
            }
            _ => None,
        }
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

    /// Tokenize a verbatim block into VerbatimStart and VerbatimContent tokens
    fn tokenize_verbatim_block(&mut self, block: &VerbatimBlock) -> Vec<Token> {
        let mut tokens = Vec::new();

        // Create VerbatimStart token for the title line
        let title_start_pos = self.current_position();

        // Advance through the title line to get its content
        let mut title_content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            if ch == ':' {
                // Include the colon in the title content
                title_content.push(ch);
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

        tokens.push(Token::VerbatimStart {
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

        // Create VerbatimEnd token with the full terminator content
        if !terminator_content.trim().is_empty() {
            tokens.push(Token::VerbatimEnd {
                content: terminator_content,
                span: SourceSpan {
                    start: terminator_start_pos,
                    end: self.current_position(),
                },
            });
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
