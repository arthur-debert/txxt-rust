//! Structural elements: containers, sessions, and paragraphs
//!
//! This module defines the hierarchical structure elements that provide
//! the organizational backbone of TXXT documents.
//!
//! # Parsing Pipeline Position
//!
//! **Phase 2.a: Block Grouping** (Containers)
//! **Phase 2.b: Parsing** (Sessions, Paragraphs, Titles)
//!
//! Containers are created during block grouping when indent/dedent tokens are
//! processed into hierarchical structure. Sessions, paragraphs, and titles are
//! created during the parsing phase when token lists are converted to semantic AST nodes.
//!
//! Pipeline: `Tokens` → **`Block Grouping`** → **`AST Parsing`** → `Assembly`
//!
//! ## Container Creation (2.a)
//!
//! The block grouping phase processes indent/dedent tokens to create the container
//! hierarchy. This is where the "container gets indented, not parent" pattern is
//! established by analyzing indentation levels in the token stream.
//!
//! ## Semantic Parsing (2.b)
//!
//! Sessions, paragraphs, and titles are parsed from grouped token lists into
//! semantic AST nodes with proper typing and structure validation.

use serde::{Deserialize, Serialize};

use super::{annotations::Annotation, inlines::Inline, tokens::TokenSequence};

/// Container for indented block content
///
/// The key architectural insight: containers are what get indented, not their
/// parent elements. This explains why flat lists don't need indentation -
/// only nested content requires a Container.
///
/// Level is computed via tree traversal, never stored as an attribute.
/// This prevents synchronization issues and supports arbitrary nesting depth.
///
/// Example:
/// ```txxt
/// - Item 1
/// - Item 2
///   - Nested item    // This creates a Container
/// ```
///
/// AST:
/// ```text
/// List
/// ├── ListItem("Item 1")
/// ├── ListItem("Item 2")
/// └── Container
///     └── List
///         └── ListItem("Nested item")
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Container {
    /// Nested blocks at this indentation level
    pub content: Vec<super::blocks::Block>,

    /// Annotations attached to this container
    /// (typically from annotations that were the last in their level)
    pub annotations: Vec<Annotation>,
}

/// Session title with hierarchical numbering support
///
/// Sessions support both automatic and manual numbering schemes:
/// - Automatic: "1. Title" becomes level 1
/// - Manual: "1.a.i. Title" becomes level 3
/// - Plain: "Title" becomes unnumbered
///
/// The numbering_style affects how the session integrates with lists
/// and other sessions in the hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTitle {
    /// The title content with token-level precision
    pub content: Vec<Inline>,

    /// Optional session numbering (e.g., "1.2.3", "a)", "i.")
    pub numbering: Option<SessionNumbering>,

    /// Raw tokens for exact source reconstruction
    pub tokens: TokenSequence,
}

/// Session numbering information
///
/// Supports the complex numbering schemes common in technical documents:
/// - Simple: 1, 2, 3
/// - Hierarchical: 1.1, 1.2, 1.3  
/// - Mixed: 1.a.i, 1.a.ii, 1.b.i
///
/// The parser extracts the numbering pattern from the first session
/// at each level, similar to list styling rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionNumbering {
    /// The actual numbering text as it appears in source ("1.2.3", "a)", etc.)
    pub marker: String,

    /// Detected numbering style for this session level
    pub style: NumberingStyle,

    /// Whether this is short form (1.) or full form (1.a.i.)
    pub form: NumberingForm,
}

/// Numbering styles supported for sessions and lists
///
/// Both sessions and lists support the same numbering styles.
/// The key difference: plain sessions have no marker, plain lists use "-".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberingStyle {
    /// Plain text, no numbering
    Plain,

    /// Numerical: 1, 2, 3, ...
    Numerical,

    /// Alphabetical lowercase: a, b, c, ...
    AlphabeticalLower,

    /// Alphabetical uppercase: A, B, C, ...
    AlphabeticalUpper,

    /// Roman numerals lowercase: i, ii, iii, ...
    RomanLower,

    /// Roman numerals uppercase: I, II, III, ...
    RomanUpper,
}

/// Numbering form affects hierarchical display
///
/// Short form: Each level shows only its own number (1., a., i.)
/// Full form: Each level shows the complete hierarchy (1.a.i.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberingForm {
    /// Short form: "1." "a." "i."
    Short,

    /// Full form: "1.a.i."
    Full,
}

/// Session block - a hierarchical section of the document
///
/// Sessions are the primary organizational unit in TXXT. They can be:
/// - Numbered (1. Introduction) or unnumbered (Introduction)
/// - Nested to arbitrary depth
/// - Mixed with different numbering styles at different levels
///
/// Sessions are session containers, meaning they can host new document
/// sessions within their content, unlike content containers (lists, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Session title with optional numbering
    pub title: SessionTitle,

    /// Session content (can contain nested sessions)
    pub content: Container,

    /// Annotations attached to this session
    pub annotations: Vec<Annotation>,
}

/// Paragraph block - the basic unit of flowing text
///
/// Paragraphs contain inline content and are leaf blocks (cannot contain
/// other blocks). They represent a single logical unit of text that flows
/// together, terminated by a blank line or structural change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    /// Paragraph content with inline formatting
    pub content: Vec<Inline>,

    /// Annotations attached to this paragraph
    pub annotations: Vec<Annotation>,

    /// Raw tokens for precise source reconstruction
    pub tokens: TokenSequence,
}

/// Blank line - structural separator
///
/// Blank lines are significant in TXXT as they:
/// - Separate paragraphs
/// - End certain block types  
/// - Affect annotation attachment rules
/// - Are preserved in the final document structure
///
/// Multiple consecutive blank lines are collapsed to a single BlankLine
/// during parsing, except in verbatim blocks where they're preserved exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlankLine {
    /// Source position information
    pub tokens: TokenSequence,
}

/// Ignore line - raw verbatim content preserved exactly
///
/// Ignore lines contain content that should not be parsed as TXXT.
/// They are used within IgnoreContainer to hold verbatim block content
/// while preserving exact spacing and formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgnoreLine {
    /// Raw content preserved byte-for-byte
    pub content: String,

    /// Source position information
    pub tokens: TokenSequence,
}

/// Ignore container - container for verbatim content only
///
/// Ignore containers follow the container architecture but hold only
/// ignore lines and blank lines. They are used exclusively for verbatim
/// block content to preserve formatting exactly while maintaining the
/// consistent container structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgnoreContainer {
    /// Raw content lines preserved exactly
    pub ignore_lines: Vec<IgnoreLine>,

    /// Blank lines within verbatim content
    pub blank_lines: Vec<BlankLine>,

    /// Annotations attached to this container
    pub annotations: Vec<Annotation>,
}
