//! Semantic Tokens for TXXT parsing pipeline
//!
//! This module defines the semantic token types that bridge the gap between
//! low-level scanner tokens and AST elements. Semantic tokens provide a
//! higher-level representation of the syntactic structure of individual lines,
//! making subsequent block parsing simpler and more direct.
//!
//! # Overview
//!
//! Semantic tokens describe the *syntactic shape* of content, not its final
//! semantic role (e.g., a paragraph vs. a definition term). They group
//! scanner tokens into meaningful language constructs while preserving
//! structural information like indentation.
//!
//! # Design Principles
//!
//! 1. **Line-based grouping**: Scanner tokens are grouped by lines into
//!    semantic tokens that represent the syntactic structure of each line.
//! 2. **Structural preservation**: Indent, Dedent, and BlankLine tokens
//!    are passed through unchanged to maintain tree structure.
//! 3. **Composability**: Semantic tokens are designed to be reusable
//!    components that can be combined in different ways.
//! 4. **Source span tracking**: All semantic tokens preserve source
//!    location information for error reporting and debugging.
//!
//! # Token Types
//!
//! Based on the semantic tokens specification in `docs/specs/core/semantic-tokens.txxt`:
//!
//! - `TxxtMarker`: The fundamental :: marker for structural elements
//! - `Label`: Structured identifier component (supports namespacing)
//! - `Parameters`: Key-value metadata component
//! - `SequenceMarker`: List and session numbering component
//! - `TextSpan`: Basic text content without formatting
//! - `SequenceTextLine`: Line with sequence marker + text content
//! - `PlainTextLine`: Simple text content without markers
//! - `IgnoreLine`: Raw content preserved exactly as written
//! - `BlankLine`: Empty or whitespace-only line
//! - `Indent`/`Dedent`: Structural indentation markers

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};

/// Semantic token representing higher-level syntactic constructs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SemanticToken {
    /// The fundamental :: marker used across annotations, definitions, and verbatim blocks
    /// Identifies txxt structural elements and provides disambiguation anchor points
    TxxtMarker {
        /// Source span of the marker
        span: SourceSpan,
    },

    /// Structured identifier component for annotations and verbatim blocks
    /// Supports namespaced identifiers like "python", "org.example.custom"
    Label {
        /// The label text (may include namespaces)
        text: String,
        /// Source span of the label
        span: SourceSpan,
    },

    /// Key-value metadata component used in annotations and verbatim elements
    /// Structured parameter list with proper key-value pair parsing
    Parameters {
        /// Map of parameter key-value pairs
        params: HashMap<String, String>,
        /// Source span of the entire parameter list
        span: SourceSpan,
    },

    /// List and session numbering component
    /// Handles numeric (1.), alphabetic (a.), roman (i.), and plain (-) markers,
    /// in both regular (2.) and extended (1.3.b) forms
    SequenceMarker {
        /// The numbering style (numeric, alphabetic, roman, plain)
        style: SemanticNumberingStyle,
        /// The numbering form (regular, extended)
        form: SemanticNumberingForm,
        /// The actual marker text
        marker: String,
        /// Source span of the marker
        span: SourceSpan,
    },

    /// Basic text content component without special formatting
    /// Building block for larger line constructs
    TextSpan {
        /// The text content
        content: String,
        /// Source span of the text
        span: SourceSpan,
    },

    /// Line beginning with sequence marker followed by text content
    /// Combines Sequence Marker and Text Span components
    SequenceTextLine {
        /// The sequence marker
        marker: Box<SemanticToken>,
        /// The text content following the marker
        content: Box<SemanticToken>,
        /// Source span of the entire line
        span: SourceSpan,
    },

    /// Simple text content without special markers or structure
    /// Contains single Text Span component
    PlainTextLine {
        /// The text content
        content: Box<SemanticToken>,
        /// Source span of the entire line
        span: SourceSpan,
    },

    /// Preserved exactly as written without txxt processing
    /// Stored as raw string with source span tracking
    IgnoreLine {
        /// The raw content
        content: String,
        /// Source span of the line
        span: SourceSpan,
    },

    /// Line containing only whitespace or completely empty
    /// Critical for whitespace enclosure detection in sessions vs lists
    BlankLine {
        /// Source span of the blank line
        span: SourceSpan,
    },

    /// Indentation marker - passed through unchanged from scanner tokens
    Indent {
        /// Source span of the indent token
        span: SourceSpan,
    },

    /// Dedentation marker - passed through unchanged from scanner tokens
    Dedent {
        /// Source span of the dedent token
        span: SourceSpan,
    },
}

