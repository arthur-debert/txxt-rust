//! TXXT Marker (::) detection and classification
//!
//! Handles detection of :: tokens and classification based on context:
//! - Annotation: :: label :: or :: label:params ::
//! - Definition: term :: or term:params ::
//! - Standalone: isolated :: tokens

use crate::cst::{Position, ScannerToken, SourceSpan};
use crate::lexer::core::patterns::{
    extract_raw_content_before_span, extract_raw_content_between_spans, get_current_line,
};
use crate::lexer::elements::components::parameters::{parse_parameters, ParameterLexer};
use crate::lexer::tokenization::{Lexer, LexerState};
use regex::Regex;

/// Pattern type for :: tokens on the current line
#[derive(Debug, Clone, PartialEq)]
pub enum ColonPattern {
    /// :: label :: pattern (annotation)
    Annotation,
    /// term :: pattern (definition)
    Definition,
    /// standalone :: pattern
    Standalone,
}

/// Trait for TXXT marker detection capabilities
pub trait TxxtMarkerLexer {
    /// State type for save/restore operations
    type State;

    /// Get current position
    fn current_position(&self) -> Position;

    /// Peek at current character
    fn peek(&self) -> Option<char>;

    /// Advance to next character
    fn advance(&mut self) -> Option<char>;

    /// Check if we're at end of input
    fn is_at_end(&self) -> bool;

    /// Get current line
    fn get_current_line(&self) -> String;

    /// Pattern detection for :: context
    fn detect_colon_pattern(&self) -> ColonPattern;

    /// Check if this :: is at the start of an annotation pattern
    fn is_start_of_annotation_pattern(&self, start_pos: Position) -> bool;

    /// Get regex patterns for :: detection
    fn annotation_pattern(&self) -> &Regex;
    fn definition_pattern(&self) -> &Regex;

    /// Save current lexer state
    fn save_state(&self) -> Self::State;

    /// Restore previous lexer state
    fn restore_state(&mut self, state: Self::State);
}

/// Read definition marker tokens (term ::)
pub fn read_definition_marker<L: TxxtMarkerLexer>(lexer: &mut L) -> Option<ScannerToken> {
    let start_pos = lexer.current_position();

    // Check for "::"
    if lexer.peek() == Some(':') {
        let saved_state = lexer.save_state();

        lexer.advance(); // First ':'

        if lexer.peek() == Some(':') {
            lexer.advance(); // Second ':'

            // Immediately check if this :: is part of an invalid sequence
            if let Some(next_ch) = lexer.peek() {
                if next_ch == ':' {
                    // This is ":::" or longer, not a valid definition marker
                    lexer.restore_state(saved_state);
                    return None;
                }
            }

            // Also check if this :: is preceded by a colon (making it part of ":::...")
            if start_pos.column > 0 {
                let line = lexer.get_current_line();
                let col_idx = start_pos.column;
                if col_idx > 0 {
                    if let Some(prev_char) = line.chars().nth(col_idx - 1) {
                        if prev_char == ':' {
                            // This :: is preceded by ':', making it part of ":::" sequence
                            lexer.restore_state(saved_state);
                            return None;
                        }
                    }
                }
            }

            // Use regex-based pattern detection to determine context
            let pattern = lexer.detect_colon_pattern();

            match pattern {
                ColonPattern::Definition => {
                    // This is definitely a definition marker
                    return Some(ScannerToken::TxxtMarker {
                        span: SourceSpan {
                            start: start_pos,
                            end: lexer.current_position(),
                        },
                    });
                }
                ColonPattern::Annotation => {
                    // This line contains annotation pattern, so this :: is not definition
                    lexer.restore_state(saved_state);
                    return None;
                }
                ColonPattern::Standalone => {
                    // Check if followed by whitespace/end (could be definition at end of line)
                    if let Some(next_ch) = lexer.peek() {
                        if next_ch == '\n' || next_ch == '\r' || next_ch == ' ' || next_ch == '\t' {
                            // Check if there's content before this :: (making it a definition)
                            if !lexer.is_start_of_annotation_pattern(start_pos) {
                                return Some(ScannerToken::TxxtMarker {
                                    span: SourceSpan {
                                        start: start_pos,
                                        end: lexer.current_position(),
                                    },
                                });
                            }
                        }
                    } else if lexer.is_at_end() {
                        // At end of input, check if there's content before
                        if !lexer.is_start_of_annotation_pattern(start_pos) {
                            return Some(ScannerToken::TxxtMarker {
                                span: SourceSpan {
                                    start: start_pos,
                                    end: lexer.current_position(),
                                },
                            });
                        }
                    }
                }
            }
        }

        // Not a definition marker, backtrack
        lexer.restore_state(saved_state);
    }

    None
}

