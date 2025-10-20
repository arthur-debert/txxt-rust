//! Formatting tokens for TXXT parsing pipeline
//!
//! This module defines formatting scanner tokens that represent
//! inline formatting delimiters like bold, italic, code, and math.

use serde::{Deserialize, Serialize};

use crate::ast::elements::scanner_tokens::SourceSpan;

/// Bold text delimiter (*)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoldDelimiterToken {
    /// Source span of the delimiter
    pub span: SourceSpan,
}

/// Italic text delimiter (_)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItalicDelimiterToken {
    /// Source span of the delimiter
    pub span: SourceSpan,
}

/// Code text delimiter (`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeDelimiterToken {
    /// Source span of the delimiter
    pub span: SourceSpan,
}

/// Math text delimiter (#)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MathDelimiterToken {
    /// Source span of the delimiter
    pub span: SourceSpan,
}