/// Numbering style for sequence markers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SemanticNumberingStyle {
    /// Numeric numbering (1., 2., 3.)
    Numeric,
    /// Alphabetic numbering (a., b., c.)
    Alphabetic,
    /// Roman numeral numbering (i., ii., iii.)
    Roman,
    /// Plain dash numbering (-)
    Plain,
}

/// Numbering form for sequence markers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SemanticNumberingForm {
    /// Regular form (1., a., i.)
    Regular,
    /// Extended hierarchical form (1.3.b)
    Extended,
}

/// List structure containing semantic tokens with flat list children
/// Mirrors TokenList structure but with higher-level semantic meaning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticTokenList {
    /// The semantic tokens in order
    pub tokens: Vec<SemanticToken>,
    /// Source span covering the entire list
    pub source_span: SourceSpan,
}

impl SemanticTokenList {
    /// Create a new empty semantic token list
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            source_span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 0 },
            },
        }
    }

    /// Create a semantic token list with the given tokens
    pub fn with_tokens(tokens: Vec<SemanticToken>) -> Self {
        let source_span = if tokens.is_empty() {
            SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 0 },
            }
        } else {
            let start = tokens.first().unwrap().span().start;
            let end = tokens.last().unwrap().span().end;
            SourceSpan { start, end }
        };

        Self {
            tokens,
            source_span,
        }
    }

    /// Add a semantic token to the list
    pub fn push(&mut self, token: SemanticToken) {
        self.tokens.push(token);
        // Update source span to include the new token
        if self.tokens.len() == 1 {
            self.source_span = self.tokens[0].span().clone();
        } else {
            let start = self.source_span.start;
            let end = self.tokens.last().unwrap().span().end;
            self.source_span = SourceSpan { start, end };
        }
    }

    /// Get the number of tokens in the list
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Get an iterator over the tokens
    pub fn iter(&self) -> std::slice::Iter<'_, SemanticToken> {
        self.tokens.iter()
    }
}

