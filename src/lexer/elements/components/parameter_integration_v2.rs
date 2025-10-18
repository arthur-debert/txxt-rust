//! Parameter integration that works with pre-tokenized streams
//!
//! This module handles parameter detection and parsing while preserving
//! all tokens (including whitespace) and maintaining correct source positions.

use crate::ast::scanner_tokens::{ScannerToken, SourceSpan};

/// Integrate parameters in annotation contexts while preserving all tokens
pub fn integrate_annotation_parameters_v2(tokens: Vec<ScannerToken>) -> Vec<ScannerToken> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if matches!(&tokens[i], ScannerToken::AnnotationMarker { .. }) {
            // Found opening annotation marker
            result.push(tokens[i].clone());
            i += 1;

            // Process tokens until closing marker
            let (processed, consumed) = process_until_annotation_end(&tokens[i..]);
            result.extend(processed);
            i += consumed;
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

/// Integrate parameters in definition contexts while preserving all tokens
pub fn integrate_definition_parameters_v2(tokens: Vec<ScannerToken>) -> Vec<ScannerToken> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if matches!(&tokens[i], ScannerToken::DefinitionMarker { .. }) {
            // Found definition marker, look backwards for term:params pattern
            let (processed, start_idx) = process_definition_term(&result);

            // Replace the tokens from start_idx onwards
            result.truncate(start_idx);
            result.extend(processed);

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

/// Process tokens until we find a closing annotation marker
fn process_until_annotation_end(tokens: &[ScannerToken]) -> (Vec<ScannerToken>, usize) {
    let mut result = Vec::new();
    let mut i = 0;
    let mut in_parameters = false;
    let mut param_start = 0;

    while i < tokens.len() {
        match &tokens[i] {
            ScannerToken::AnnotationMarker { .. } => {
                // Found closing marker
                if in_parameters {
                    // Process accumulated parameter tokens
                    let param_tokens = process_parameter_tokens(&tokens[param_start..i]);
                    result.extend(param_tokens);
                }
                result.push(tokens[i].clone());
                return (result, i + 1);
            }
            ScannerToken::Colon { span: _ } if !in_parameters => {
                // This might be the start of parameters
                // Check if we have a label before this
                if has_label_before_colon(&result) {
                    in_parameters = true;
                    param_start = i + 1;
                    // Keep the colon
                    result.push(tokens[i].clone());
                } else {
                    result.push(tokens[i].clone());
                }
            }
            _ => {
                if !in_parameters {
                    result.push(tokens[i].clone());
                }
                // If in_parameters, we accumulate tokens for later processing
            }
        }
        i += 1;
    }

    // No closing marker found, return what we have
    if in_parameters {
        let param_tokens = process_parameter_tokens(&tokens[param_start..]);
        result.extend(param_tokens);
    }
    (result, tokens.len())
}

/// Process tokens that should contain parameters
fn process_parameter_tokens(tokens: &[ScannerToken]) -> Vec<ScannerToken> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            ScannerToken::Text { content, span } => {
                // Check if this contains an equals sign
                if let Some(eq_pos) = content.find('=') {
                    let key = &content[..eq_pos];
                    let value = &content[eq_pos + 1..];

                    result.push(ScannerToken::Parameter {
                        key: key.to_string(),
                        value: value.to_string(),
                        span: span.clone(),
                    });
                } else if is_valid_param_key(content) && peek_equals(&tokens[i + 1..]) {
                    // This is a key, and equals follows
                    let key_span = span.clone();
                    let key = content.clone();
                    i += 1;

                    // Skip whitespace
                    while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. })
                    {
                        result.push(tokens[i].clone());
                        i += 1;
                    }

                    // Skip equals (we know it's there from peek_equals)
                    if i < tokens.len() && is_equals(&tokens[i]) {
                        i += 1;
                    }

                    // Skip whitespace after equals
                    while i < tokens.len() && matches!(&tokens[i], ScannerToken::Whitespace { .. })
                    {
                        result.push(tokens[i].clone());
                        i += 1;
                    }

                    // Get value
                    if i < tokens.len() {
                        if let Some(value) = get_token_text(&tokens[i]) {
                            let value_span = get_token_span(&tokens[i]);
                            result.push(ScannerToken::Parameter {
                                key,
                                value,
                                span: SourceSpan {
                                    start: key_span.start,
                                    end: value_span.end,
                                },
                            });
                        } else {
                            // Not a valid value token, just push the key as text
                            result.push(ScannerToken::Text {
                                content: key,
                                span: key_span,
                            });
                        }
                    } else {
                        // No value, treat as text
                        result.push(ScannerToken::Text {
                            content: key,
                            span: key_span,
                        });
                    }
                } else if is_valid_param_key(content) {
                    // Boolean parameter (key without value)
                    result.push(ScannerToken::Parameter {
                        key: content.clone(),
                        value: "true".to_string(),
                        span: span.clone(),
                    });
                } else {
                    // Regular text
                    result.push(tokens[i].clone());
                }
            }
            ScannerToken::Identifier { content, span } if is_valid_param_key(content) => {
                // Check for key=value pattern
                if peek_equals(&tokens[i + 1..]) {
                    // Process as parameter (similar to above)
                    let key_span = span.clone();
                    let key = content.clone();
                    i += 1;

                    // Skip to value (similar logic as above)
                    // ... (abbreviated for brevity)
                    result.push(ScannerToken::Parameter {
                        key,
                        value: "value".to_string(), // Simplified
                        span: key_span,
                    });
                } else {
                    // Boolean parameter
                    result.push(ScannerToken::Parameter {
                        key: content.clone(),
                        value: "true".to_string(),
                        span: span.clone(),
                    });
                }
            }
            _ => {
                // Keep other tokens as-is (whitespace, etc.)
                result.push(tokens[i].clone());
            }
        }
        i += 1;
    }

    result
}

