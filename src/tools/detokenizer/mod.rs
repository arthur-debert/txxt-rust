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

use crate::ast::scanner_tokens::ScannerToken;
use crate::lexer::core::indentation::INDENT_SIZE;
use crate::lexer::pipeline::ScannerTokenTree;

/// Detokenizer for round-trip verification
pub struct Detokenizer;

impl Detokenizer {
    /// Create a new detokenizer
    pub fn new() -> Self {
        Self
    }

    /// Simple detokenization for round-trip verification
    ///
    /// This is a simplified version that reconstructs text from tokens without
    /// complex indentation tracking, suitable for round-trip verification tests.
    pub fn detokenize_for_verification(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<String, DetokenizeError> {
        let mut result = String::new();
        let mut prev_token: Option<&ScannerToken> = None;

        for token in tokens {
            // Skip Indent/Dedent tokens as they're structural markers, not content
            if matches!(
                token,
                ScannerToken::Indent { .. } | ScannerToken::Dedent { .. }
            ) {
                continue;
            }

            // Handle parameter separators based on previous token
            if let ScannerToken::Parameter { key, value, .. } = token {
                if let Some(prev) = prev_token {
                    match prev {
                        ScannerToken::VerbatimLabel { .. } => {
                            result.push(':'); // First param after verbatim label
                        }
                        ScannerToken::Parameter { .. } => {
                            result.push(','); // Subsequent params
                        }
                        _ => {}
                    }
                }

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
                continue;
            }

            // Append the token using simple logic
            self.append_token(&mut result, token, 0)?;
            prev_token = Some(token);
        }

        Ok(result)
    }

    /// Reconstruct source text from block groups
    ///
    /// Takes the output of Phase 2a (block grouping) and reconstructs
    /// the original source text for verification purposes.
    pub fn detokenize(&self, token_tree: &ScannerTokenTree) -> Result<String, DetokenizeError> {
        let mut result = String::new();
        self.append_token_tree(&mut result, token_tree, 0)?;
        Ok(result)
    }

    /// Recursively append a token tree to the result
    fn append_token_tree(
        &self,
        result: &mut String,
        token_tree: &ScannerTokenTree,
        indent_level: usize,
    ) -> Result<(), DetokenizeError> {
        // Track whether we're at the start of a line
        let mut at_line_start = result.is_empty() || result.ends_with('\n');
        let mut prev_token: Option<&ScannerToken> = None;

        // Process all tokens at this level
        for token in &token_tree.tokens {
            // Handle parameter tokens specially for separator logic
            if let ScannerToken::Parameter { key, value, .. } = token {
                // Add appropriate separator before parameter
                if let Some(prev) = prev_token {
                    match prev {
                        ScannerToken::VerbatimLabel { .. } => {
                            result.push(':'); // First param after verbatim label
                        }
                        ScannerToken::Colon { .. } => {
                            // After a colon in annotation context, no separator needed
                        }
                        ScannerToken::Parameter { .. } => {
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
            if let ScannerToken::BlankLine { whitespace, .. } = token {
                // For blank lines, preserve the whitespace as-is
                result.push_str(whitespace);
                result.push('\n');
                at_line_start = true;
                prev_token = Some(token);
                continue;
            }

            // Add indentation at the start of each line
            if at_line_start
                && !matches!(
                    token,
                    ScannerToken::Indent { .. } | ScannerToken::Dedent { .. }
                )
            {
                // For child blocks, skip leading whitespace tokens as they represent
                // the original indentation that we're replacing
                if indent_level > 0 && matches!(token, ScannerToken::Whitespace { .. }) {
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
            if matches!(token, ScannerToken::Newline { .. }) {
                at_line_start = true;
            }
        }

        // Process all children with increased indentation
        for child in &token_tree.children {
            self.append_token_tree(result, child, indent_level + 1)?;
        }

        Ok(())
    }

    /// Append a single token to the result string
    fn append_token(
        &self,
        result: &mut String,
        token: &ScannerToken,
        current_indent_level: usize,
    ) -> Result<(), DetokenizeError> {
        match token {
            ScannerToken::Text { content, .. } => {
                result.push_str(content);
            }
            ScannerToken::Newline { .. } => {
                result.push('\n');
            }
            ScannerToken::BlankLine { whitespace, .. } => {
                // Add the whitespace content of the blank line, then newline
                result.push_str(whitespace);
                result.push('\n');
            }
            ScannerToken::Indent { .. } => {
                // Indent tokens track indent level changes, not actual whitespace
                // The whitespace is handled by Whitespace tokens
            }
            ScannerToken::Dedent { .. } => {
                // Dedent tokens are consumed during block grouping
                // They don't produce output directly
            }
            ScannerToken::SequenceMarker { marker_type, .. } => {
                result.push_str(marker_type.content());
                result.push(' ');
            }
            ScannerToken::AnnotationMarker { content, .. } => {
                result.push_str(content);
            }
            ScannerToken::DefinitionMarker { content, .. } => {
                result.push_str(content);
            }
            ScannerToken::Dash { .. } => {
                result.push('-');
            }
            ScannerToken::Period { .. } => {
                result.push('.');
            }
            ScannerToken::LeftBracket { .. } => {
                result.push('[');
            }
            ScannerToken::RightBracket { .. } => {
                result.push(']');
            }
            ScannerToken::AtSign { .. } => {
                result.push('@');
            }
            ScannerToken::LeftParen { .. } => {
                result.push('(');
            }
            ScannerToken::RightParen { .. } => {
                result.push(')');
            }
            ScannerToken::Colon { .. } => {
                result.push(':');
            }
            ScannerToken::Identifier { content, .. } => {
                result.push_str(content);
            }
            ScannerToken::RefMarker { content, .. } => {
                result.push('[');
                result.push_str(content);
                result.push(']');
            }
            ScannerToken::FootnoteRef { footnote_type, .. } => {
                use crate::lexer::elements::references::footnote_ref::FootnoteType;
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
            ScannerToken::VerbatimTitle { content, .. } => {
                result.push_str(content);
                result.push(':');
                result.push('\n');
            }
            ScannerToken::VerbatimContent { content, .. } => {
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
            ScannerToken::VerbatimLabel { content, .. } => {
                result.push_str("::");
                result.push(' ');
                result.push_str(content);
                // Note: VerbatimLabel tokens don't include a trailing newline
                // The newline after a label comes as a separate Newline token
            }
            ScannerToken::Parameter { .. } => {
                // Parameters are handled specially in append_token_tree
                // This case should not be reached
                unreachable!("Parameter tokens should be handled in append_token_tree");
            }
            ScannerToken::BoldDelimiter { .. } => {
                result.push('*');
            }
            ScannerToken::ItalicDelimiter { .. } => {
                result.push('_');
            }
            ScannerToken::CodeDelimiter { .. } => {
                result.push('`');
            }
            ScannerToken::MathDelimiter { .. } => {
                result.push('#');
            }
            ScannerToken::CitationRef { content, .. } => {
                result.push_str("[@");
                result.push_str(content);
                result.push(']');
            }
            ScannerToken::PageRef { content, .. } => {
                result.push_str("[p.");
                result.push_str(content);
                result.push(']');
            }
            ScannerToken::SessionRef { content, .. } => {
                result.push_str("[#");
                result.push_str(content);
                result.push(']');
            }
            ScannerToken::Whitespace { content, .. } => {
                // Whitespace tokens represent inline spacing (not indentation)
                // Indentation is reconstructed from the ScannerTokenTree hierarchy
                result.push_str(content);
            }
            ScannerToken::Eof { .. } => {
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