impl Default for SemanticTokenList {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for getting source span information from semantic tokens
pub trait SemanticTokenSpan {
    /// Get the source span of this semantic token
    fn span(&self) -> &SourceSpan;
}

impl SemanticTokenSpan for SemanticToken {
    fn span(&self) -> &SourceSpan {
        match self {
            SemanticToken::TxxtMarker { span }
            | SemanticToken::Label { span, .. }
            | SemanticToken::Parameters { span, .. }
            | SemanticToken::SequenceMarker { span, .. }
            | SemanticToken::TextSpan { span, .. }
            | SemanticToken::SequenceTextLine { span, .. }
            | SemanticToken::PlainTextLine { span, .. }
            | SemanticToken::IgnoreLine { span, .. }
            | SemanticToken::BlankLine { span }
            | SemanticToken::Indent { span }
            | SemanticToken::Dedent { span } => span,
        }
    }
}

/// Conversion trait from scanner tokens to semantic tokens
pub trait FromScannerToken {
    /// Convert a scanner token to a semantic token
    fn from_scanner_token(token: &ScannerToken) -> Option<Self>
    where
        Self: Sized;
}

/// Conversion trait from semantic tokens to scanner tokens
pub trait ToScannerToken {
    /// Convert a semantic token to scanner tokens
    fn to_scanner_tokens(&self) -> Vec<ScannerToken>;
}

impl FromScannerToken for SemanticToken {
    fn from_scanner_token(token: &ScannerToken) -> Option<Self> {
        match token {
            ScannerToken::AnnotationMarker { span, .. } => {
                Some(SemanticToken::TxxtMarker { span: span.clone() })
            }
            ScannerToken::DefinitionMarker { span, .. } => {
                Some(SemanticToken::TxxtMarker { span: span.clone() })
            }
            ScannerToken::BlankLine { span, .. } => {
                Some(SemanticToken::BlankLine { span: span.clone() })
            }
            ScannerToken::Indent { span } => Some(SemanticToken::Indent { span: span.clone() }),
            ScannerToken::Dedent { span } => Some(SemanticToken::Dedent { span: span.clone() }),
            // For now, we'll handle simple cases and expand in later phases
            _ => None,
        }
    }
}

impl ToScannerToken for SemanticToken {
    fn to_scanner_tokens(&self) -> Vec<ScannerToken> {
        match self {
            SemanticToken::TxxtMarker { span } => vec![ScannerToken::AnnotationMarker {
                content: "::".to_string(),
                span: span.clone(),
            }],
            SemanticToken::BlankLine { span } => vec![ScannerToken::BlankLine {
                whitespace: "".to_string(),
                span: span.clone(),
            }],
            SemanticToken::Indent { span } => vec![ScannerToken::Indent { span: span.clone() }],
            SemanticToken::Dedent { span } => vec![ScannerToken::Dedent { span: span.clone() }],
            // For now, we'll handle simple cases and expand in later phases
            _ => vec![],
        }
    }
}

/// Builder for creating semantic tokens with proper validation
pub struct SemanticTokenBuilder;

impl SemanticTokenBuilder {
    /// Create a txxt marker semantic token
    pub fn txxt_marker(span: SourceSpan) -> SemanticToken {
        SemanticToken::TxxtMarker { span }
    }

    /// Create a label semantic token
    pub fn label(text: String, span: SourceSpan) -> SemanticToken {
        SemanticToken::Label { text, span }
    }

    /// Create a parameters semantic token
    pub fn parameters(params: HashMap<String, String>, span: SourceSpan) -> SemanticToken {
        SemanticToken::Parameters { params, span }
    }

    /// Create a sequence marker semantic token
    pub fn sequence_marker(
        style: SemanticNumberingStyle,
        form: SemanticNumberingForm,
        marker: String,
        span: SourceSpan,
    ) -> SemanticToken {
        SemanticToken::SequenceMarker {
            style,
            form,
            marker,
            span,
        }
    }

    /// Create a text span semantic token
    pub fn text_span(content: String, span: SourceSpan) -> SemanticToken {
        SemanticToken::TextSpan { content, span }
    }

    /// Create a plain text line semantic token
    pub fn plain_text_line(content: SemanticToken, span: SourceSpan) -> SemanticToken {
        SemanticToken::PlainTextLine {
            content: Box::new(content),
            span,
        }
    }

    /// Create an ignore line semantic token
    pub fn ignore_line(content: String, span: SourceSpan) -> SemanticToken {
        SemanticToken::IgnoreLine { content, span }
    }

    /// Create a blank line semantic token
    pub fn blank_line(span: SourceSpan) -> SemanticToken {
        SemanticToken::BlankLine { span }
    }

    /// Create an indent semantic token
    pub fn indent(span: SourceSpan) -> SemanticToken {
        SemanticToken::Indent { span }
    }