/// Read annotation marker tokens (::)
pub fn read_annotation_marker<L: TxxtMarkerLexer>(lexer: &mut L) -> Option<ScannerToken> {
    let start_pos = lexer.current_position();

    // Check for "::"
    if lexer.peek() == Some(':') {
        let saved_state = lexer.save_state();

        lexer.advance(); // First ':'

        if lexer.peek() == Some(':') {
            lexer.advance(); // Second ':'

            // Check that this is not part of a longer sequence like ":::"
            // Annotation markers should be exactly "::"
            if let Some(next_ch) = lexer.peek() {
                if next_ch == ':' {
                    // This is ":::" or longer, not a valid annotation marker
                    lexer.restore_state(saved_state);
                    return None;
                }
            }

            // Also check if this :: is preceded by a colon (making it part of ":::...")
            if start_pos.column > 0 {
                let line = lexer.get_current_line();
                let col_idx = start_pos.column;
                if col_idx > 0 {
                    if let Some(prev_char) = line.chars().nth(col_idx - 1) {
                        if prev_char == ':' {
                            // This :: is preceded by ':', making it part of ":::" sequence
                            lexer.restore_state(saved_state);
                            return None;
                        }
                    }
                }
            }

            return Some(ScannerToken::TxxtMarker {
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            });
        } else {
            // Not an annotation marker, backtrack
            lexer.restore_state(saved_state);
        }
    }

    None
}

/// Detect what type of :: pattern this line contains
pub fn detect_colon_pattern<L: TxxtMarkerLexer>(lexer: &L) -> ColonPattern {
    let line = lexer.get_current_line();

    // First check for invalid sequences (:::, ::::, etc.)
    if line.contains(":::") {
        return ColonPattern::Standalone; // Treat invalid sequences as standalone (will be rejected)
    }

    // Check for annotation pattern first (:: label ::)
    if lexer.annotation_pattern().is_match(&line) {
        return ColonPattern::Annotation;
    }

    // Check for definition pattern (term ::)
    if lexer.definition_pattern().is_match(&line) {
        return ColonPattern::Definition;
    }

    // Default to standalone
    ColonPattern::Standalone
}

/// Check if this :: is at the start of an annotation pattern by looking backwards
pub fn is_start_of_annotation_pattern<L: TxxtMarkerLexer>(lexer: &L, start_pos: Position) -> bool {
    // Definition pattern: "term ::" - has non-whitespace before ::
    // Annotation pattern: ":: label ::" - has only whitespace before first ::

    // For now, use a simple heuristic - this can be improved later
    let line = lexer.get_current_line();
    let content_before = line.chars().take(start_pos.column).collect::<String>();

    // If only whitespace before ::, this is annotation start
    content_before.trim().is_empty()
}

