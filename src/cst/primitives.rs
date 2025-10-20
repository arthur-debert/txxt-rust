//! Core primitive types for the Concrete Syntax Tree
//!
//! This module defines the foundational types used across all CST tokens:
//! - `Position`: Character-precise (row, column) location
//! - `SourceSpan`: Range of source positions
//! - `ScannerTokenSequence`: Ordered collection of scanner tokens

use serde::{Deserialize, Serialize};

use super::scanner_tokens::ScannerToken;

/// Precise source position for character-level language server support
///
/// Unlike traditional AST source spans, we need both start and end positions
/// because inline elements don't necessarily start at column 0, and we need
/// precise boundaries for language server operations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed)
    pub row: usize,
    /// Column number (0-indexed, UTF-8 byte offset)
    pub column: usize,
}

/// Source span covering a range of characters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

/// Collection of scanner tokens that forms a logical text unit
///
/// This bridges the gap between character-level precision (scanner tokens) and
/// semantic structure (blocks/inlines). Most semantic operations work
/// with ScannerTokenSequence, while language server operations drill down to
/// individual scanner tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScannerTokenSequence {
    pub tokens: Vec<ScannerToken>,
}

impl Default for ScannerTokenSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl ScannerTokenSequence {
    /// Create a new empty scanner token sequence
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Get the overall source span covering all scanner tokens
    pub fn span(&self) -> Option<SourceSpan> {
        if self.tokens.is_empty() {
            return None;
        }

        let start = self.tokens[0].span().start;
        let end = self.tokens.last().unwrap().span().end;

        Some(SourceSpan { start, end })
    }

    /// Get the text content by concatenating all scanner token content
    pub fn text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.content())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Create a scanner token sequence from a vector of scanner tokens
    pub fn from_tokens(tokens: Vec<ScannerToken>) -> Self {
        Self { tokens }
    }
}