    /// Create a dedent semantic token
    pub fn dedent(span: SourceSpan) -> SemanticToken {
        SemanticToken::Dedent { span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::scanner_tokens::Position;

    #[test]
    fn test_semantic_token_list_creation() {
        let list = SemanticTokenList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_semantic_token_list_with_tokens() {
        let span = SourceSpan {
            start: Position { row: 1, column: 1 },
            end: Position { row: 1, column: 2 },
        };
        let token = SemanticToken::TxxtMarker { span: span.clone() };
        let list = SemanticTokenList::with_tokens(vec![token.clone()]);

        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert_eq!(list.source_span, span);
    }

    #[test]
    fn test_semantic_token_list_push() {
        let mut list = SemanticTokenList::new();
        let span = SourceSpan {
            start: Position { row: 1, column: 1 },
            end: Position { row: 1, column: 2 },
        };
        let token = SemanticToken::TxxtMarker { span };

        list.push(token);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_from_scanner_token_conversion() {
        let span = SourceSpan {
            start: Position { row: 1, column: 1 },
            end: Position { row: 1, column: 2 },
        };
        let scanner_token = ScannerToken::AnnotationMarker {
            content: "::".to_string(),
            span: span.clone(),
        };

        let semantic_token = SemanticToken::from_scanner_token(&scanner_token);
        assert!(semantic_token.is_some());

        let semantic_token = semantic_token.unwrap();
        match semantic_token {
            SemanticToken::TxxtMarker { span: token_span } => {
                assert_eq!(token_span, span);
            }
            _ => panic!("Expected TxxtMarker"),
        }
    }

    #[test]
    fn test_to_scanner_token_conversion() {
        let span = SourceSpan {
            start: Position { row: 1, column: 1 },
            end: Position { row: 1, column: 2 },
        };
        let semantic_token = SemanticToken::TxxtMarker { span: span.clone() };

        let scanner_tokens = semantic_token.to_scanner_tokens();
        assert_eq!(scanner_tokens.len(), 1);

        match &scanner_tokens[0] {
            ScannerToken::AnnotationMarker {
                span: token_span, ..
            } => {
                assert_eq!(token_span, &span);
            }
            _ => panic!("Expected AnnotationMarker"),
        }
    }

    #[test]
    fn test_semantic_token_builder() {
        let span = SourceSpan {
            start: Position { row: 1, column: 1 },
            end: Position { row: 1, column: 2 },
        };

        let txxt_marker = SemanticTokenBuilder::txxt_marker(span.clone());
        match txxt_marker {
            SemanticToken::TxxtMarker { span: token_span } => {
                assert_eq!(token_span, span);
            }
            _ => panic!("Expected TxxtMarker"),
        }

        let label = SemanticTokenBuilder::label("test".to_string(), span.clone());
        match label {
            SemanticToken::Label {
                text,
                span: token_span,
            } => {
                assert_eq!(text, "test");
                assert_eq!(token_span, span);
            }
            _ => panic!("Expected Label"),
        }
    }

    #[test]
    fn test_semantic_numbering_styles() {
        assert_eq!(
            SemanticNumberingStyle::Numeric,
            SemanticNumberingStyle::Numeric
        );
        assert_eq!(
            SemanticNumberingStyle::Alphabetic,
            SemanticNumberingStyle::Alphabetic
        );
        assert_eq!(SemanticNumberingStyle::Roman, SemanticNumberingStyle::Roman);
        assert_eq!(SemanticNumberingStyle::Plain, SemanticNumberingStyle::Plain);
    }

    #[test]
    fn test_semantic_numbering_forms() {
        assert_eq!(
            SemanticNumberingForm::Regular,
            SemanticNumberingForm::Regular
        );
        assert_eq!(
            SemanticNumberingForm::Extended,
            SemanticNumberingForm::Extended
        );
    }

    #[test]
    fn test_serialization() {
        let span = SourceSpan {
            start: Position { row: 1, column: 1 },
            end: Position { row: 1, column: 2 },
        };
        let token = SemanticToken::TxxtMarker { span };

        let serialized = serde_json::to_string(&token).unwrap();
        let deserialized: SemanticToken = serde_json::from_str(&serialized).unwrap();

        assert_eq!(token, deserialized);
    }
}
