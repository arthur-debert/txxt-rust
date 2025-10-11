use crate::tokenizer::{Token, TokenType};

/// Utilities for extracting text content from tokens
pub struct TextExtractor;

impl TextExtractor {
    /// Extract plain text from tokens
    pub fn extract_text(tokens: &[&Token]) -> String {
        tokens
            .iter()
            .filter_map(|t| match t.token_type {
                TokenType::Text | TokenType::Identifier => t.value.as_ref(),
                _ => None,
            })
            .cloned()
            .collect::<Vec<String>>()
            .join("")
    }

    /// Extract text with inline formatting
    pub fn extract_text_with_inlines(tokens: &[&Token]) -> String {
        let mut result = String::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = tokens[i];

            match token.token_type {
                TokenType::Text | TokenType::Identifier => {
                    if let Some(value) = &token.value {
                        result.push_str(value);
                    }
                }
                TokenType::EmphasisMarker
                | TokenType::StrongMarker
                | TokenType::CodeMarker
                | TokenType::MathMarker => {
                    // For now, just include the marker content
                    if let Some(value) = &token.value {
                        result.push_str(value);
                    }
                }
                TokenType::Newline => {
                    result.push(' ');
                }
                _ => {}
            }
            i += 1;
        }

        result
    }

    /// Extract verbatim content
    pub fn extract_verbatim_content(tokens: &[&Token]) -> String {
        tokens
            .iter()
            .filter_map(|t| {
                if t.token_type == TokenType::VerbatimContent {
                    t.value.as_ref()
                } else {
                    None
                }
            })
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Extract verbatim title (text before VerbatimStart)
    pub fn extract_verbatim_title(tokens: &[&Token]) -> Option<String> {
        let mut title_tokens = Vec::new();

        for token in tokens {
            if token.token_type == TokenType::VerbatimStart {
                break;
            }
            if matches!(token.token_type, TokenType::Text | TokenType::Identifier) {
                title_tokens.push(*token);
            }
        }

        if title_tokens.is_empty() {
            None
        } else {
            let title = Self::extract_text(&title_tokens).trim().to_string();
            if title.is_empty() {
                None
            } else {
                Some(title)
            }
        }
    }

    /// Extract verbatim label
    pub fn extract_verbatim_label(tokens: &[&Token]) -> Option<String> {
        // Look for identifier between VerbatimEnd tokens
        let mut in_label = false;
        for token in tokens {
            if token.token_type == TokenType::VerbatimEnd {
                if let Some(value) = &token.value {
                    if value == "(" {
                        in_label = true;
                        continue;
                    } else if value == ")" {
                        break;
                    }
                }
            }
            if in_label && token.token_type == TokenType::Identifier {
                if let Some(label) = &token.value {
                    return Some(label.clone());
                }
            }
        }
        None
    }
}