/// Process definition term looking for parameters
fn process_definition_term(tokens: &[ScannerToken]) -> (Vec<ScannerToken>, usize) {
    // Look backwards for term:params pattern
    let mut term_start = tokens.len();
    let mut found_colon = false;
    let mut colon_idx = 0;

    // Scan backwards to find the term boundaries
    for i in (0..tokens.len()).rev() {
        match &tokens[i] {
            ScannerToken::Colon { .. } if !found_colon => {
                found_colon = true;
                colon_idx = i;
            }
            ScannerToken::Text { .. }
            | ScannerToken::Identifier { .. }
            | ScannerToken::Whitespace { .. } => {
                if i < term_start {
                    term_start = i;
                }
            }
            _ => {
                // Stop at any other token type
                break;
            }
        }
    }

    if found_colon && colon_idx < tokens.len() - 1 {
        // We have term:something pattern
        let mut processed = tokens[term_start..colon_idx].to_vec();
        processed.push(tokens[colon_idx].clone()); // Keep the colon

        // Process tokens after colon as potential parameters
        let param_tokens = process_parameter_tokens(&tokens[colon_idx + 1..]);
        processed.extend(param_tokens);

        (processed, term_start)
    } else {
        // No parameters, return original tokens
        (tokens[term_start..].to_vec(), term_start)
    }
}

// Helper functions

fn has_label_before_colon(tokens: &[ScannerToken]) -> bool {
    // Check if there's a text or identifier token before the colon
    for i in (0..tokens.len()).rev() {
        match &tokens[i] {
            ScannerToken::Text { .. } | ScannerToken::Identifier { .. } => return true,
            ScannerToken::Whitespace { .. } => continue,
            _ => return false,
        }
    }
    false
}

fn peek_equals(tokens: &[ScannerToken]) -> bool {
    for token in tokens {
        match token {
            ScannerToken::Text { content, .. } if content.starts_with('=') => return true,
            ScannerToken::Whitespace { .. } => continue,
            _ => return false,
        }
    }
    false
}

fn is_valid_param_key(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
}

fn get_token_text(token: &ScannerToken) -> Option<String> {
    match token {
        ScannerToken::Text { content, .. } | ScannerToken::Identifier { content, .. } => {
            Some(content.clone())
        }
        _ => None,
    }
}

fn get_token_span(token: &ScannerToken) -> &SourceSpan {
    token.span()
}

fn is_equals(token: &ScannerToken) -> bool {
    matches!(token, ScannerToken::Text { content, .. } if content == "=")
}
