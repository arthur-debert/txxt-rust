use crate::tokenizer::{Token, TokenType};

/// Utilities for analyzing and splitting tokens into element groups
pub struct TokenAnalyzer;

impl TokenAnalyzer {
    /// Split a token sequence into separate element groups
    pub fn split_tokens_into_elements(tokens: &[Token]) -> Vec<Vec<&Token>> {
        let mut result = Vec::new();
        let mut current_group = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];

            // Skip EOF tokens
            if token.token_type == TokenType::Eof {
                i += 1;
                continue;
            }

            // Check if this token starts a new element
            let starts_new_element = match token.token_type {
                // Annotations start with ::
                TokenType::PragmaMarker => {
                    // Look ahead to see if this is an annotation pattern
                    Self::looks_like_annotation_start(&tokens[i..])
                }
                // Definitions have text followed by ::
                // Verbatim blocks have text followed by :
                TokenType::Text => {
                    // Look ahead to see if this is a definition pattern or verbatim pattern
                    // But don't start a new element if we're already inside a verbatim block
                    if Self::is_inside_verbatim_block(&current_group) {
                        false
                    } else {
                        Self::looks_like_definition_start(&tokens[i..])
                            || Self::looks_like_verbatim_start(&tokens[i..])
                    }
                }
                // List items start with - or numbers
                TokenType::Dash | TokenType::SequenceMarker => {
                    // Don't start new element if inside verbatim
                    !Self::is_inside_verbatim_block(&current_group)
                }
                // Verbatim blocks (but only if not preceded by text on same line - that would be a title)
                TokenType::VerbatimStart => {
                    // Check if this VerbatimStart is preceded by text on the same line
                    // If so, it's part of a verbatim title and shouldn't start a new element
                    !Self::verbatim_start_has_title(&tokens[..i + 1])
                }
                // Verbatim content should never start a new element
                TokenType::VerbatimContent | TokenType::VerbatimEnd => false,
                // Blank lines are individual elements (unless inside verbatim)
                TokenType::BlankLine => !Self::is_inside_verbatim_block(&current_group),
                _ => false,
            };

            // If starting a new element and we have a current group, save it
            if starts_new_element && !current_group.is_empty() {
                result.push(current_group);
                current_group = Vec::new();
            }

            current_group.push(token);
            i += 1;
        }

        // Add the last group if it has content
        if !current_group.is_empty() {
            result.push(current_group);
        }

        result
    }

    /// Check if tokens look like an annotation start: :: label ::
    pub fn looks_like_annotation_start(tokens: &[Token]) -> bool {
        if tokens.len() < 3 {
            return false;
        }

        // Must start with ::
        if tokens[0].token_type != TokenType::PragmaMarker {
            return false;
        }

        // Look for pattern :: identifier/text ::
        let mut pragma_count = 0;
        for token in tokens {
            if token.token_type == TokenType::PragmaMarker {
                pragma_count += 1;
                if pragma_count >= 2 {
                    return true;
                }
            } else if matches!(token.token_type, TokenType::Identifier | TokenType::Text)
                && pragma_count == 1
            {
                // Found content between pragmas, continue looking for second ::
                continue;
            } else if token.token_type == TokenType::Newline {
                // Newline before second :: means this might be multiline
                return pragma_count >= 2;
            }
        }

        false
    }

    /// Check if tokens look like a definition start: term ::
    pub fn looks_like_definition_start(tokens: &[Token]) -> bool {
        if tokens.len() < 2 {
            return false;
        }

        // Look for :: somewhere in the tokens
        for token in tokens {
            if token.token_type == TokenType::DefinitionMarker {
                return true;
            }
            // Stop looking if we hit a newline without finding ::
            if token.token_type == TokenType::Newline {
                break;
            }
        }

        false
    }

    /// Check if tokens look like a verbatim start: title :
    pub fn looks_like_verbatim_start(tokens: &[Token]) -> bool {
        if tokens.len() < 2 {
            return false;
        }

        // Look for VerbatimStart (:) after text
        for token in tokens {
            if token.token_type == TokenType::VerbatimStart {
                return true;
            }
            // Stop looking if we hit a newline without finding :
            if token.token_type == TokenType::Newline {
                break;
            }
        }

        false
    }

    /// Check if a VerbatimStart token has a title (text preceding it on the same line)
    pub fn verbatim_start_has_title(tokens_up_to_start: &[Token]) -> bool {
        if tokens_up_to_start.is_empty() {
            return false;
        }

        // Look backward from the VerbatimStart to see if there's text without a newline
        for token in tokens_up_to_start.iter().rev() {
            if token.token_type == TokenType::VerbatimStart {
                continue; // Skip the VerbatimStart itself
            }
            if matches!(token.token_type, TokenType::Text | TokenType::Identifier) {
                return true; // Found text before the VerbatimStart
            }
            if token.token_type == TokenType::Newline {
                return false; // Hit a newline, so no title on the same line
            }
        }

        false
    }

    /// Check if we're currently inside a verbatim block (started but not finished)
    pub fn is_inside_verbatim_block(current_group: &[&Token]) -> bool {
        let mut verbatim_start_count = 0;
        let mut verbatim_end_count = 0;

        for token in current_group {
            if token.token_type == TokenType::VerbatimStart {
                verbatim_start_count += 1;
            } else if token.token_type == TokenType::VerbatimEnd {
                verbatim_end_count += 1;
            }
        }

        // We're inside a verbatim block if we've seen a start but not completed the end pair
        verbatim_start_count > 0 && verbatim_end_count < 2
    }
}
