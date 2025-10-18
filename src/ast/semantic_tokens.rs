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
    /// Structured identifier component for annotations and verbatim blocks
    /// Supports namespaced identifiers like "python", "org.example.custom"
    Label {
        /// The label text (may include namespaces)
        text: String,
        /// Source span of the label
        span: SourceSpan,
    },

    /// Fundamental :: marker used across annotations, definitions, and verbatim blocks
    /// Identifies txxt structural elements and provides disambiguation anchor points
    TxxtMarker {
        /// Source span of the marker
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

    /// Annotation semantic token combining txxt markers with labels and optional content
    /// Composition: TxxtMarker + Whitespace + Text + Whitespace + TxxtMarker + Text?
    /// Used for metadata elements that attach structured information to other elements
    Annotation {
        /// The annotation label/type
        label: Box<SemanticToken>,
        /// Optional parameters in key=value format
        parameters: Option<Box<SemanticToken>>,
        /// Optional annotation content
        content: Option<Box<SemanticToken>>,
        /// Source span of the entire annotation
        span: SourceSpan,
    },

    /// Definition semantic token combining text with txxt markers
    /// Composition: Text + Whitespace + TxxtMarker
    /// Used for structured elements that define terms, concepts, and entities
    Definition {
        /// The definition term/label
        term: Box<SemanticToken>,
        /// Optional parameters in key=value format
        parameters: Option<Box<SemanticToken>>,
        /// Source span of the entire definition
        span: SourceSpan,
    },

    /// Verbatim block semantic token using wall architecture
    /// Composition: VerbatimTitle + IndentationWall + IgnoreTextSpan + VerbatimLabel
    /// Used for content that preserves exact formatting and spacing
    VerbatimBlock {
        /// The verbatim title
        title: Box<SemanticToken>,
        /// The indentation wall marker
        wall: Box<SemanticToken>,
        /// The verbatim content (preserved exactly)
        content: Box<SemanticToken>,
        /// The verbatim label
        label: Box<SemanticToken>,
        /// Optional parameters in key=value format
        parameters: Option<Box<SemanticToken>>,
        /// Source span of the entire verbatim block
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
            SemanticToken::Label { span, .. }
            | SemanticToken::TxxtMarker { span }
            | SemanticToken::Parameters { span, .. }
            | SemanticToken::SequenceMarker { span, .. }
            | SemanticToken::TextSpan { span, .. }
            | SemanticToken::SequenceTextLine { span, .. }
            | SemanticToken::PlainTextLine { span, .. }
            | SemanticToken::IgnoreLine { span, .. }
            | SemanticToken::BlankLine { span }
            | SemanticToken::Indent { span }
            | SemanticToken::Dedent { span }
            | SemanticToken::Annotation { span, .. }
            | SemanticToken::Definition { span, .. }
            | SemanticToken::VerbatimBlock { span, .. } => span,
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
    /// Create a label semantic token
    pub fn label(text: String, span: SourceSpan) -> SemanticToken {
        SemanticToken::Label { text, span }
    }

    /// Create a txxt marker semantic token
    pub fn txxt_marker(span: SourceSpan) -> SemanticToken {
        SemanticToken::TxxtMarker { span }
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

    /// Create a sequence text line semantic token
    pub fn sequence_text_line(
        marker: SemanticToken,
        content: SemanticToken,
        span: SourceSpan,
    ) -> SemanticToken {
        SemanticToken::SequenceTextLine {
            marker: Box::new(marker),
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

    /// Create an annotation semantic token
    pub fn annotation(
        label: SemanticToken,
        parameters: Option<SemanticToken>,
        content: Option<SemanticToken>,
        span: SourceSpan,
    ) -> SemanticToken {
        SemanticToken::Annotation {
            label: Box::new(label),
            parameters: parameters.map(Box::new),
            content: content.map(Box::new),
            span,
        }
    }

    /// Create a definition semantic token
    pub fn definition(
        term: SemanticToken,
        parameters: Option<SemanticToken>,
        span: SourceSpan,
    ) -> SemanticToken {
        SemanticToken::Definition {
            term: Box::new(term),
            parameters: parameters.map(Box::new),
            span,
        }
    }

    /// Create a verbatim block semantic token
    pub fn verbatim_block(
        title: SemanticToken,
        wall: SemanticToken,
        content: SemanticToken,
        label: SemanticToken,
        parameters: Option<SemanticToken>,
        span: SourceSpan,
    ) -> SemanticToken {
        SemanticToken::VerbatimBlock {
            title: Box::new(title),
            wall: Box::new(wall),
            content: Box::new(content),
            label: Box::new(label),
            parameters: parameters.map(Box::new),
            span,
        }
    }
}
