//! High-Level Tokens for TXXT parsing pipeline
//!
//! This module defines the high-level token types that bridge the gap between
//! low-level scanner tokens and AST elements. High-level tokens provide a
//! higher-level representation of the syntactic structure of individual lines,
//! making subsequent block parsing simpler and more direct.
//!
//! # Overview
//!
//! High-level tokens describe the *syntactic shape* of content, not its final
//! semantic role (e.g., a paragraph vs. a definition term). They group
//! scanner tokens into meaningful language constructs while preserving
//! structural information like indentation.
//!
//! # Design Principles
//!
//! 1. **Line-based grouping**: Scanner tokens are grouped by lines into
//!    high-level tokens that represent the syntactic structure of each line.
//! 2. **Structural preservation**: Indent, Dedent, and BlankLine tokens
//!    are passed through unchanged to maintain tree structure.
//! 3. **Composability**: High-level tokens are designed to be reusable
//!    components that can be combined in different ways.
//! 4. **Source span tracking**: All high-level tokens preserve source
//!    location information for error reporting and debugging.
//!
//! # Token Types
//!
//! Based on the high-level tokens specification in `docs/specs/core/high-level-tokens.txxt`:
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

use super::primitives::{Position, ScannerTokenSequence, SourceSpan};
use super::scanner_tokens::{ScannerToken, WallType};
use crate::syntax::parameter_parsing::extract_parameters_from_tokens;

