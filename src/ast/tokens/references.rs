//! Reference tokens for TXXT parsing pipeline
//!
//! This module defines reference scanner tokens that represent
//! various types of references in TXXT documents.

use serde::{Deserialize, Serialize};

use crate::ast::elements::scanner_tokens::{Position, SourceSpan};

/// Reference markers ([text], [@citation], [#section])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefMarkerToken {
    /// The reference content
    pub content: String,
    /// Source span of the reference
    pub span: SourceSpan,
}

/// Footnote references ([1], [2], [^label])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FootnoteRefToken {
    /// The footnote type information
    pub footnote_type: crate::lexer::elements::references::footnote_ref::FootnoteType,
    /// Source span of the footnote reference
    pub span: SourceSpan,
}

/// Citation reference ([@key])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CitationRefToken {
    /// The citation content
    pub content: String,
    /// Source span of the citation reference
    pub span: SourceSpan,
}

/// Page reference ([p.123] or [p.123-125])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageRefToken {
    /// The page reference content
    pub content: String,
    /// Source span of the page reference
    pub span: SourceSpan,
}

/// Session reference ([#1.2] or [#section])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRefToken {
    /// The session reference content
    pub content: String,
    /// Source span of the session reference
    pub span: SourceSpan,
}
