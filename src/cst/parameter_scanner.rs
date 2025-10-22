//! Low-level parameter scanner for txxt parameter syntax
//!
//! This module provides reusable functions for scanning parameter sequences
//! into low-level scanner tokens. It handles the basic tokenization of:
//! - Identifier (keys)
//! - Equals (=)
//! - Text/QuotedString (values)
//! - Comma (,)
//! - Colon (:) (separator between label and parameters)
//!
//! The scanner produces basic tokens that are later assembled into a Parameters
//! semantic token by the high-level tokenizer.

use super::primitives::{Position, SourceSpan};
use super::scanner_tokens::ScannerToken;

/// Scan a parameter string into low-level scanner tokens
///
/// Input format: `key=value,key2="quoted value",key3`
/// Output: Vec of ScannerToken (Identifier, Equals, Text/QuotedString, Comma)
///
/// # Arguments
/// * `input` - The parameter string to scan
/// * `start_pos` - Starting position in source for span tracking
///
/// # Returns
/// Vector of scanner tokens representing the parameter sequence
pub fn scan_parameter_string(input: &str, start_pos: Position) -> Vec<ScannerToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut current_pos = start_pos;
    let mut after_equals = false; // Track if we just saw an equals sign

    while pos < chars.len() {
        // Skip whitespace
        while pos < chars.len() && (chars[pos] == ' ' || chars[pos] == '\t') {
            let start = current_pos;
            current_pos = advance_position(current_pos, chars[pos]);
            pos += 1;
            
            tokens.push(ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start,
                    end: current_pos,
                },
            });
        }

        if pos >= chars.len() {
            break;
        }

        let ch = chars[pos];

        // Scan different token types
        if (ch.is_ascii_alphabetic() || ch == '_') && !after_equals {
            // Identifier (parameter key) - only if not after equals
            let (identifier, consumed) = scan_identifier(&chars, pos);
            let start = current_pos;
            for _ in 0..consumed {
                current_pos = advance_position(current_pos, chars[pos]);
                pos += 1;
            }
            tokens.push(ScannerToken::Identifier {
                content: identifier,
                span: SourceSpan {
                    start,
                    end: current_pos,
                },
            });
        } else if ch == '=' {
            after_equals = true;
            // Equals sign
            let start = current_pos;
            current_pos = advance_position(current_pos, ch);
            pos += 1;
            tokens.push(ScannerToken::Equals {
                span: SourceSpan {
                    start,
                    end: current_pos,
                },
            });
        } else if ch == ',' {
            // Comma separator
            after_equals = false; // Reset after comma
            let start = current_pos;
            current_pos = advance_position(current_pos, ch);
            pos += 1;
            tokens.push(ScannerToken::Comma {
                span: SourceSpan {
                    start,
                    end: current_pos,
                },
            });
        } else if ch == '"' {
            after_equals = false; // We're reading a value
            // Quoted string
            match scan_quoted_string(&chars, pos) {
                Some((content, consumed)) => {
                    let start = current_pos;
                    for _ in 0..consumed {
                        current_pos = advance_position(current_pos, chars[pos]);
                        pos += 1;
                    }
                    tokens.push(ScannerToken::QuotedString {
                        content,
                        span: SourceSpan {
                            start,
                            end: current_pos,
                        },
                    });
                }
                None => {
                    // Malformed quoted string - emit as text and advance
                    let start = current_pos;
                    current_pos = advance_position(current_pos, ch);
                    pos += 1;
                    tokens.push(ScannerToken::Text {
                        content: ch.to_string(),
                        span: SourceSpan {
                            start,
                            end: current_pos,
                        },
                    });
                }
            }
        } else if ch == ':' {
            // Colon (for label:params separator)
            let start = current_pos;
            current_pos = advance_position(current_pos, ch);
            pos += 1;
            tokens.push(ScannerToken::Colon {
                span: SourceSpan {
                    start,
                    end: current_pos,
                },
            });
        } else {
            // Unquoted value text (read until comma or end)
            after_equals = false; // We're reading a value
            let (text, consumed) = scan_unquoted_value(&chars, pos);
            if !text.is_empty() {
                let start = current_pos;
                for _ in 0..consumed {
                    current_pos = advance_position(current_pos, chars[pos]);
                    pos += 1;
                }
                tokens.push(ScannerToken::Text {
                    content: text,
                    span: SourceSpan {
                        start,
                        end: current_pos,
                    },
                });
            } else {
                // Unknown character, skip it
                current_pos = advance_position(current_pos, ch);
                pos += 1;
            }
        }
    }

    tokens
}

/// Scan an identifier (parameter key)
/// Returns (identifier, consumed_chars)
fn scan_identifier(chars: &[char], start: usize) -> (String, usize) {
    let mut pos = start;

    // Must start with letter or underscore
    if pos >= chars.len() || (!chars[pos].is_ascii_alphabetic() && chars[pos] != '_') {
        return (String::new(), 0);
    }

    // Continue with alphanumeric, underscore, dash, or period (for namespaces)
    while pos < chars.len() {
        let ch = chars[pos];
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            pos += 1;
        } else {
            break;
        }
    }

    // Don't end with period
    if pos > start && chars[pos - 1] == '.' {
        pos -= 1;
    }

    let identifier: String = chars[start..pos].iter().collect();
    (identifier, pos - start)
}

