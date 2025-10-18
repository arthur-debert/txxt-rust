//! Content tokens for TXXT parsing pipeline
//!
//! This module defines content-related scanner tokens that represent
//! the actual text content and whitespace in TXXT documents.

use serde::{Deserialize, Serialize};

use crate::ast::elements::scanner_tokens::{Position, SourceSpan};

/// Regular text content (words, sentences, paragraphs)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextToken {
    /// The text content
    pub content: String,
    /// Source span of the text
    pub span: SourceSpan,
}

/// Whitespace characters (spaces and tabs, but not newlines)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhitespaceToken {
    /// The whitespace content
    pub content: String,
    /// Source span of the whitespace
    pub span: SourceSpan,
}

/// Identifier (variable names, labels)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifierToken {
    /// The identifier content
    pub content: String,
    /// Source span of the identifier
    pub span: SourceSpan,
}
