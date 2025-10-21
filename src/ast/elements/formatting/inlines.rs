//! Inline elements with text-transform layer
//!
//! This module implements the uniform text transform architecture where
//! every piece of text goes through a transform layer for consistent
//! processing across the AST.
//!
//! src/parser/mod.rs has the full architecture overview.
//!
//! ## Text Transform Processing (2.b)
//!
//! Every piece of text goes through the transform layer during parsing:
//! 1. **Token sequences** are analyzed for formatting markers
//! 2. **Transform nesting** is resolved for complex formatting
//! 3. **Semantic structure** is built with token-level precision preserved
//! 4. **Uniform processing** ensures consistent handling across all contexts
//!
//! This architecture enables powerful text processing while maintaining the
//! character-level precision needed for language server features.
//!
//! ## Reference Integration (2.b)
//!
//! References use the comprehensive ReferenceTarget system to handle all types:
//! - **File references**: `[./filename.txxt]`, `[../dir/file.txxt]`  
//! - **Section references**: `[#3]`, `[#2.1]`, `[local-section]`
//! - **Citation references**: `[@smith2023]`, `[@doe2024; @jones2025]`
//! - **Named anchor references**: `[#hello-world]` (via ref= parameters)
//! - **Naked numerical references**: `[1]`, `[2]` (shorthand for footnotes)
//! - **URL references**: `[example.com]`, `[https://example.com]`
//!
//! The ReferenceTarget system provides complete type information even if the
//! parser doesn't fully resolve all reference types yet.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::cst::{Position, ScannerToken, ScannerTokenSequence, SourceSpan};

/// Inline elements with text-transform layer
///
/// Architecture: Every text goes through a transform layer for uniform processing.
/// This provides:
/// - Consistent handling of text content across all inline types
/// - Extensibility for new transform types without changing core structure
/// - Composability for nested formatting (e.g., **_bold italic_**)
/// - Language server precision through token sequences
///
/// Examples:
/// - "banana" → `TextLine(Identity(Text("banana")))`
/// - "*important*" → `TextLine(Emphasis(vec![Identity(Text("important"))]))`
/// - "**_both_**" → `TextLine(Strong(vec![Emphasis(vec![Identity(Text("both"))])]))`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Inline {
    /// Text content with transform layer - the core inline element
    TextLine(TextTransform),

    /// Link to external resource or internal reference
    Link {
        /// Link target (URL, file path, etc.)
        target: String,

        /// Link text content (can contain formatting)
        content: Vec<Inline>,

        /// Additional link attributes (title, class, etc.)
        attributes: HashMap<String, String>,

        /// Raw tokens for precise positioning
        tokens: ScannerTokenSequence,
    },

    /// Reference to document elements (citations, cross-refs, etc.)
    /// Examples: [filename.txxt], [#section], [@smith2023], [#hello-world], [1]
    Reference(crate::ast::elements::references::Reference),

    /// Future extensibility for custom inline types
    Custom {
        /// Custom inline type name
        name: String,

        /// Custom attributes
        attributes: HashMap<String, String>,

        /// Custom content
        content: Vec<Inline>,

        /// Raw tokens for positioning
        tokens: ScannerTokenSequence,
    },
}

/// Text transform layer - uniform processing for all text content
///
/// This is the key architectural innovation that provides uniform text handling.
/// Every piece of text in the document goes through this transform layer,
/// enabling:
///
/// 1. **Uniform processing**: All text follows the same code paths
/// 2. **Extensibility**: New transforms can be added without changing existing code
/// 3. **Composability**: Transforms can be nested and combined
/// 4. **Performance**: Identity transforms can be optimized during rendering
///
/// The transform layer bridges semantic meaning (emphasis, strong) with
/// rendering output while maintaining token-level precision for tooling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextTransform {
    /// Identity transform (does nothing) - applied to plain text
    /// This is the default transform for regular text content
    Identity(Text),

    /// Emphasis transform - typically renders as italic
    /// Supports nested transforms for complex formatting
    Emphasis(Vec<TextTransform>),

    /// Strong emphasis transform - typically renders as bold
    /// Supports nested transforms for complex formatting  
    Strong(Vec<TextTransform>),

    /// Inline code transform - monospace formatting
    /// Code cannot contain nested transforms (by design)
    Code(Text),

    /// Math transform - mathematical expressions
    /// Math cannot contain nested transforms (by design)
    Math(Text),

    /// Composed transform - for complex nested cases
    /// Used when multiple transforms need to be applied
    /// Example: **_bold italic_** becomes Strong(vec![Emphasis(...)])
    Composed(Vec<TextTransform>),

    /// Custom transform for extensibility
    /// Allows for user-defined or format-specific transforms
    Custom {
        /// Transform type name
        name: String,

        /// Transform parameters
        parameters: HashMap<String, String>,

        /// Nested transforms (if applicable)
        content: Vec<TextTransform>,
    },
}

/// Lowest level text node - collection of tokens
///
/// This is the foundation of all text content in the AST. A Text node:
/// - Contains character-level token information for language server support
/// - Represents a logical unit of text (word, phrase, or line segment)
/// - Never contains line breaks (those are structural elements)
/// - May span multiple tokens (word + punctuation, etc.)
///
/// The token sequence enables:
/// - Precise hover information
/// - Character-accurate autocomplete
/// - Exact syntax highlighting
/// - Perfect source reconstruction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    /// Token sequence with character-level precision
    pub tokens: ScannerTokenSequence,
}

impl Text {
    /// Get the text content by concatenating token content
    pub fn content(&self) -> String {
        self.tokens.text()
    }

    /// Create a simple text node from a string (for testing/convenience)
    pub fn simple(content: &str) -> Self {
        // This would normally be created by the tokenizer with proper positions
        Self {
            tokens: ScannerTokenSequence {
                tokens: vec![ScannerToken::Text {
                    content: content.to_string(),
                    span: SourceSpan {
                        start: Position { row: 0, column: 0 },
                        end: Position {
                            row: 0,
                            column: content.len(),
                        },
                    },
                }],
            },
        }
    }
}

impl TextTransform {
    /// Extract the text content from this transform recursively
    pub fn text_content(&self) -> String {
        match self {
            TextTransform::Identity(text) => text.content(),
            TextTransform::Emphasis(transforms) => transforms
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
            TextTransform::Strong(transforms) => transforms
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
            TextTransform::Code(text) => text.content(),
            TextTransform::Math(text) => text.content(),
            TextTransform::Composed(transforms) => transforms
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
            TextTransform::Custom { content, .. } => content
                .iter()
                .map(|t| t.text_content())
                .collect::<Vec<_>>()
                .join(""),
        }
    }

    /// Check if this transform is an identity (no formatting applied)
    pub fn is_identity(&self) -> bool {
        matches!(self, TextTransform::Identity(_))
    }

    pub fn to_inline(&self) -> Inline {
        Inline::TextLine(self.clone())
    }
}
