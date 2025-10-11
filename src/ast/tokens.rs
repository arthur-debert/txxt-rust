//! Token-level AST nodes for character-precise language server support
//!
//! This module defines the lowest-level AST nodes that maintain exact source
//! positions for every character. This enables precise language server features
//! like hover, autocomplete, go-to-definition, and syntax highlighting.
//!
//! # Parsing Pipeline Position
//!
//! **Phase 1.b: Tokenization**
//!
//! These tokens are produced by the lexer after verbatim line marking (1.a).
//! The tokenizer converts raw source text into character-precise tokens with
//! exact source positions. This is the foundation for all subsequent parsing
//! phases and language server precision.
//!
//! Pipeline: `Source Text` → `Verbatim Marking` → **`Tokens`** → `Block Grouping` → `AST Nodes`

use serde::{Deserialize, Serialize};

/// Precise source position for character-level language server support
///
/// Unlike traditional AST source spans, we need both start and end positions
/// because inline elements don't necessarily start at column 0, and we need
/// precise boundaries for language server operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Individual token with precise source location
///
/// Every piece of text in the AST is represented as tokens to enable:
/// - Character-precise hover information
/// - Exact autocomplete trigger points
/// - Precise syntax highlighting
/// - Accurate error underlining
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Token {
    /// Regular text content (words, identifiers)
    Text { content: String, span: SourceSpan },

    /// Whitespace (spaces, tabs - after normalization)
    Whitespace { content: String, span: SourceSpan },

    /// Punctuation and special characters
    Punctuation { content: String, span: SourceSpan },

    /// Numbers (for list items, session numbers)
    Number { content: String, span: SourceSpan },

    /// Symbols and markers (*, -, ::, etc.)
    Symbol { content: String, span: SourceSpan },

    /// Line breaks (preserved for structural information)
    LineBreak { span: SourceSpan },
}

impl Token {
    /// Get the source span for this token
    pub fn span(&self) -> &SourceSpan {
        match self {
            Token::Text { span, .. } => span,
            Token::Whitespace { span, .. } => span,
            Token::Punctuation { span, .. } => span,
            Token::Number { span, .. } => span,
            Token::Symbol { span, .. } => span,
            Token::LineBreak { span } => span,
        }
    }

    /// Get the text content of this token (empty for LineBreak)
    pub fn content(&self) -> &str {
        match self {
            Token::Text { content, .. } => content,
            Token::Whitespace { content, .. } => content,
            Token::Punctuation { content, .. } => content,
            Token::Number { content, .. } => content,
            Token::Symbol { content, .. } => content,
            Token::LineBreak { .. } => "",
        }
    }
}

/// Collection of tokens that forms a logical text unit
///
/// This bridges the gap between character-level precision (tokens) and
/// semantic structure (blocks/inlines). Most semantic operations work
/// with TokenSequence, while language server operations drill down to
/// individual tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenSequence {
    pub tokens: Vec<Token>,
}

impl Default for TokenSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenSequence {
    /// Create a new empty token sequence
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Get the overall source span covering all tokens
    pub fn span(&self) -> Option<SourceSpan> {
        if self.tokens.is_empty() {
            return None;
        }

        let start = self.tokens[0].span().start.clone();
        let end = self.tokens.last().unwrap().span().end.clone();

        Some(SourceSpan { start, end })
    }

    /// Get the text content by concatenating all token content
    pub fn text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.content())
            .collect::<Vec<_>>()
            .join("")
    }
}
