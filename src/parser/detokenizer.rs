//! Detokenizer - Round-trip Verification
//!
//! Provides functionality to reconstruct source text from tokens,
//! enabling round-trip verification and debugging of the parsing pipeline.
//!
//! The detokenizer takes a stream of tokens and reconstructs source text that,
//! when re-tokenized, produces identical tokens. This is crucial for:
//! - Verifying parsing pipeline correctness
//! - Debugging tokenization issues
//! - Round-trip testing
//!
//! Note: The reconstructed text may differ from the original in whitespace
//! and punctuation that wasn't tokenized, but re-tokenizing will produce
//! identical tokens.

use crate::ast::tokens::Token;
use crate::parser::pipeline::BlockGroup;

/// Detokenizer for round-trip verification
pub struct Detokenizer;

impl Detokenizer {
    /// Create a new detokenizer
    pub fn new() -> Self {
        Self
    }

    /// Reconstruct source text from a flat list of tokens
    ///
    /// This produces text that, when re-tokenized, yields the same tokens.
    /// The output may differ from the original source in non-tokenized content.
    pub fn detokenize_tokens(&self, tokens: &[Token]) -> Result<String, DetokenizeError> {
        let mut result = String::new();
        let mut prev_token: Option<&Token> = None;

        for token in tokens {
            // Add appropriate spacing between tokens
            if let Some(prev) = prev_token {
                self.add_spacing(&mut result, prev, token);
            }

            // Append the token content
            self.append_token(&mut result, token)?;

            // Remember this token for spacing decisions
            if !matches!(token, Token::Eof { .. }) {
                prev_token = Some(token);
            }
        }

        Ok(result)
    }

    /// Reconstruct source text from block groups
    ///
    /// Takes the output of Phase 2a (block grouping) and reconstructs
    /// the original source text for verification purposes.
    pub fn detokenize(&self, blocks: &BlockGroup) -> Result<String, DetokenizeError> {
        let mut result = String::new();
        self.append_block_group(&mut result, blocks, 0)?;
        Ok(result)
    }

    /// Add appropriate spacing between tokens
    fn add_spacing(&self, result: &mut String, prev: &Token, curr: &Token) {
        use Token::*;

        match (prev, curr) {
            // Newline handling
            (Newline { .. }, _) => {
                // Newline already added by prev token
            }
            (BlankLine { .. }, _) => {
                // BlankLine already added newlines
            }

            // No space before punctuation
            (_, Period { .. }) | (_, RightBracket { .. }) | (_, RightParen { .. }) => {
                // No space
            }

            // No space after opening brackets/parens
            (LeftBracket { .. }, _) | (LeftParen { .. }, _) | (AtSign { .. }, _) => {
                // No space
            }

            // Space after punctuation (except brackets/parens)
            (Period { .. }, _) | (Colon { .. }, _) => {
                result.push(' ');
            }

            // Space between text tokens
            (Text { .. }, Text { .. }) => {
                result.push(' ');
            }
            
            // Space between text and inline delimiters
            (Text { .. }, BoldDelimiter { .. }) |
            (Text { .. }, ItalicDelimiter { .. }) |
            (Text { .. }, CodeDelimiter { .. }) |
            (Text { .. }, MathDelimiter { .. }) |
            (BoldDelimiter { .. }, Text { .. }) |
            (ItalicDelimiter { .. }, Text { .. }) |
            (CodeDelimiter { .. }, Text { .. }) |
            (MathDelimiter { .. }, Text { .. }) => {
                result.push(' ');
            }

            // Space after sequence markers
            (SequenceMarker { .. }, _) => {
                // Space already included in sequence marker
            }

            // Default: no space
            _ => {}
        }
    }

    /// Recursively append a block group to the result
    fn append_block_group(
        &self,
        result: &mut String,
        block: &BlockGroup,
        indent_level: usize,
    ) -> Result<(), DetokenizeError> {
        let mut prev_token: Option<&Token> = None;

        // First, append all tokens at this level
        for token in &block.tokens {
            // Add indentation at the start of lines
            if (result.is_empty() || result.ends_with('\n'))
                && !matches!(
                    token,
                    Token::Newline { .. } | Token::BlankLine { .. } | Token::Eof { .. }
                ) {
                    result.push_str(&"    ".repeat(indent_level));
                }

            // Add spacing between tokens
            if let Some(prev) = prev_token {
                self.add_spacing(result, prev, token);
            }

            self.append_token(result, token)?;

            if !matches!(token, Token::Eof { .. }) {
                prev_token = Some(token);
            }
        }

        // Then, process each child with increased indentation
        for child in &block.children {
            self.append_block_group(result, child, indent_level + 1)?;
        }

        Ok(())
    }

