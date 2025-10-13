//! Token-level AST nodes for character-precise language server support
//!
//! This module defines the lowest-level AST nodes that maintain exact source
//! positions for every character. This enables precise language server features
//! like hover, autocomplete, go-to-definition, and syntax highlighting.
//!
//! # Parsing Pipeline Position
//!
//! **Phase 1.b: Tokenization**
//!
//! These tokens are produced by the lexer after verbatim line marking (1.a).
//! The tokenizer converts raw source text into character-precise tokens with
//! exact source positions. This is the foundation for all subsequent parsing
//! phases and language server precision.
//!
//! Pipeline: `Source Text` → `Verbatim Marking` → **`Tokens`** → `Block Grouping` → `AST Nodes`

use serde::{Deserialize, Serialize};

/// Rich semantic information for sequence markers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SequenceMarkerType {
    /// Plain markers like "-", "*"
    Plain(String),
    /// Numerical markers like "1.", "42)" with parsed number and original string
    Numerical(u64, String),
    /// Alphabetical markers like "a.", "Z)" with parsed letter and original string  
    Alphabetical(char, String),
    /// Roman numeral markers like "i.", "IV)" with parsed value and original string
    Roman(u64, String),
}

impl SequenceMarkerType {
    /// Get the original string representation of this sequence marker
    pub fn content(&self) -> &str {
        match self {
            SequenceMarkerType::Plain(s) => s,
            SequenceMarkerType::Numerical(_, s) => s,
            SequenceMarkerType::Alphabetical(_, s) => s,
            SequenceMarkerType::Roman(_, s) => s,
        }
    }

    /// Get the semantic value as a number (for ordered lists)
    pub fn numeric_value(&self) -> Option<u64> {
        match self {
            SequenceMarkerType::Plain(_) => None,
            SequenceMarkerType::Numerical(n, _) => Some(*n),
            SequenceMarkerType::Alphabetical(c, _) => {
                // Convert a-z to 1-26, A-Z to 1-26
                if c.is_ascii_lowercase() {
                    Some((*c as u8 - b'a' + 1) as u64)
                } else if c.is_ascii_uppercase() {
                    Some((*c as u8 - b'A' + 1) as u64)
                } else {
                    None
                }
            }
            SequenceMarkerType::Roman(n, _) => Some(*n),
        }
    }
}

/// Precise source position for character-level language server support
///
/// Unlike traditional AST source spans, we need both start and end positions
/// because inline elements don't necessarily start at column 0, and we need
/// precise boundaries for language server operations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed)
    pub row: usize,
    /// Column number (0-indexed, UTF-8 byte offset)
    pub column: usize,
}

/// Source span covering a range of characters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)  
    pub end: Position,
}

/// Individual token with precise source location
///
/// Type-safe token variants based on TXXT reference implementation.
/// Each variant represents a specific syntactic element for precise
/// language server support and type safety.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Token {
    /// Regular text content (words, sentences, paragraphs)
    Text { content: String, span: SourceSpan },

    /// Whitespace characters (spaces and tabs, but not newlines)
    Whitespace { content: String, span: SourceSpan },

    /// Line break characters
    Newline { span: SourceSpan },

    /// Blank line (empty line with possible whitespace)
    BlankLine {
        /// The whitespace content of the blank line (spaces/tabs before the newline)
        whitespace: String,
        span: SourceSpan,
    },

    /// Indentation increase
    Indent { span: SourceSpan },

    /// Indentation decrease  
    Dedent { span: SourceSpan },

    /// List/sequence markers (1., -, a), etc.) with rich semantic information
    SequenceMarker {
        marker_type: SequenceMarkerType,
        span: SourceSpan,
    },

    /// Annotation markers (:: label ::)
    AnnotationMarker { content: String, span: SourceSpan },

    /// Definition markers (term ::)
    DefinitionMarker { content: String, span: SourceSpan },

    /// Dash character (-)
    Dash { span: SourceSpan },

    /// Period character (.)
    Period { span: SourceSpan },

    /// Left bracket character ([)
    LeftBracket { span: SourceSpan },

    /// Right bracket character (])
    RightBracket { span: SourceSpan },

    /// At-sign character (@)
    AtSign { span: SourceSpan },

    /// Left parenthesis character (()
    LeftParen { span: SourceSpan },

    /// Right parenthesis character ())
    RightParen { span: SourceSpan },

    /// Colon character (:)
    Colon { span: SourceSpan },

    /// Identifier (variable names, labels)
    Identifier { content: String, span: SourceSpan },

    /// Reference markers ([text], [@citation], [#section])
    RefMarker { content: String, span: SourceSpan },

    /// Footnote references ([1], [2], [^label])
    FootnoteRef {
        footnote_type: crate::tokenizer::inline::references::footnote_ref::FootnoteType,
        span: SourceSpan,
    },

    /// Verbatim block title (title:)
    VerbatimTitle { content: String, span: SourceSpan },

    /// Verbatim block content (preserved exactly)
    VerbatimContent { content: String, span: SourceSpan },

    /// Verbatim block label (:: label syntax)
    VerbatimLabel { content: String, span: SourceSpan },

    /// Parameter key-value pair (key=value)
    Parameter {
        key: String,
        value: String,
        span: SourceSpan,
    },

    /// Bold text delimiter (*)
    BoldDelimiter { span: SourceSpan },

    /// Italic text delimiter (_)
    ItalicDelimiter { span: SourceSpan },

    /// Code text delimiter (`)
    CodeDelimiter { span: SourceSpan },

    /// Math text delimiter (#)
    MathDelimiter { span: SourceSpan },

    /// Citation reference ([@key])
    CitationRef { content: String, span: SourceSpan },

    /// Page reference ([p.123] or [p.123-125])
    PageRef { content: String, span: SourceSpan },

    /// Session reference ([#1.2] or [#section])
    SessionRef { content: String, span: SourceSpan },

    /// End of file marker
    Eof { span: SourceSpan },
}

