//! Scanner Tokens - Low-level character-precise tokens
//!
//! This module defines the lowest-level scanner tokens that maintain exact source
//! positions for every character. These are distinct from semantic tokens which
//! represent higher-level syntactic structures. Scanner tokens enable precise
//! language server features like hover, autocomplete, go-to-definition, and
//! syntax highlighting.

use serde::{Deserialize, Serialize};

use super::primitives::SourceSpan;

/// Type of indentation wall for verbatim blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WallType {
    /// In-flow mode: wall at title_indent + 4 spaces
    InFlow(usize),
    /// Stretched mode: wall at absolute column 1 (0-based)
    Stretched,
}

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

/// Individual scanner token with precise source location
///
/// Type-safe scanner token variants based on TXXT reference implementation.
/// Each variant represents a specific syntactic element for precise
/// language server support and type safety. These are low-level tokens
/// from the lexer, distinct from semantic tokens which represent higher-level
/// syntactic structures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScannerToken {
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

    /// Equals character (=)
    Equals { span: SourceSpan },

    /// Comma character (,)
    Comma { span: SourceSpan },

    /// TXXT marker (::) - fundamental structural element
    TxxtMarker { span: SourceSpan },

    /// Identifier (variable names, labels)
    Identifier { content: String, span: SourceSpan },

    /// Reference markers ([text], [@citation], [#section])
    RefMarker { content: String, span: SourceSpan },

    /// Footnote references ([1], [2], [^label])
    FootnoteRef {
        footnote_type: crate::syntax::elements::references::footnote_ref::FootnoteType,
        span: SourceSpan,
    },

    /// Verbatim block start boundary (NEW - Issue #132)
    VerbatimBlockStart {
        /// Title text (without trailing colon)
        title: String,
        /// Wall type for content indentation
        wall_type: WallType,
        span: SourceSpan,
    },

    /// Verbatim content line (NEW - Issue #132)
    /// Raw content line inside verbatim block, unparsed
    VerbatimContentLine {
        /// Raw content of the line
        content: String,
        /// Full indentation before content (for wall calculation)
        indentation: String,
        span: SourceSpan,
    },

    /// Verbatim block end boundary (NEW - Issue #132)
    VerbatimBlockEnd {
        /// Raw label and parameters string (e.g., "python:version=3.11")
        label_raw: String,
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

impl ScannerToken {
    /// Get the source span for this scanner token
    pub fn span(&self) -> &SourceSpan {
        match self {
            ScannerToken::Text { span, .. } => span,
            ScannerToken::Whitespace { span, .. } => span,
            ScannerToken::Newline { span } => span,
            ScannerToken::BlankLine { span, .. } => span,
            ScannerToken::Indent { span } => span,
            ScannerToken::Dedent { span } => span,
            ScannerToken::SequenceMarker { span, .. } => span,
            ScannerToken::TxxtMarker { span } => span,
            ScannerToken::Dash { span } => span,
            ScannerToken::Period { span } => span,
            ScannerToken::LeftBracket { span } => span,
            ScannerToken::RightBracket { span } => span,
            ScannerToken::AtSign { span } => span,
            ScannerToken::LeftParen { span } => span,
            ScannerToken::RightParen { span } => span,
            ScannerToken::Colon { span } => span,
            ScannerToken::Equals { span } => span,
            ScannerToken::Comma { span } => span,
            ScannerToken::Identifier { span, .. } => span,
            ScannerToken::RefMarker { span, .. } => span,
            ScannerToken::FootnoteRef { span, .. } => span,
            ScannerToken::VerbatimBlockStart { span, .. } => span,
            ScannerToken::VerbatimContentLine { span, .. } => span,
            ScannerToken::VerbatimBlockEnd { span, .. } => span,
            ScannerToken::BoldDelimiter { span } => span,
            ScannerToken::ItalicDelimiter { span } => span,
            ScannerToken::CodeDelimiter { span } => span,
            ScannerToken::MathDelimiter { span } => span,
            ScannerToken::CitationRef { span, .. } => span,
            ScannerToken::PageRef { span, .. } => span,
            ScannerToken::SessionRef { span, .. } => span,
            ScannerToken::Eof { span } => span,
        }
    }

    /// Get the text content of this scanner token (empty for structural tokens)
    pub fn content(&self) -> &str {
        match self {
            ScannerToken::Text { content, .. } => content,
            ScannerToken::Whitespace { content, .. } => content,
            ScannerToken::SequenceMarker { marker_type, .. } => marker_type.content(),
            ScannerToken::TxxtMarker { .. } => "::",
            ScannerToken::Identifier { content, .. } => content,
            ScannerToken::RefMarker { content, .. } => content,
            ScannerToken::FootnoteRef { .. } => "", // Use footnote_type() method for structured access
            ScannerToken::VerbatimBlockStart { title, .. } => title,
            ScannerToken::VerbatimContentLine { content, .. } => content,
            ScannerToken::VerbatimBlockEnd { label_raw, .. } => label_raw,
            ScannerToken::BoldDelimiter { .. } => "*",
            ScannerToken::ItalicDelimiter { .. } => "_",
            ScannerToken::CodeDelimiter { .. } => "`",
            ScannerToken::MathDelimiter { .. } => "#",
            ScannerToken::CitationRef { content, .. } => content,
            ScannerToken::PageRef { content, .. } => content,
            ScannerToken::SessionRef { content, .. } => content,
            ScannerToken::Newline { .. } => "\n",
            ScannerToken::BlankLine { whitespace, .. } => whitespace,
            ScannerToken::Indent { .. } => "",
            ScannerToken::Dedent { .. } => "",
            ScannerToken::Dash { .. } => "-",
            ScannerToken::Period { .. } => ".",
            ScannerToken::LeftBracket { .. } => "[",
            ScannerToken::RightBracket { .. } => "]",
            ScannerToken::AtSign { .. } => "@",
            ScannerToken::LeftParen { .. } => "(",
            ScannerToken::RightParen { .. } => ")",
            ScannerToken::Colon { .. } => ":",
            ScannerToken::Equals { .. } => "=",
            ScannerToken::Comma { .. } => ",",
            ScannerToken::Eof { .. } => "",
        }
    }

    /// Get the semantic sequence marker information (only valid for SequenceMarker scanner tokens)
    pub fn sequence_marker_type(&self) -> Option<&SequenceMarkerType> {
        match self {
            ScannerToken::SequenceMarker { marker_type, .. } => Some(marker_type),
            _ => None,
        }
    }

    /// Get the footnote type information (only valid for FootnoteRef scanner tokens)
    pub fn footnote_type(
        &self,
    ) -> Option<&crate::syntax::elements::references::footnote_ref::FootnoteType> {
        match self {
            ScannerToken::FootnoteRef { footnote_type, .. } => Some(footnote_type),
            _ => None,
        }
    }

    pub fn is_bold_delimiter(&self) -> bool {
        matches!(self, ScannerToken::BoldDelimiter { .. })
    }

    pub fn is_italic_delimiter(&self) -> bool {
        matches!(self, ScannerToken::ItalicDelimiter { .. })
    }

    pub fn is_code_delimiter(&self) -> bool {
        matches!(self, ScannerToken::CodeDelimiter { .. })
    }

    pub fn is_math_delimiter(&self) -> bool {
        matches!(self, ScannerToken::MathDelimiter { .. })
    }
}