    /// Append a single token to the result string
    fn append_token(&self, result: &mut String, token: &Token) -> Result<(), DetokenizeError> {
        match token {
            Token::Text { content, .. } => {
                result.push_str(content);
            }
            Token::Newline { .. } => {
                result.push('\n');
            }
            Token::BlankLine { .. } => {
                // Ensure we have a blank line
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            Token::Indent { .. } => {
                // Indent tokens are consumed during block grouping
                // They don't produce output directly
            }
            Token::Dedent { .. } => {
                // Dedent tokens are consumed during block grouping
                // They don't produce output directly
            }
            Token::SequenceMarker { marker_type, .. } => {
                result.push_str(marker_type.content());
                result.push(' ');
            }
            Token::AnnotationMarker { content, .. } => {
                result.push_str(content);
            }
            Token::DefinitionMarker { content, .. } => {
                result.push_str(content);
                result.push(' ');
                result.push_str("::");
            }
            Token::Dash { .. } => {
                result.push('-');
            }
            Token::Period { .. } => {
                result.push('.');
            }
            Token::LeftBracket { .. } => {
                result.push('[');
            }
            Token::RightBracket { .. } => {
                result.push(']');
            }
            Token::AtSign { .. } => {
                result.push('@');
            }
            Token::LeftParen { .. } => {
                result.push('(');
            }
            Token::RightParen { .. } => {
                result.push(')');
            }
            Token::Colon { .. } => {
                result.push(':');
            }
            Token::Identifier { content, .. } => {
                result.push_str(content);
            }
            Token::RefMarker { content, .. } => {
                result.push('[');
                result.push_str(content);
                result.push(']');
            }
            Token::FootnoteRef { footnote_type, .. } => {
                use crate::tokenizer::inline::references::footnote_ref::FootnoteType;
                match footnote_type {
                    FootnoteType::Naked(n) => {
                        result.push('[');
                        result.push_str(&n.to_string());
                        result.push(']');
                    }
                    FootnoteType::Labeled(label) => {
                        result.push('[');
                        result.push('^');
                        result.push_str(label);
                        result.push(']');
                    }
                }
            }
            Token::VerbatimTitle { content, .. } => {
                result.push_str(content);
                result.push(':');
            }
            Token::VerbatimContent { content, .. } => {
                result.push_str(content);
            }
            Token::VerbatimLabel { content, .. } => {
                result.push_str("::");
                result.push(' ');
                result.push_str(content);
            }
            Token::Parameter { key, value, .. } => {
                result.push_str(key);
                result.push('=');
                // Check if value needs quotes
                if value.contains(' ') || value.contains(',') || value.contains('=') {
                    result.push('"');
                    result.push_str(value);
                    result.push('"');
                } else {
                    result.push_str(value);
                }
            }
            Token::BoldDelimiter { .. } => {
                result.push('*');
            }
            Token::ItalicDelimiter { .. } => {
                result.push('_');
            }
            Token::CodeDelimiter { .. } => {
                result.push('`');
            }
            Token::MathDelimiter { .. } => {
                result.push('#');
            }
            Token::CitationRef { content, .. } => {
                result.push_str("[@");
                result.push_str(content);
                result.push(']');
            }
            Token::PageRef { content, .. } => {
                result.push_str("[p.");
                result.push_str(content);
                result.push(']');
            }
            Token::SessionRef { content, .. } => {
                result.push_str("[#");
                result.push_str(content);
                result.push(']');
            }
            Token::Eof { .. } => {
                // EOF doesn't produce output
            }
        }
        Ok(())
    }
}

impl Default for Detokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Detokenization errors
#[derive(Debug, Clone)]
pub enum DetokenizeError {
    InvalidBlockStructure(String),
    MissingTokenInfo(String),
}

impl std::fmt::Display for DetokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetokenizeError::InvalidBlockStructure(msg) => {
                write!(f, "Invalid block structure: {}", msg)
            }
            DetokenizeError::MissingTokenInfo(msg) => {
                write!(f, "Missing token information: {}", msg)
            }
        }
    }
}

impl std::error::Error for DetokenizeError {}
