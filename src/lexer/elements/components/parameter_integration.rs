//! Fixed parameter integration that preserves whitespace tokens and correct spans
//!
//! This module provides corrected versions of annotation and definition parameter
//! integration that maintain whitespace tokens and accurate source positions.

use crate::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};
use crate::lexer::elements::components::parameters::ParameterLexer;

/// Process annotation tokens to integrate parameters while preserving whitespace
pub fn integrate_annotation_parameters_fixed<L: ParameterLexer>(
    tokens: Vec<ScannerToken>,
    _lexer: &mut L,
) -> Vec<ScannerToken> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if let ScannerToken::AnnotationMarker { .. } = &tokens[i] {
            // Found opening annotation marker
            result.push(tokens[i].clone());
            i += 1;

            // Collect all tokens until closing annotation marker or end
            let mut content_tokens = Vec::new();
            let mut found_closing = false;
            let mut closing_idx = i;

            while i < tokens.len() {
                if matches!(&tokens[i], ScannerToken::AnnotationMarker { .. }) {
                    found_closing = true;
                    closing_idx = i;
                    break;
                }
                content_tokens.push(tokens[i].clone());
                i += 1;
            }

            if found_closing {
                // Process the content tokens to detect and parse parameters
                let (processed_tokens, _has_params) = process_annotation_content(content_tokens);
                result.extend(processed_tokens);

                // Add closing marker
                result.push(tokens[closing_idx].clone());
                i = closing_idx + 1;
            } else {
                // No closing marker found, keep original content
                result.extend(content_tokens);
            }
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

/// Process definition tokens to integrate parameters while preserving whitespace
pub fn integrate_definition_parameters_fixed<L: ParameterLexer>(
    tokens: Vec<ScannerToken>,
    _lexer: &mut L,
) -> Vec<ScannerToken> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if let ScannerToken::DefinitionMarker { .. } = &tokens[i] {
            // Found definition marker - look backwards for content

            let mut j = result.len();

            // Collect backwards until we hit something that can't be part of term
            while j > 0 {
                j -= 1;
                match &result[j] {
                    ScannerToken::Text { .. }
                    | ScannerToken::Identifier { .. }
                    | ScannerToken::Whitespace { .. }
                    | ScannerToken::Colon { .. } => {
                        // Continue collecting
                    }
                    _ => {
                        // Stop here
                        j += 1;
                        break;
                    }
                }
            }

            // Extract the term tokens
            let term_tokens = result.drain(j..).collect();

            // Process term tokens to detect and parse parameters
            let (processed_tokens, _has_params) = process_definition_content(term_tokens);
            result.extend(processed_tokens);

            // Add the definition marker
            result.push(tokens[i].clone());
            i += 1;
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

/// Process annotation content tokens to detect and parse parameters
fn process_annotation_content(tokens: Vec<ScannerToken>) -> (Vec<ScannerToken>, bool) {
    // Look for the pattern: Text (label) + optional Whitespace + Colon + parameters
    let mut result = Vec::new();
    let mut i = 0;
    let mut found_params = false;

    while i < tokens.len() {
        match &tokens[i] {
            ScannerToken::Text { content, span } => {
                // Check if this might be a label with parameters
                if content.contains(':') {
                    // Split at first colon
                    if let Some(colon_byte_pos) = content.find(':') {
                        let label = &content[..colon_byte_pos];
                        let params_str = &content[colon_byte_pos + 1..];

                        // Calculate character position of colon
                        let colon_char_pos = content[..colon_byte_pos].chars().count();

                        // Create label token with correct span
                        result.push(ScannerToken::Text {
                            content: label.to_string(),
                            span: SourceSpan {
                                start: span.start,
                                end: Position {
                                    row: span.start.row,
                                    column: span.start.column + colon_char_pos,
                                },
                            },
                        });

                        // Create colon token
                        result.push(ScannerToken::Colon {
                            span: SourceSpan {
                                start: Position {
                                    row: span.start.row,
                                    column: span.start.column + colon_char_pos,
                                },
                                end: Position {
                                    row: span.start.row,
                                    column: span.start.column + colon_char_pos + 1,
                                },
                            },
                        });

                        // Parse parameters with correct position
                        if !params_str.trim().is_empty() {
                            let param_start = Position {
                                row: span.start.row,
                                column: span.start.column + colon_char_pos + 1,
                            };
                            let param_tokens =
                                parse_parameters_with_position(params_str, param_start);
                            result.extend(param_tokens);
                            found_params = true;
                        }
                    } else {
                        // No colon in content, keep as-is
                        result.push(tokens[i].clone());
                    }
                } else {
                    // No colon in content, keep as-is
                    result.push(tokens[i].clone());
                }
                i += 1;
            }
            ScannerToken::Colon { span: _ } => {
                // Standalone colon - check if parameters follow
                result.push(tokens[i].clone());
                i += 1;

                // Skip whitespace
                while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. }) {
                    result.push(tokens[i].clone());
                    i += 1;
                }

                // Check for parameter-like content
                if i < tokens.len() {
                    if let ScannerToken::Text {
                        content,
                        span: text_span,
                    } = &tokens[i]
                    {
                        // This might be parameters
                        if content.contains('=') || looks_like_parameters(content) {
                            let param_tokens =
                                parse_parameters_with_position(content, text_span.start);
                            result.extend(param_tokens);
                            found_params = true;
                            i += 1;
                        }
                    }
                }
            }
            _ => {
                // Keep other tokens as-is
                result.push(tokens[i].clone());
                i += 1;
            }
        }
    }

    (result, found_params)
}

