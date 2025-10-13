//! Phase 1: Lexer (Tokenization Wrapper)
//!
//! This module provides a wrapper interface to the tokenizer for use in the
//! parsing pipeline. Phase 1 is already implemented in `src/tokenizer/`.

// use crate::tokenizer; // TODO: Add when actual integration is implemented

/// Phase 1 Lexer - wrapper around the tokenizer
///
/// This provides a consistent interface for the parsing pipeline while
/// delegating actual tokenization to the proven tokenizer implementation.
pub struct Lexer;

impl Lexer {
    /// Create a new lexer instance
    pub fn new() -> Self {
        Self
    }

    /// Tokenize input text into a stream of positioned tokens
    ///
    /// This is a wrapper around the tokenizer that fits into the
    /// three-phase pipeline architecture.
    pub fn tokenize(&self, _input: &str) -> Result<Vec<()>, LexerError> {
        // TODO: Integrate with actual tokenizer once pipeline is established
        // For now, return placeholder to allow compilation
        Ok(vec![])
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Lexer error types
#[derive(Debug, Clone)]
pub enum LexerError {
    /// Input/output error during tokenization
    IoError(String),
    /// Invalid input that cannot be tokenized
    InvalidInput(String),
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::IoError(msg) => write!(f, "Lexer I/O error: {}", msg),
            LexerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for LexerError {}
