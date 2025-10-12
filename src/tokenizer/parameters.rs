//! Parameter tokenization for TXXT parameter syntax
//!
//! Handles parsing of key=value,key2=value2 parameter syntax with support for:
//! - Quoted strings with escape sequences
//! - Boolean shorthand (key without value)
//! - Namespaced keys (org.example.metadata)

use crate::ast::tokens::{Position, SourceSpan, Token};
use crate::tokenizer::infrastructure::lexer::Lexer;

/// Trait for parameter parsing
pub trait ParameterLexer {
    /// Get current position
    fn current_position(&self) -> Position;

    /// Peek at current character
    fn peek(&self) -> Option<char>;

    /// Advance to next character
    fn advance(&mut self) -> Option<char>;

    /// Check if we're at end of input
    fn is_at_end(&self) -> bool;

    /// Get the input as a char slice for content extraction
    fn get_input(&self) -> &[char];
}

/// Parse a parameter string into individual Parameter tokens
/// Input format: key=value,key2="quoted value",key3,key4=value
pub fn parse_parameters<L: ParameterLexer>(lexer: &mut L, input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        // Skip whitespace
        while pos < chars.len() && (chars[pos] == ' ' || chars[pos] == '\t') {
            pos += 1;
        }

        if pos >= chars.len() {
            break;
        }

        // Parse a single parameter
        if let Some((key, value, consumed)) = parse_single_parameter(&chars, pos) {
            let start_pos = lexer.current_position();

            // Advance lexer position to match consumed chars
            for _ in 0..consumed {
                lexer.advance();
            }

            tokens.push(Token::Parameter {
                key,
                value,
                span: SourceSpan {
                    start: start_pos,
                    end: lexer.current_position(),
                },
            });

            pos += consumed;

            // Skip comma if present
            if pos < chars.len() && chars[pos] == ',' {
                pos += 1;
                lexer.advance();
            }
        } else {
            // Skip invalid character
            pos += 1;
            lexer.advance();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_parameter_simple() {
        let chars: Vec<char> = "key=value".chars().collect();
        let result = parse_single_parameter(&chars, 0);
        assert_eq!(result, Some(("key".to_string(), "value".to_string(), 9)));
    }

    #[test]
    fn test_parse_single_parameter_boolean() {
        let chars: Vec<char> = "debug".chars().collect();
        let result = parse_single_parameter(&chars, 0);
        assert_eq!(result, Some(("debug".to_string(), "true".to_string(), 5)));
    }

    #[test]
    fn test_parse_single_parameter_quoted() {
        let chars: Vec<char> = "title=\"My Document\"".chars().collect();
        let result = parse_single_parameter(&chars, 0);
        assert_eq!(
            result,
            Some(("title".to_string(), "My Document".to_string(), 19))
        );
    }

    #[test]
    fn test_parse_single_parameter_escaped() {
        let chars: Vec<char> = "message=\"She said, \\\"Hello!\\\"\"".chars().collect();
        let result = parse_single_parameter(&chars, 0);
        assert_eq!(
            result,
            Some((
                "message".to_string(),
                "She said, \"Hello!\"".to_string(),
                30
            ))
        );
    }

    #[test]
    fn test_parse_single_parameter_namespaced() {
        let chars: Vec<char> = "org.example.key=value".chars().collect();
        let result = parse_single_parameter(&chars, 0);
        assert_eq!(
            result,
            Some(("org.example.key".to_string(), "value".to_string(), 21))
        );
    }
}
