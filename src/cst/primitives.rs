//! Core primitive types for the Concrete Syntax Tree
//!
//! This module defines the foundational types used across all CST tokens:
//! - `Position`: Character-precise (row, column) location
//! - `SourceSpan`: Range of source positions
//! - `ScannerTokenSequence`: Ordered collection of scanner tokens

use serde::{Deserialize, Serialize};

use super::scanner_tokens::ScannerToken;

/// Remove escape backslashes from text content
///
/// The scanner includes backslashes in text tokens when they escape special characters.
/// This function removes those backslashes to get the actual display text.
///
/// # Examples
/// - `\*not bold\*` → `*not bold*`
/// - `\_not italic\_` → `_not italic_`
/// - `\`not code\`` → `` `not code` ``
/// - `\#not math\#` → `#not math#`
///
/// Backslashes before non-special characters are preserved.
fn unescape_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // Check if next character is a special character that was escaped
            if let Some(&next_ch) = chars.peek() {
                if matches!(next_ch, '*' | '_' | '`' | '#' | '[' | ']' | '-' | '\\') {
                    // Skip the backslash, include the escaped character
                    chars.next();
                    result.push(next_ch);
                    continue;
                }
            }
        }
        // Not an escape sequence, include as-is
        result.push(ch);
    }

    result
}

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
        let raw_text = self
            .tokens
            .iter()
            .map(|token| token.content())
            .collect::<Vec<_>>()
            .join("");

        // Process escape sequences: remove backslashes before special characters
        unescape_text(&raw_text)
    }

    /// Create a scanner token sequence from a vector of scanner tokens
    pub fn from_tokens(tokens: Vec<ScannerToken>) -> Self {
        Self { tokens }
    }
}