/// Process definition content tokens to detect and parse parameters  
fn process_definition_content(tokens: Vec<ScannerToken>) -> (Vec<ScannerToken>, bool) {
    // For definitions, we need to handle: term:params
    // The tokens might be split as: Text("term") + Colon + Text("params")
    // Or combined as: Text("term:params")

    let mut result = Vec::new();
    let mut i = 0;
    let mut found_params = false;

    while i < tokens.len() {
        match &tokens[i] {
            ScannerToken::Text { content, span } => {
                // Check if this contains the whole pattern
                if content.contains(':') {
                    // Split at first colon
                    if let Some(colon_byte_pos) = content.find(':') {
                        let term = &content[..colon_byte_pos];
                        let params_str = &content[colon_byte_pos + 1..];

                        // Calculate character position of colon
                        let colon_char_pos = content[..colon_byte_pos].chars().count();

                        // Create term token with correct span
                        result.push(ScannerToken::Text {
                            content: term.to_string(),
                            span: SourceSpan {
                                start: span.start,
                                end: Position {
                                    row: span.start.row,
                                    column: span.start.column + colon_char_pos,
                                },
                            },
                        });

                        // Create colon token
                        result.push(ScannerToken::Colon {
                            span: SourceSpan {
                                start: Position {
                                    row: span.start.row,
                                    column: span.start.column + colon_char_pos,
                                },
                                end: Position {
                                    row: span.start.row,
                                    column: span.start.column + colon_char_pos + 1,
                                },
                            },
                        });

                        // Parse parameters with correct position
                        if !params_str.trim().is_empty() {
                            let param_start = Position {
                                row: span.start.row,
                                column: span.start.column + colon_char_pos + 1,
                            };
                            let param_tokens =
                                parse_parameters_with_position(params_str, param_start);
                            result.extend(param_tokens);
                            found_params = true;
                        }
                    } else {
                        // No colon in content, keep as-is
                        result.push(tokens[i].clone());
                    }
                } else {
                    // No colon in content, keep as-is
                    result.push(tokens[i].clone());
                }
                i += 1;
            }
            ScannerToken::Colon { .. } => {
                // Colon token - check if parameters follow
                result.push(tokens[i].clone());
                i += 1;

                // Skip whitespace (but preserve it)
                while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. }) {
                    result.push(tokens[i].clone());
                    i += 1;
                }

                // Check for parameter content
                if i < tokens.len() {
                    if let ScannerToken::Text { content, span } = &tokens[i] {
                        if looks_like_parameters(content) {
                            let param_tokens = parse_parameters_with_position(content, span.start);
                            result.extend(param_tokens);
                            found_params = true;
                            i += 1;
                        }
                    }
                }
            }
            _ => {
                // Keep other tokens as-is
                result.push(tokens[i].clone());
                i += 1;
            }
        }
    }

    (result, found_params)
}

/// Check if a string looks like parameters
fn looks_like_parameters(s: &str) -> bool {
    // Simple heuristic: contains '=' or is a valid identifier (boolean shorthand)
    s.contains('=') || is_valid_parameter_key(s)
}