/// High-level token representing higher-level syntactic constructs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HighLevelToken {
    /// Structured identifier component for annotations and verbatim blocks
    /// Supports namespaced identifiers like "python", "org.example.custom"
    Label {
        /// The label text (may include namespaces)
        text: String,
        /// Source span of the label
        span: SourceSpan,
        /// Scanner tokens that make up this label
        tokens: ScannerTokenSequence,
    },

    /// Fundamental :: marker used across annotations, definitions, and verbatim blocks
    /// Identifies txxt structural elements and provides disambiguation anchor points
    TxxtMarker {
        /// Source span of the marker
        span: SourceSpan,
        /// Scanner tokens that make up this marker
        tokens: ScannerTokenSequence,
    },

    /// Colon punctuation marker used for parameter separation
    /// Preserves syntactic meaning for parameter parsing and error reporting
    Colon {
        /// Source span of the colon
        span: SourceSpan,
        /// Scanner tokens that make up this colon
        tokens: ScannerTokenSequence,
    },

    /// Key-value metadata component used in annotations and verbatim elements
    /// Structured parameter list with proper key-value pair parsing
    Parameters {
        /// Map of parameter key-value pairs
        params: HashMap<String, String>,
        /// Source span of the entire parameter list
        span: SourceSpan,
        /// Scanner tokens that make up these parameters
        tokens: ScannerTokenSequence,
    },

    /// List and session numbering component
    /// Handles numeric (1.), alphabetic (a.), roman (i.), and plain (-) markers,
    /// in both regular (2.) and extended (1.3.b) forms
    SequenceMarker {
        /// The numbering style (numeric, alphabetic, roman, plain)
        style: HighLevelNumberingStyle,
        /// The numbering form (regular, extended)
        form: HighLevelNumberingForm,
        /// The actual marker text
        marker: String,
        /// Source span of the marker
        span: SourceSpan,
        /// Scanner tokens that make up this marker
        tokens: ScannerTokenSequence,
    },

    /// Basic text content component without special formatting
    /// Building block for larger line constructs
    TextSpan {
        /// The text content
        content: String,
        /// Source span of the text
        span: SourceSpan,
        /// Scanner tokens that make up this text
        tokens: ScannerTokenSequence,
    },

    /// Line beginning with sequence marker followed by text content
    /// Combines Sequence Marker and Text Span components
    SequenceTextLine {
        /// Leading whitespace before the sequence marker (implements the "wall" concept)
        ///
        /// This is the STRUCTURAL indentation padding that positions content at the wall.
        /// It contains the physical whitespace characters that appear after an Indent token
        /// but are not part of the semantic content.
        ///
        /// Values:
        /// - "" (empty) for top-level content (no indentation)
        /// - "    " (4 spaces) for content at indentation level 1
        /// - "        " (8 spaces) for content at indentation level 2
        /// - etc.
        ///
        /// CRITICAL: This field must be populated consistently with the current indentation level.
        /// The parser must NEVER see this whitespace in the content field - it's purely structural.
        ///
        /// See: SemanticAnalyzer::analyze() for implementation details.
        indentation_chars: String,
        /// The sequence marker
        marker: Box<HighLevelToken>,
        /// The text content following the marker
        content: Box<HighLevelToken>,
        /// Source span of the entire line
        span: SourceSpan,
        /// Scanner tokens that make up this line
        tokens: ScannerTokenSequence,
    },

    /// Simple text content without special markers or structure
    /// Contains single Text Span component
    PlainTextLine {
        /// Leading whitespace before the text content (implements the "wall" concept)
        ///
        /// This is the STRUCTURAL indentation padding that positions content at the wall.
        /// It contains the physical whitespace characters that appear after an Indent token
        /// but are not part of the semantic content.
        ///
        /// Values:
        /// - "" (empty) for top-level content (no indentation)
        /// - "    " (4 spaces) for content at indentation level 1
        /// - "        " (8 spaces) for content at indentation level 2
        /// - etc.
        ///
        /// CRITICAL: This field must be populated consistently with the current indentation level.
        /// The parser must NEVER see this whitespace in the content field - it's purely structural.
        ///
        /// See: SemanticAnalyzer::analyze() for implementation details.
        indentation_chars: String,
        /// The text content
        content: Box<HighLevelToken>,
        /// Source span of the entire line
        span: SourceSpan,
        /// Scanner tokens that make up this line
        tokens: ScannerTokenSequence,
    },

    /// Preserved exactly as written without txxt processing
    /// Stored as raw string with source span tracking
    IgnoreLine {
        /// The raw content
        content: String,
        /// Source span of the line
        span: SourceSpan,
        /// Scanner tokens that make up this line
        tokens: ScannerTokenSequence,
    },

    /// Line containing only whitespace or completely empty
    /// Critical for whitespace enclosure detection in sessions vs lists
    BlankLine {
        /// Source span of the blank line
        span: SourceSpan,
        /// Scanner tokens that make up this blank line
        tokens: ScannerTokenSequence,
    },

    /// Indentation marker - passed through unchanged from scanner tokens
    Indent {
        /// Source span of the indent token
        span: SourceSpan,
        /// Scanner tokens that make up this indent
        tokens: ScannerTokenSequence,
    },

    /// Dedentation marker - passed through unchanged from scanner tokens
    Dedent {
        /// Source span of the dedent token
        span: SourceSpan,
        /// Scanner tokens that make up this dedent
        tokens: ScannerTokenSequence,
    },

    /// Annotation semantic token combining txxt markers with labels and optional content
    /// Composition: TxxtMarker + Whitespace + Text + Whitespace + TxxtMarker + Text?
    /// Used for metadata elements that attach structured information to other elements
    Annotation {
        /// The annotation label/type
        label: Box<HighLevelToken>,
        /// Optional parameters in key=value format
        parameters: Option<Box<HighLevelToken>>,
        /// Optional annotation content
        content: Option<Box<HighLevelToken>>,
        /// Source span of the entire annotation
        span: SourceSpan,
        /// Scanner tokens that make up this annotation
        tokens: ScannerTokenSequence,
    },

    /// Definition semantic token combining text with txxt markers
    /// Composition: Text + Whitespace + TxxtMarker
    /// Used for structured elements that define terms, concepts, and entities
    Definition {
        /// The definition term/label
        term: Box<HighLevelToken>,
        /// Optional parameters in key=value format
        parameters: Option<Box<HighLevelToken>>,
        /// Source span of the entire definition
        span: SourceSpan,
        /// Scanner tokens that make up this definition
        tokens: ScannerTokenSequence,
    },

    /// Verbatim block semantic token using wall architecture
    /// Composition: title + wall + content (IgnoreLine/BlankLine) + label + parameters
    /// Used for content that preserves exact formatting and spacing
    VerbatimBlock {
        /// The verbatim title
        title: Box<HighLevelToken>,
        /// The indentation wall marker
        wall: Box<HighLevelToken>,
        /// The verbatim content as a vector of IgnoreLine/BlankLine tokens
        /// This preserves line-level structure for wall-stripping at AST level
        content: Vec<HighLevelToken>,
        /// The verbatim label
        label: Box<HighLevelToken>,
        /// Optional parameters in key=value format
        parameters: Option<Box<HighLevelToken>>,
        /// Wall type (InFlow or Stretched)
        wall_type: WallType,
        /// Source span of the entire verbatim block
        span: SourceSpan,
        /// Scanner tokens that make up this verbatim block
        tokens: ScannerTokenSequence,
    },
}