/// Scan a quoted string with escape sequence support
/// Returns (unescaped_content, consumed_chars) including quotes
fn scan_quoted_string(chars: &[char], start: usize) -> Option<(String, usize)> {
    if start >= chars.len() || chars[start] != '"' {
        return None;
    }

    let mut pos = start + 1; // Skip opening quote
    let mut content = String::new();

    while pos < chars.len() {
        let ch = chars[pos];

        if ch == '"' {
            // Found closing quote
            return Some((content, pos - start + 1));
        } else if ch == '\\' && pos + 1 < chars.len() {
            // Escape sequence
            pos += 1;
            let escaped_ch = chars[pos];
            match escaped_ch {
                '"' => content.push('"'),
                '\\' => content.push('\\'),
                'n' => content.push('\n'),
                't' => content.push('\t'),
                'r' => content.push('\r'),
                _ => {
                    // Unknown escape - treat as literal
                    content.push('\\');
                    content.push(escaped_ch);
                }
            }
            pos += 1;
        } else {
            content.push(ch);
            pos += 1;
        }
    }

    // Unclosed quote - return None
    None
}

/// Scan an unquoted value (until comma or end)
/// Returns (value, consumed_chars)
fn scan_unquoted_value(chars: &[char], start: usize) -> (String, usize) {
    let mut pos = start;

    // Read until comma, equals, colon, or end
    // Colons should not appear in unquoted values (use quoted strings for that)
    while pos < chars.len() {
        let ch = chars[pos];
        if ch == ',' || ch == '=' || ch == ':' {
            break;
        }
        pos += 1;
    }

    if pos == start {
        return (String::new(), 0);
    }

    // Trim trailing whitespace
    let mut end = pos;
    while end > start && (chars[end - 1] == ' ' || chars[end - 1] == '\t') {
        end -= 1;
    }

    let value: String = chars[start..end].iter().collect();
    (value, pos - start)
}

/// Advance position by one character
fn advance_position(mut pos: Position, ch: char) -> Position {
    if ch == '\n' {
        pos.row += 1;
        pos.column = 0;
    } else {
        pos.column += 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_simple_parameter() {
        let start_pos = Position { row: 0, column: 0 };
        let tokens = scan_parameter_string("key=value", start_pos);
        
        // Should have: Identifier("key"), Equals, Text("value")
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], ScannerToken::Identifier { content, .. } if content == "key"));
        assert!(matches!(&tokens[1], ScannerToken::Equals { .. }));
        assert!(matches!(&tokens[2], ScannerToken::Text { content, .. } if content == "value"));
    }

    #[test]
    fn test_scan_quoted_parameter() {
        let start_pos = Position { row: 0, column: 0 };
        let tokens = scan_parameter_string("title=\"My Document\"", start_pos);
        
        // Should have: Identifier("title"), Equals, QuotedString("My Document")
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], ScannerToken::Identifier { content, .. } if content == "title"));
        assert!(matches!(&tokens[1], ScannerToken::Equals { .. }));
        assert!(matches!(&tokens[2], ScannerToken::QuotedString { content, .. } if content == "My Document"));
    }

    #[test]
    fn test_scan_multiple_parameters() {
        let start_pos = Position { row: 0, column: 0 };
        let tokens = scan_parameter_string("key1=value1,key2=value2", start_pos);
        
        // Filter out whitespace for easier testing
        let non_ws_tokens: Vec<&ScannerToken> = tokens.iter()
            .filter(|t| !matches!(t, ScannerToken::Whitespace { .. }))
            .collect();
        
        // Should have: key1, =, value1, comma, key2, =, value2
        assert_eq!(non_ws_tokens.len(), 7);
    }

    #[test]
    fn test_scan_escaped_quotes() {
        let start_pos = Position { row: 0, column: 0 };
        let tokens = scan_parameter_string("message=\"She said, \\\"Hello!\\\"\"", start_pos);
        
        // Check that the quoted string has escape sequences processed
        if let ScannerToken::QuotedString { content, .. } = &tokens[2] {
            assert_eq!(content, "She said, \"Hello!\"");
        } else {
            panic!("Expected QuotedString token");
        }
    }

    #[test]
    fn test_scan_namespaced_key() {
        let start_pos = Position { row: 0, column: 0 };
        let tokens = scan_parameter_string("org.example.key=value", start_pos);
        
        assert!(matches!(&tokens[0], ScannerToken::Identifier { content, .. } if content == "org.example.key"));
    }

    #[test]
    fn test_scan_boolean_shorthand() {
        let start_pos = Position { row: 0, column: 0 };
        let tokens = scan_parameter_string("debug", start_pos);
        
        // Should just have: Identifier("debug")
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], ScannerToken::Identifier { content, .. } if content == "debug"));
    }
}
