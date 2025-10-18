//! Structural tokens for TXXT parsing pipeline
//!
//! This module defines structural scanner tokens that represent
//! document structure like indentation, line breaks, and blank lines.

use serde::{Deserialize, Serialize};

use crate::ast::elements::scanner_tokens::{Position, SourceSpan};

/// Line break characters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewlineToken {
    /// Source span of the newline
    pub span: SourceSpan,
}

/// Blank line (empty line with possible whitespace)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlankLineToken {
    /// The whitespace content of the blank line (spaces/tabs before the newline)
    pub whitespace: String,
    /// Source span of the blank line
    pub span: SourceSpan,
}

/// Indentation increase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndentToken {
    /// Source span of the indent token
    pub span: SourceSpan,
}

/// Indentation decrease  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DedentToken {
    /// Source span of the dedent token
    pub span: SourceSpan,
}

/// End of file marker
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EofToken {
    /// Source span of the EOF token
    pub span: SourceSpan,
}