/// Check if a string is a valid parameter key
fn is_valid_parameter_key(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let chars: Vec<char> = s.chars().collect();

    // Must start with letter or underscore
    if !chars[0].is_ascii_alphabetic() && chars[0] != '_' {
        return false;
    }

    // Rest can be alphanumeric, underscore, dash, or dot
    chars[1..]
        .iter()
        .all(|&ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
}

/// Parse parameters with correct source position tracking
fn parse_parameters_with_position(input: &str, start_position: Position) -> Vec<ScannerToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut current_position = start_position;

    while pos < chars.len() {
        // Skip whitespace
        while pos < chars.len() && (chars[pos] == ' ' || chars[pos] == '\t') {
            pos += 1;
            current_position.column += 1;
        }

        if pos >= chars.len() {
            break;
        }

        // Parse a single parameter
        if let Some((key, value, consumed)) = parse_single_parameter(&chars, pos) {
            let param_start = current_position;

            // Calculate end position
            let param_end = Position {
                row: param_start.row,
                column: param_start.column + consumed,
            };

            tokens.push(ScannerToken::Parameter {
                key,
                value,
                span: SourceSpan {
                    start: param_start,
                    end: param_end,
                },
            });

            pos += consumed;
            current_position.column += consumed;

            // Skip comma if present
            if pos < chars.len() && chars[pos] == ',' {
                pos += 1;
                current_position.column += 1;
            }
        } else {
            // Skip invalid character
            pos += 1;
            current_position.column += 1;
        }
    }

    tokens
}

/// Parse a single parameter from character array starting at position
/// Returns (key, value, consumed_chars) or None if invalid
fn parse_single_parameter(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    let mut pos = start;

    // Parse key (identifier with optional namespaces)
    let key = parse_parameter_key(chars, &mut pos)?;

    // Check for = or end (boolean shorthand)
    if pos >= chars.len() || chars[pos] == ',' {
        // Boolean shorthand: key without value means key=true
        return Some((key, "true".to_string(), pos - start));
    }

    if chars[pos] != '=' {
        // Invalid parameter syntax
        return None;
    }

    pos += 1; // Skip =

    // Parse value
    let value = parse_parameter_value(chars, &mut pos)?;

    Some((key, value, pos - start))
}

/// Parse parameter key (identifier with optional dot-separated namespaces)
fn parse_parameter_key(chars: &[char], pos: &mut usize) -> Option<String> {
    let start = *pos;

    // Must start with letter or underscore
    if *pos >= chars.len() || (!chars[*pos].is_ascii_alphabetic() && chars[*pos] != '_') {
        return None;
    }

    *pos += 1;

    // Continue with alphanumeric, underscore, dash, or period
    while *pos < chars.len() {
        let ch = chars[*pos];
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            *pos += 1;
        } else {
            break;
        }
    }

    // Don't end with period
    if *pos > start && chars[*pos - 1] == '.' {
        *pos -= 1;
    }

    if *pos == start {
        return None;
    }

    Some(chars[start..*pos].iter().collect())
}

/// Parse parameter value (unquoted text or quoted string)
fn parse_parameter_value(chars: &[char], pos: &mut usize) -> Option<String> {
    if *pos >= chars.len() {
        return Some(String::new()); // Empty value is valid
    }

    if chars[*pos] == '"' {
        // Quoted string
        parse_quoted_string(chars, pos)
    } else {
        // Unquoted value (until comma or end)
        parse_unquoted_value(chars, pos)
    }
}

/// Parse quoted string with escape sequence support
fn parse_quoted_string(chars: &[char], pos: &mut usize) -> Option<String> {
    if *pos >= chars.len() || chars[*pos] != '"' {
        return None;
    }

    *pos += 1; // Skip opening quote
    let mut value = String::new();

    while *pos < chars.len() {
        let ch = chars[*pos];

        if ch == '"' {
            *pos += 1; // Skip closing quote
            return Some(value);
        } else if ch == '\\' && *pos + 1 < chars.len() {
            // Escape sequence
            *pos += 1;
            let escaped_ch = chars[*pos];
            match escaped_ch {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                'n' => value.push('\n'),
                't' => value.push('\t'),
                'r' => value.push('\r'),
                _ => {
                    // Unknown escape, treat as literal
                    value.push('\\');
                    value.push(escaped_ch);
                }
            }
            *pos += 1;
        } else {
            value.push(ch);
            *pos += 1;
        }
    }

    // Unclosed quote - return None
    None
}

/// Parse unquoted value (until comma or end)
fn parse_unquoted_value(chars: &[char], pos: &mut usize) -> Option<String> {
    let start = *pos;

    // Read until comma or end
    while *pos < chars.len() && chars[*pos] != ',' {
        *pos += 1;
    }

    if *pos == start {
        return Some(String::new()); // Empty unquoted value
    }

    // Trim trailing whitespace
    let mut end = *pos;
    while end > start && (chars[end - 1] == ' ' || chars[end - 1] == '\t') {
        end -= 1;
    }

    Some(chars[start..end].iter().collect())
}