/// Numbering style for sequence markers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HighLevelNumberingStyle {
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
pub enum HighLevelNumberingForm {
    /// Regular form (1., a., i.)
    Regular,
    /// Extended hierarchical form (1.3.b)
    Extended,
}

/// List structure containing semantic tokens with flat list children
/// Mirrors TokenList structure but with higher-level semantic meaning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighLevelTokenList {
    /// The semantic tokens in order
    pub tokens: Vec<HighLevelToken>,
    /// Source span covering the entire list
    pub source_span: SourceSpan,
}

impl HighLevelTokenList {
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
    pub fn with_tokens(tokens: Vec<HighLevelToken>) -> Self {
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
    pub fn push(&mut self, token: HighLevelToken) {
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
    pub fn iter(&self) -> std::slice::Iter<'_, HighLevelToken> {
        self.tokens.iter()
    }
}

impl Default for HighLevelTokenList {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for getting source span information from semantic tokens
pub trait HighLevelTokenSpan {
    /// Get the source span of this semantic token
    fn span(&self) -> &SourceSpan;
}

impl HighLevelTokenSpan for HighLevelToken {
    fn span(&self) -> &SourceSpan {
        match self {
            HighLevelToken::Label { span, .. }
            | HighLevelToken::TxxtMarker { span, .. }
            | HighLevelToken::Colon { span, .. }
            | HighLevelToken::Parameters { span, .. }
            | HighLevelToken::SequenceMarker { span, .. }
            | HighLevelToken::TextSpan { span, .. }
            | HighLevelToken::SequenceTextLine { span, .. }
            | HighLevelToken::PlainTextLine { span, .. }
            | HighLevelToken::IgnoreLine { span, .. }
            | HighLevelToken::BlankLine { span, .. }
            | HighLevelToken::Indent { span, .. }
            | HighLevelToken::Dedent { span, .. }
            | HighLevelToken::Annotation { span, .. }
            | HighLevelToken::Definition { span, .. }
            | HighLevelToken::VerbatimBlock { span, .. } => span,
        }
    }
}

impl HighLevelToken {
    /// Get the scanner token sequence for this high-level token
    ///
    /// Returns a reference to the token sequence that makes up this high-level token.
    /// All high-level tokens must have their source tokens populated.
    pub fn tokens(&self) -> &ScannerTokenSequence {
        match self {
            HighLevelToken::Label { tokens, .. }
            | HighLevelToken::TxxtMarker { tokens, .. }
            | HighLevelToken::Colon { tokens, .. }
            | HighLevelToken::Parameters { tokens, .. }
            | HighLevelToken::SequenceMarker { tokens, .. }
            | HighLevelToken::TextSpan { tokens, .. }
            | HighLevelToken::SequenceTextLine { tokens, .. }
            | HighLevelToken::PlainTextLine { tokens, .. }
            | HighLevelToken::IgnoreLine { tokens, .. }
            | HighLevelToken::BlankLine { tokens, .. }
            | HighLevelToken::Indent { tokens, .. }
            | HighLevelToken::Dedent { tokens, .. }
            | HighLevelToken::Annotation { tokens, .. }
            | HighLevelToken::Definition { tokens, .. }
            | HighLevelToken::VerbatimBlock { tokens, .. } => tokens,
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

impl FromScannerToken for HighLevelToken {
    fn from_scanner_token(token: &ScannerToken) -> Option<Self> {
        let tokens = ScannerTokenSequence::from_tokens(vec![token.clone()]);
        match token {
            ScannerToken::BlankLine { span, .. } => Some(HighLevelToken::BlankLine {
                span: span.clone(),
                tokens,
            }),
            ScannerToken::Indent { span } => Some(HighLevelToken::Indent {
                span: span.clone(),
                tokens,
            }),
            ScannerToken::Dedent { span } => Some(HighLevelToken::Dedent {
                span: span.clone(),
                tokens,
            }),
            // For now, we'll handle simple cases and expand in later phases
            _ => None,
        }
    }
}

impl ToScannerToken for HighLevelToken {
    fn to_scanner_tokens(&self) -> Vec<ScannerToken> {
        match self {
            HighLevelToken::BlankLine { span, .. } => vec![ScannerToken::BlankLine {
                whitespace: "".to_string(),
                span: span.clone(),
            }],
            HighLevelToken::Indent { span, .. } => {
                vec![ScannerToken::Indent { span: span.clone() }]
            }
            HighLevelToken::Dedent { span, .. } => {
                vec![ScannerToken::Dedent { span: span.clone() }]
            }
            // For now, we'll handle simple cases and expand in later phases
            _ => vec![],
        }
    }
}

/// Builder for creating semantic tokens with proper validation
pub struct HighLevelTokenBuilder;

impl HighLevelTokenBuilder {
    /// Create a label semantic token
    pub fn label(text: String, span: SourceSpan) -> HighLevelToken {
        Self::label_with_tokens(text, span, ScannerTokenSequence::new())
    }

    /// Create a label semantic token with source tokens
    pub fn label_with_tokens(
        text: String,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::Label { text, span, tokens }
    }

    /// Create a txxt marker semantic token
    pub fn txxt_marker(span: SourceSpan) -> HighLevelToken {
        Self::txxt_marker_with_tokens(span, ScannerTokenSequence::new())
    }

    /// Create a txxt marker semantic token with source tokens
    pub fn txxt_marker_with_tokens(
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::TxxtMarker { span, tokens }
    }

    /// Create a colon semantic token
    pub fn colon(span: SourceSpan) -> HighLevelToken {
        Self::colon_with_tokens(span, ScannerTokenSequence::new())
    }

    /// Create a colon semantic token with source tokens
    pub fn colon_with_tokens(span: SourceSpan, tokens: ScannerTokenSequence) -> HighLevelToken {
        HighLevelToken::Colon { span, tokens }
    }

    /// Create a parameters semantic token
    pub fn parameters(params: HashMap<String, String>, span: SourceSpan) -> HighLevelToken {
        Self::parameters_with_tokens(params, span, ScannerTokenSequence::new())
    }

    /// Create a parameters semantic token with source tokens
    pub fn parameters_with_tokens(
        params: HashMap<String, String>,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::Parameters {
            params,
            span,
            tokens,
        }
    }

    /// Build Parameters from scanner tokens
    ///
    /// Takes a sequence of scanner tokens (Identifier, Equals, Text/QuotedString, Comma)
    /// and assembles them into a Parameters high-level token.
    ///
    /// # Parameter Processing Flow
    ///
    /// This is **Step 2** of the three-level parameter processing:
    ///
    /// ```text
    /// Step 1 (Scanner): "key=value,key2=val2"
    ///        → [Identifier("key"), Equals, Text("value"), Comma, ...]
    ///
    /// Step 2 (This function): [Identifier("key"), Equals, Text("value"), ...]
    ///        → Parameters { map: {key: "value", key2: "val2"} }
    ///
    /// Step 3 (AST): Parameters { map: {...} }
    ///        → AstParameters { map: {...}, tokens: ... }
    /// ```
    ///
    /// See: [`crate::cst::parameter_scanner`] for Scanner level (Step 1)
    ///
    /// # Arguments
    /// * `scanner_tokens` - Slice of scanner tokens representing parameters
    ///
    /// # Returns
    /// A Parameters high-level token with parsed key-value pairs, or None if invalid
    pub fn parameters_from_scanner_tokens(
        scanner_tokens: &[ScannerToken],
    ) -> Option<HighLevelToken> {
        if scanner_tokens.is_empty() {
            return None;
        }

        // Extract parameters using extracted function
        let params = extract_parameters_from_tokens(scanner_tokens);

        if params.is_empty() {
            return None;
        }

        // Calculate span
        let start_span = scanner_tokens.first()?.span();
        let end_span = scanner_tokens.last()?.span();
        let span = SourceSpan {
            start: start_span.start,
            end: end_span.end,
        };

        Some(HighLevelToken::Parameters {
            params,
            span,
            tokens: ScannerTokenSequence::from_tokens(scanner_tokens.to_vec()),
        })
    }

    /// Create a sequence marker semantic token
    pub fn sequence_marker(
        style: HighLevelNumberingStyle,
        form: HighLevelNumberingForm,
        marker: String,
        span: SourceSpan,
    ) -> HighLevelToken {
        Self::sequence_marker_with_tokens(style, form, marker, span, ScannerTokenSequence::new())
    }

    /// Create a sequence marker semantic token with source tokens
    pub fn sequence_marker_with_tokens(
        style: HighLevelNumberingStyle,
        form: HighLevelNumberingForm,
        marker: String,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::SequenceMarker {
            style,
            form,
            marker,
            span,
            tokens,
        }
    }

    /// Create a text span semantic token (TEST ONLY - DO NOT USE IN PRODUCTION)
    ///
    /// **WARNING**: This function creates synthetic tokens and should ONLY be used
    /// in tests. Production code MUST use text_span_with_tokens() to ensure proper
    /// token tracking for language server features.
    ///
    /// Using this in production will result in incorrect source position tracking.
    #[doc(hidden)]
    #[deprecated(note = "TEST ONLY: Use text_span_with_tokens() in production code")]
    pub fn text_span(content: String, span: SourceSpan) -> HighLevelToken {
        Self::text_span_with_tokens(
            content.clone(),
            span.clone(),
            ScannerTokenSequence {
                tokens: vec![ScannerToken::Text { content, span }],
            },
        )
    }

    /// Create a text span semantic token with source tokens
    ///
    /// This is the ONLY way to create a TextSpan HighLevelToken in production.
    /// All text must have associated scanner tokens for accurate source position tracking.
    pub fn text_span_with_tokens(
        content: String,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::TextSpan {
            content,
            span,
            tokens,
        }
    }

    /// Create a plain text line semantic token
    ///
    /// # Arguments
    /// * `indentation_chars` - Leading whitespace before content (the "wall" padding).
    ///                         Empty string for top-level content, "    " for indented content, etc.
    ///                         This MUST match the physical spaces after an Indent token.
    /// * `content` - The actual line content (starts at the wall, no leading spaces)
    /// * `span` - Source span covering the entire line
    ///
    /// # The Wall Concept
    ///
    /// The `indentation_chars` field implements the "wall" architecture for indented content.
    /// It separates STRUCTURAL indentation (padding) from SEMANTIC content.
    ///
    /// Example:
    /// ```text
    /// Input:  "    This is indented text"
    ///          ^^^^--- indentation_chars (structural)
    ///              ^^^^^^^^^^^^^^^^^^^--- content (semantic)
    /// ```
    ///
    /// This ensures the parser never sees structural whitespace in content,
    /// while preserving exact source positions for error reporting and LSP features.
    pub fn plain_text_line(
        indentation_chars: String,
        content: HighLevelToken,
        span: SourceSpan,
    ) -> HighLevelToken {
        Self::plain_text_line_with_tokens(
            indentation_chars,
            content,
            span,
            ScannerTokenSequence::new(),
        )
    }

    /// Create a plain text line semantic token with source tokens
    pub fn plain_text_line_with_tokens(
        indentation_chars: String,
        content: HighLevelToken,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::PlainTextLine {
            indentation_chars,
            content: Box::new(content),
            span,
            tokens,
        }
    }

    /// Create a sequence text line semantic token
    ///
    /// # Arguments
    /// * `indentation_chars` - Leading whitespace before the sequence marker (the "wall" padding).
    ///                         Empty string for top-level content, "    " for indented content, etc.
    ///                         This MUST match the physical spaces after an Indent token.
    /// * `marker` - The sequence marker (1., -, a., etc.)
    /// * `content` - The line content following the marker
    /// * `span` - Source span covering the entire line
    ///
    /// # The Wall Concept
    ///
    /// The `indentation_chars` field implements the "wall" architecture for indented content.
    /// It separates STRUCTURAL indentation (padding) from SEMANTIC content.
    ///
    /// Example:
    /// ```text
    /// Input:  "    - List item"
    ///          ^^^^--- indentation_chars (structural)
    ///              ^----------- marker
    ///                ^^^^^^^^^^--- content (semantic)
    /// ```
    ///
    /// This ensures the parser never sees structural whitespace in content,
    /// while preserving exact source positions for error reporting and LSP features.
    pub fn sequence_text_line(
        indentation_chars: String,
        marker: HighLevelToken,
        content: HighLevelToken,
        span: SourceSpan,
    ) -> HighLevelToken {
        Self::sequence_text_line_with_tokens(
            indentation_chars,
            marker,
            content,
            span,
            ScannerTokenSequence::new(),
        )
    }

    /// Create a sequence text line semantic token with source tokens
    pub fn sequence_text_line_with_tokens(
        indentation_chars: String,
        marker: HighLevelToken,
        content: HighLevelToken,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::SequenceTextLine {
            indentation_chars,
            marker: Box::new(marker),
            content: Box::new(content),
            span,
            tokens,
        }
    }

    /// Create an ignore line semantic token
    pub fn ignore_line(content: String, span: SourceSpan) -> HighLevelToken {
        Self::ignore_line_with_tokens(content, span, ScannerTokenSequence::new())
    }

    /// Create an ignore line semantic token with source tokens
    pub fn ignore_line_with_tokens(
        content: String,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::IgnoreLine {
            content,
            span,
            tokens,
        }
    }

    /// Create a blank line semantic token
    pub fn blank_line(span: SourceSpan) -> HighLevelToken {
        Self::blank_line_with_tokens(span, ScannerTokenSequence::new())
    }

    /// Create a blank line semantic token with source tokens
    pub fn blank_line_with_tokens(
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::BlankLine { span, tokens }
    }

    /// Create an indent semantic token
    pub fn indent(span: SourceSpan) -> HighLevelToken {
        Self::indent_with_tokens(span, ScannerTokenSequence::new())
    }

    /// Create an indent semantic token with source tokens
    pub fn indent_with_tokens(span: SourceSpan, tokens: ScannerTokenSequence) -> HighLevelToken {
        HighLevelToken::Indent { span, tokens }
    }

    /// Create a dedent semantic token
    pub fn dedent(span: SourceSpan) -> HighLevelToken {
        Self::dedent_with_tokens(span, ScannerTokenSequence::new())
    }

    /// Create a dedent semantic token with source tokens
    pub fn dedent_with_tokens(span: SourceSpan, tokens: ScannerTokenSequence) -> HighLevelToken {
        HighLevelToken::Dedent { span, tokens }
    }

    /// Create an annotation semantic token
    pub fn annotation(
        label: HighLevelToken,
        parameters: Option<HighLevelToken>,
        content: Option<HighLevelToken>,
        span: SourceSpan,
    ) -> HighLevelToken {
        Self::annotation_with_tokens(
            label,
            parameters,
            content,
            span,
            ScannerTokenSequence::new(),
        )
    }

    /// Create an annotation semantic token with source tokens
    pub fn annotation_with_tokens(
        label: HighLevelToken,
        parameters: Option<HighLevelToken>,
        content: Option<HighLevelToken>,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::Annotation {
            label: Box::new(label),
            parameters: parameters.map(Box::new),
            content: content.map(Box::new),
            span,
            tokens,
        }
    }

    /// Create a definition semantic token
    pub fn definition(
        term: HighLevelToken,
        parameters: Option<HighLevelToken>,
        span: SourceSpan,
    ) -> HighLevelToken {
        Self::definition_with_tokens(term, parameters, span, ScannerTokenSequence::new())
    }

    /// Create a definition semantic token with source tokens
    pub fn definition_with_tokens(
        term: HighLevelToken,
        parameters: Option<HighLevelToken>,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::Definition {
            term: Box::new(term),
            parameters: parameters.map(Box::new),
            span,
            tokens,
        }
    }

    /// Create a verbatim block semantic token
    pub fn verbatim_block(
        title: HighLevelToken,
        wall: HighLevelToken,
        content: Vec<HighLevelToken>,
        label: HighLevelToken,
        parameters: Option<HighLevelToken>,
        wall_type: WallType,
        span: SourceSpan,
    ) -> HighLevelToken {
        Self::verbatim_block_with_tokens(
            title,
            wall,
            content,
            label,
            parameters,
            wall_type,
            span,
            ScannerTokenSequence::new(),
        )
    }

    /// Create a verbatim block semantic token with source tokens
    #[allow(clippy::too_many_arguments)]
    pub fn verbatim_block_with_tokens(
        title: HighLevelToken,
        wall: HighLevelToken,
        content: Vec<HighLevelToken>,
        label: HighLevelToken,
        parameters: Option<HighLevelToken>,
        wall_type: WallType,
        span: SourceSpan,
        tokens: ScannerTokenSequence,
    ) -> HighLevelToken {
        HighLevelToken::VerbatimBlock {
            title: Box::new(title),
            wall: Box::new(wall),
            content,
            label: Box::new(label),
            parameters: parameters.map(Box::new),
            wall_type,
            span,
            tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameters_from_scanner_tokens_simple() {
        let tokens = vec![
            ScannerToken::Identifier {
                content: "key".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 3 },
                },
            },
            ScannerToken::Equals {
                span: SourceSpan {
                    start: Position { row: 0, column: 3 },
                    end: Position { row: 0, column: 4 },
                },
            },
            ScannerToken::Text {
                content: "value".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 4 },
                    end: Position { row: 0, column: 9 },
                },
            },
        ];

        let result = HighLevelTokenBuilder::parameters_from_scanner_tokens(&tokens);
        assert!(result.is_some());

        if let Some(HighLevelToken::Parameters { params, .. }) = result {
            assert_eq!(params.get("key"), Some(&"value".to_string()));
        } else {
            panic!("Expected Parameters token");
        }
    }

    #[test]
    fn test_parameters_from_scanner_tokens_quoted() {
        let tokens = vec![
            ScannerToken::Identifier {
                content: "title".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 5 },
                },
            },
            ScannerToken::Equals {
                span: SourceSpan {
                    start: Position { row: 0, column: 5 },
                    end: Position { row: 0, column: 6 },
                },
            },
            ScannerToken::QuotedString {
                content: "My Document".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 6 },
                    end: Position { row: 0, column: 19 },
                },
            },
        ];