impl Token {
    /// Get the source span for this token
    pub fn span(&self) -> &SourceSpan {
        match self {
            Token::Text { span, .. } => span,
            Token::Whitespace { span, .. } => span,
            Token::Newline { span } => span,
            Token::BlankLine { span, .. } => span,
            Token::Indent { span } => span,
            Token::Dedent { span } => span,
            Token::SequenceMarker { span, .. } => span,
            Token::AnnotationMarker { span, .. } => span,
            Token::DefinitionMarker { span, .. } => span,
            Token::Dash { span } => span,
            Token::Period { span } => span,
            Token::LeftBracket { span } => span,
            Token::RightBracket { span } => span,
            Token::AtSign { span } => span,
            Token::LeftParen { span } => span,
            Token::RightParen { span } => span,
            Token::Colon { span } => span,
            Token::Identifier { span, .. } => span,
            Token::RefMarker { span, .. } => span,
            Token::FootnoteRef { span, .. } => span,
            Token::VerbatimTitle { span, .. } => span,
            Token::VerbatimContent { span, .. } => span,
            Token::VerbatimLabel { span, .. } => span,
            Token::Parameter { span, .. } => span,
            Token::BoldDelimiter { span } => span,
            Token::ItalicDelimiter { span } => span,
            Token::CodeDelimiter { span } => span,
            Token::MathDelimiter { span } => span,
            Token::CitationRef { span, .. } => span,
            Token::PageRef { span, .. } => span,
            Token::SessionRef { span, .. } => span,
            Token::Eof { span } => span,
        }
    }

    /// Get the text content of this token (empty for structural tokens)
    pub fn content(&self) -> &str {
        match self {
            Token::Text { content, .. } => content,
            Token::Whitespace { content, .. } => content,
            Token::SequenceMarker { marker_type, .. } => marker_type.content(),
            Token::AnnotationMarker { content, .. } => content,
            Token::DefinitionMarker { content, .. } => content,
            Token::Identifier { content, .. } => content,
            Token::RefMarker { content, .. } => content,
            Token::FootnoteRef { .. } => "", // Use footnote_type() method for structured access
            Token::VerbatimTitle { content, .. } => content,
            Token::VerbatimContent { content, .. } => content,
            Token::VerbatimLabel { content, .. } => content,
            Token::Parameter { key, .. } => key, // Return key for content (value accessible separately)
            Token::BoldDelimiter { .. } => "*",
            Token::ItalicDelimiter { .. } => "_",
            Token::CodeDelimiter { .. } => "`",
            Token::MathDelimiter { .. } => "#",
            Token::CitationRef { content, .. } => content,
            Token::PageRef { content, .. } => content,
            Token::SessionRef { content, .. } => content,
            Token::Newline { .. } => "\n",
            Token::BlankLine { whitespace, .. } => whitespace,
            Token::Indent { .. } => "",
            Token::Dedent { .. } => "",
            Token::Dash { .. } => "-",
            Token::Period { .. } => ".",
            Token::LeftBracket { .. } => "[",
            Token::RightBracket { .. } => "]",
            Token::AtSign { .. } => "@",
            Token::LeftParen { .. } => "(",
            Token::RightParen { .. } => ")",
            Token::Colon { .. } => ":",
            Token::Eof { .. } => "",
        }
    }

    /// Get the parameter value (only valid for Parameter tokens)
    pub fn parameter_value(&self) -> Option<&str> {
        match self {
            Token::Parameter { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Get the semantic sequence marker information (only valid for SequenceMarker tokens)
    pub fn sequence_marker_type(&self) -> Option<&SequenceMarkerType> {
        match self {
            Token::SequenceMarker { marker_type, .. } => Some(marker_type),
            _ => None,
        }
    }

    /// Get the footnote type information (only valid for FootnoteRef tokens)
    pub fn footnote_type(
        &self,
    ) -> Option<&crate::tokenizer::inline::references::footnote_ref::FootnoteType> {
        match self {
            Token::FootnoteRef { footnote_type, .. } => Some(footnote_type),
            _ => None,
        }
    }
}

/// Collection of tokens that forms a logical text unit
///
/// This bridges the gap between character-level precision (tokens) and
/// semantic structure (blocks/inlines). Most semantic operations work
/// with TokenSequence, while language server operations drill down to
/// individual tokens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenSequence {
    pub tokens: Vec<Token>,
}

impl Default for TokenSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenSequence {
    /// Create a new empty token sequence
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Get the overall source span covering all tokens
    pub fn span(&self) -> Option<SourceSpan> {
        if self.tokens.is_empty() {
            return None;
        }

        let start = self.tokens[0].span().start;
        let end = self.tokens.last().unwrap().span().end;

        Some(SourceSpan { start, end })
    }

    /// Get the text content by concatenating all token content
    pub fn text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.content())
            .collect::<Vec<_>>()
            .join("")
    }
}
