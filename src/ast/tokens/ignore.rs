//! Ignore tokens for TXXT parsing pipeline
//!
//! This module defines ignore scanner tokens that represent
//! content that should be preserved exactly as written without processing.

use serde::{Deserialize, Serialize};

use crate::ast::elements::scanner_tokens::{Position, SourceSpan};

/// Verbatim block title (title:)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbatimTitleToken {
    /// The title content
    pub content: String,
    /// Source span of the title
    pub span: SourceSpan,
}

/// Verbatim block content (preserved exactly)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbatimContentToken {
    /// The verbatim content
    pub content: String,
    /// Source span of the content
    pub span: SourceSpan,
}

/// Verbatim block label (:: label syntax)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbatimLabelToken {
    /// The label content
    pub content: String,
    /// Source span of the label
    pub span: SourceSpan,
}

/// Parameter key-value pair (key=value)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterToken {
    /// The parameter key
    pub key: String,
    /// The parameter value
    pub value: String,
    /// Source span of the parameter
    pub span: SourceSpan,
}