        let result = HighLevelTokenBuilder::parameters_from_scanner_tokens(&tokens);
        assert!(result.is_some());

        if let Some(HighLevelToken::Parameters { params, .. }) = result {
            assert_eq!(params.get("title"), Some(&"My Document".to_string()));
        } else {
            panic!("Expected Parameters token");
        }
    }

    #[test]
    fn test_parameters_from_scanner_tokens_multiple() {
        let tokens = vec![
            ScannerToken::Identifier {
                content: "key1".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 4 },
                },
            },
            ScannerToken::Equals {
                span: SourceSpan {
                    start: Position { row: 0, column: 4 },
                    end: Position { row: 0, column: 5 },
                },
            },
            ScannerToken::Text {
                content: "value1".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 5 },
                    end: Position { row: 0, column: 11 },
                },
            },
            ScannerToken::Comma {
                span: SourceSpan {
                    start: Position { row: 0, column: 11 },
                    end: Position { row: 0, column: 12 },
                },
            },
            ScannerToken::Identifier {
                content: "key2".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 12 },
                    end: Position { row: 0, column: 16 },
                },
            },
            ScannerToken::Equals {
                span: SourceSpan {
                    start: Position { row: 0, column: 16 },
                    end: Position { row: 0, column: 17 },
                },
            },
            ScannerToken::Text {
                content: "value2".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 17 },
                    end: Position { row: 0, column: 23 },
                },
            },
        ];

        let result = HighLevelTokenBuilder::parameters_from_scanner_tokens(&tokens);
        assert!(result.is_some());

        if let Some(HighLevelToken::Parameters { params, .. }) = result {
            assert_eq!(params.get("key1"), Some(&"value1".to_string()));
            assert_eq!(params.get("key2"), Some(&"value2".to_string()));
        } else {
            panic!("Expected Parameters token");
        }
    }

    #[test]
    fn test_parameters_from_scanner_tokens_boolean() {
        let tokens = vec![ScannerToken::Identifier {
            content: "debug".to_string(),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 5 },
            },
        }];

        let result = HighLevelTokenBuilder::parameters_from_scanner_tokens(&tokens);
        assert!(result.is_some());

        if let Some(HighLevelToken::Parameters { params, .. }) = result {
            assert_eq!(params.get("debug"), Some(&"true".to_string()));
        } else {
            panic!("Expected Parameters token");
        }
    }
}
