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
use crate::tokenizer::indentation::INDENT_SIZE;

/// Detokenizer for round-trip verification
pub struct Detokenizer;

impl Detokenizer {
    /// Create a new detokenizer
    pub fn new() -> Self {
        Self
    }

    /// Reconstruct source text from tokens
    ///
    /// This method takes a flat list of tokens and reconstructs the source text.
    /// It tracks indentation levels using Indent/Dedent tokens to properly
    /// reconstruct the hierarchical structure.
    pub fn detokenize_tokens(&self, tokens: &[Token]) -> Result<String, DetokenizeError> {
        let mut result = String::new();
        let mut prev_token: Option<&Token> = None;
        let mut indent_level: usize = 0;
        let mut at_line_start = true;

        for token in tokens {
            // Handle parameter separators based on previous token
            if let Token::Parameter { key, value, .. } = token {
                // Add appropriate separator before parameter
                if let Some(prev) = prev_token {
                    match prev {
                        Token::VerbatimLabel { .. } => {
                            result.push(':'); // First param after verbatim label
                        }
                        Token::Colon { .. } => {
                            // After a colon in annotation context, no separator needed
                        }
                        Token::Parameter { .. } => {
                            result.push(','); // Subsequent params
                        }
                        _ => {}
                    }
                }

                // Add the parameter
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

                prev_token = Some(token);
                at_line_start = false;
                continue;
            }

            // Track indentation level changes
            match token {
                Token::Indent { .. } => {
                    indent_level += 1;
                    continue;
                }
                Token::Dedent { .. } => {
                    indent_level = indent_level.saturating_sub(1);
                    continue;
                }
                _ => {}
            }

            // Handle BlankLine tokens specially to ensure correct indentation
            if let Token::BlankLine { whitespace, .. } = token {
                // For blank lines, preserve the whitespace as-is since it's part of the blank line
                result.push_str(whitespace);
                result.push('\n');
                at_line_start = true;
                prev_token = Some(token);
                continue;
            }

            // Add indentation at the start of lines (but not for certain tokens)
            if at_line_start && !matches!(token, Token::Whitespace { .. }) {
                let indent_ws = " ".repeat(indent_level * INDENT_SIZE);
                result.push_str(&indent_ws);
                at_line_start = false;
            }

            // Skip whitespace tokens at the start of lines as they represent
            // the original indentation that we're replacing
            if at_line_start && matches!(token, Token::Whitespace { .. }) {
                continue;
            }

            // Append the token
            self.append_token(&mut result, token, indent_level)?;
            prev_token = Some(token);

            // Track if we just added a newline
            if matches!(token, Token::Newline { .. }) {
                at_line_start = true;
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

    /// Recursively append a block group to the result
    fn append_block_group(
        &self,
        result: &mut String,
        block: &BlockGroup,
        indent_level: usize,
    ) -> Result<(), DetokenizeError> {
        // Track whether we're at the start of a line
        let mut at_line_start = result.is_empty() || result.ends_with('\n');
        let mut prev_token: Option<&Token> = None;

        // Process all tokens at this level
        for token in &block.tokens {
            // Handle parameter tokens specially for separator logic
            if let Token::Parameter { key, value, .. } = token {
                // Add appropriate separator before parameter
                if let Some(prev) = prev_token {
                    match prev {
                        Token::VerbatimLabel { .. } => {
                            result.push(':'); // First param after verbatim label
                        }
                        Token::Colon { .. } => {
                            // After a colon in annotation context, no separator needed
                        }
                        Token::Parameter { .. } => {
                            result.push(','); // Subsequent params
                        }
                        _ => {}
                    }
                }

                // Add the parameter
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

                prev_token = Some(token);
                at_line_start = false;
                continue;
            }

            // Handle BlankLine tokens specially
            if let Token::BlankLine { whitespace, .. } = token {
                // For blank lines, preserve the whitespace as-is
                result.push_str(whitespace);
                result.push('\n');
                at_line_start = true;
                prev_token = Some(token);
                continue;
            }

            // Add indentation at the start of each line
            if at_line_start && !matches!(token, Token::Indent { .. } | Token::Dedent { .. }) {
                // For child blocks, skip leading whitespace tokens as they represent
                // the original indentation that we're replacing
                if indent_level > 0 && matches!(token, Token::Whitespace { .. }) {
                    continue;
                }

                // Add the correct indentation based on block depth
                let indent_ws = " ".repeat(indent_level * INDENT_SIZE);
                result.push_str(&indent_ws);
                at_line_start = false;
            }

            // Append the token
            self.append_token(result, token, indent_level)?;
            prev_token = Some(token);

            // Track if we just added a newline
            if matches!(token, Token::Newline { .. }) {
                at_line_start = true;
            }
        }

        // Process all children with increased indentation
        for child in &block.children {
            self.append_block_group(result, child, indent_level + 1)?;
        }

        Ok(())
    }

    /// Append a single token to the result string
    fn append_token(
        &self,
        result: &mut String,
        token: &Token,
        current_indent_level: usize,
    ) -> Result<(), DetokenizeError> {
        match token {
            Token::Text { content, .. } => {
                result.push_str(content);
            }
            Token::Newline { .. } => {
                result.push('\n');
            }
            Token::BlankLine { whitespace, .. } => {
                // Add the whitespace content of the blank line, then newline
                result.push_str(whitespace);
                result.push('\n');
            }
            Token::Indent { .. } => {
                // Indent tokens track indent level changes, not actual whitespace
                // The whitespace is handled by Whitespace tokens
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
                result.push('\n');
            }
            Token::VerbatimContent { content, .. } => {
                // For verbatim content, we need to add the wall indentation back
                // Split content into lines and add proper indentation to each
                let lines: Vec<&str> = content.split('\n').collect();
                for (i, line) in lines.iter().enumerate() {
                    if i > 0 {
                        result.push('\n');
                    }
                    // Add current indentation plus one extra level for the wall
                    if !line.is_empty() {
                        result.push_str(&" ".repeat((current_indent_level + 1) * INDENT_SIZE));
                        result.push_str(line);
                    }
                }
                result.push('\n');
            }
            Token::VerbatimLabel { content, .. } => {
                result.push_str("::");
                result.push(' ');
                result.push_str(content);
                // Note: VerbatimLabel tokens don't include a trailing newline
                // The newline after a label comes as a separate Newline token
            }
            Token::Parameter { .. } => {
                // Parameters are handled specially in append_block_group
                // This case should not be reached
                unreachable!("Parameter tokens should be handled in append_block_group");
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
            Token::Whitespace { content, .. } => {
                // Whitespace tokens represent inline spacing (not indentation)
                // Indentation is reconstructed from the BlockGroup hierarchy
                result.push_str(content);
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
