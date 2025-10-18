//! Punctuation tokens for TXXT parsing pipeline
//!
//! This module defines punctuation scanner tokens that represent
//! punctuation characters used in TXXT syntax.

use serde::{Deserialize, Serialize};

use crate::ast::elements::scanner_tokens::{Position, SourceSpan};

/// Dash character (-)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashToken {
    /// Source span of the dash
    pub span: SourceSpan,
}

/// Period character (.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeriodToken {
    /// Source span of the period
    pub span: SourceSpan,
}

/// Left bracket character ([)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeftBracketToken {
    /// Source span of the bracket
    pub span: SourceSpan,
}

/// Right bracket character (])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RightBracketToken {
    /// Source span of the bracket
    pub span: SourceSpan,
}

/// At-sign character (@)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtSignToken {
    /// Source span of the at-sign
    pub span: SourceSpan,
}

/// Left parenthesis character (()
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeftParenToken {
    /// Source span of the parenthesis
    pub span: SourceSpan,
}

/// Right parenthesis character ())
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RightParenToken {
    /// Source span of the parenthesis
    pub span: SourceSpan,
}

/// Colon character (:)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColonToken {
    /// Source span of the colon
    pub span: SourceSpan,
}

/// Equals character (=)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualsToken {
    /// Source span of the equals
    pub span: SourceSpan,
}

/// Comma character (,)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommaToken {
    /// Source span of the comma
    pub span: SourceSpan,
}