/// Simple annotation parameter integration - find :: label:params :: and split
pub fn integrate_annotation_parameters<L: ParameterLexer>(
    tokens: Vec<ScannerToken>,
    lexer: &mut L,
) -> Vec<ScannerToken> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if let Some(ScannerToken::TxxtMarker { .. }) = tokens.get(i) {
            // Look for content between annotation markers
            if let Some((start_idx, end_idx, content)) = find_annotation_content(&tokens, i, lexer)
            {
                // Add opening marker
                result.push(tokens[start_idx].clone());

                // Split content at first colon for parameters
                if let Some(colon_pos) = content.find(':') {
                    let label = &content[..colon_pos];
                    let params_str = &content[colon_pos + 1..];

                    // Add clean label token
                    if let Some(first_token) = tokens.get(start_idx + 1) {
                        result.push(ScannerToken::Text {
                            content: label.to_string(),
                            span: first_token.span().clone(),
                        });
                    }

                    // Add colon token to separate label from parameters
                    result.push(ScannerToken::Colon {
                        span: SourceSpan {
                            start: Position {
                                row: 0, // This is a synthetic token, position isn't exact
                                column: 0,
                            },
                            end: Position { row: 0, column: 0 },
                        },
                    });

                    // Add parameter tokens using existing parse_parameters
                    if !params_str.trim().is_empty() {
                        let param_tokens = parse_parameters(lexer, params_str);
                        result.extend(param_tokens);
                    }
                } else {
                    // No parameters, create a clean TEXT token with the raw content
                    if let Some(first_token) = tokens.get(start_idx + 1) {
                        result.push(ScannerToken::Text {
                            content: content.clone(),
                            span: first_token.span().clone(),
                        });
                    }
                }

                // Add closing marker
                result.push(tokens[end_idx].clone());
                i = end_idx + 1;
            } else {
                result.push(tokens[i].clone());
                i += 1;
            }
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

/// Simple definition parameter integration - find term:params :: and split
pub fn integrate_definition_parameters<L: ParameterLexer>(
    tokens: Vec<ScannerToken>,
    _lexer: &mut L,
) -> Vec<ScannerToken> {
    // For now, disable parameter integration to fix duplication issue
    // TODO: Implement proper parameter parsing without token duplication
    tokens
}

/// Find annotation content between markers by extracting raw text
#[allow(dead_code)]
fn find_annotation_content<L: ParameterLexer>(
    tokens: &[ScannerToken],
    start_idx: usize,
    lexer: &L,
) -> Option<(usize, usize, String)> {
    let mut end_idx = None;

    // Find the closing annotation marker
    for (i, token) in tokens.iter().enumerate().skip(start_idx + 1) {
        if matches!(token, ScannerToken::TxxtMarker { .. }) {
            end_idx = Some(i);
            break;
        }
    }

    if let Some(end) = end_idx {
        // Get the raw text between the markers by extracting from source
        if let (Some(start_token), Some(end_token)) = (tokens.get(start_idx), tokens.get(end)) {
            let start_span = start_token.span();
            let end_span = end_token.span();

            // Extract raw content between markers from input source
            let input = lexer.get_input();
            let content = extract_raw_content_between_spans(start_span, end_span, input);
            Some((start_idx, end, content))
        } else {
            None
        }
    } else {
        None
    }
}

/// Find definition content before marker by extracting raw text
#[allow(dead_code)]
fn find_definition_content<L: ParameterLexer>(
    tokens: &[ScannerToken],
    def_idx: usize,
    lexer: &L,
) -> Option<(String, usize)> {
    let mut term_start_idx = def_idx;

    // Look backwards for the start of term content
    for i in (0..def_idx).rev() {
        match &tokens[i] {
            ScannerToken::Text { .. } | ScannerToken::Identifier { .. } => {
                term_start_idx = i;
            }
            ScannerToken::Newline { .. } | ScannerToken::BlankLine { .. } => continue,
            _ => break,
        }
    }

    if term_start_idx < def_idx {
        // Extract raw content from source between term start and definition marker
        if let (Some(start_token), Some(def_token)) =
            (tokens.get(term_start_idx), tokens.get(def_idx))
        {
            let start_span = start_token.span();
            let def_span = def_token.span();

            // Extract raw content from start of term to before the :: marker
            let input = lexer.get_input();
            let content = extract_raw_content_before_span(start_span, def_span, input);
            Some((content, term_start_idx))
        } else {
            None
        }
    } else {
        None
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
