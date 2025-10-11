//! Reference types: citations, cross-references, and link targets
//!
//! This module defines types for references and citation systems that link
//! to other parts of documents or external resources.
//!
//! # Parsing Pipeline Position
//!
//! **Phase 2.b: Parsing** (Reference Recognition and Classification)
//!
//! References are parsed during the main parsing phase when inline content
//! is processed. The comprehensive ReferenceTarget system captures all
//! reference types even if full resolution isn't implemented yet.
//!
//! Pipeline: `Tokens` → `Block Grouping` → **`Reference Parsing`** → `Assembly`
//!
//! ## Reference Processing (2.b)
//!
//! The parser recognizes and classifies different reference types:
//! 1. **Pattern matching**: Identify reference syntax in token stream
//! 2. **Type classification**: Determine specific reference type
//! 3. **Structure extraction**: Parse internal structure (keys, locators, etc.)
//! 4. **Preservation**: Maintain raw form for exact source reconstruction
//!
//! Note: Full reference resolution (link validation, target existence) is
//! intentionally left to higher-level tools, following the principle that
//! the parser doesn't validate external resources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::tokens::TokenSequence;

/// Comprehensive reference target system for TXXT documents
///
/// This system handles all types of references supported by TXXT, even though
/// the parser may not fully implement all resolution logic yet. Including the
/// complete type system ensures no documentation is lost and provides a
/// foundation for future tooling development.
///
/// Reference types supported:
/// - File references: [./filename.txxt], [../dir/file.txxt]
/// - Section references: [local-section], [#3], [#-1.2]
/// - URL references: [example.com], [https://example.com]
/// - Citation references: [@smith2023], [@doe2024; @jones2025]
/// - Named anchor references: [#hello-world], [#security-note]
/// - Naked numerical references: [1], [2] (shorthand for [#-1.1], [#-1.2])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReferenceTarget {
    /// File reference to local or absolute paths
    /// Examples: [./filename.txxt], [../dir/file.txxt], [/absolute/path.txxt]
    File {
        /// File path (relative or absolute)
        path: String,

        /// Optional section within the file
        section: Option<String>,

        /// Raw reference text as it appears in source
        raw: String,

        /// Source position
        tokens: TokenSequence,
    },

    /// Section reference within current or other documents
    /// Examples: [local-section], [#3], [#2.1], [#-1.2]
    Section {
        /// Section identifier (can be numeric or named)
        identifier: SectionIdentifier,

        /// Raw reference text
        raw: String,

        /// Source position
        tokens: TokenSequence,
    },

    /// URL reference to external resources
    /// Examples: [example.com], [https://example.com], [user@domain.com]
    Url {
        /// Full URL or domain
        url: String,

        /// Optional fragment (#anchor)
        fragment: Option<String>,

        /// Raw reference text
        raw: String,

        /// Source position
        tokens: TokenSequence,
    },

    /// Citation reference to bibliography entries
    /// Examples: [@smith2023], [@doe2024; @jones2025], [@smith2023, p. 45]
    Citation {
        /// Citation keys and locators
        citations: Vec<CitationEntry>,

        /// Raw reference text
        raw: String,

        /// Source position
        tokens: TokenSequence,
    },

    /// Named anchor reference using parameters
    /// Examples: [#hello-world], [#security-note]
    NamedAnchor {
        /// Anchor name (from ref= or id= parameters)
        anchor: String,

        /// Raw reference text
        raw: String,

        /// Source position
        tokens: TokenSequence,
    },

    /// Naked numerical reference (footnote-style shorthand)
    /// Examples: [1], [2] → automatically interpreted as [#-1.1], [#-1.2]
    NakedNumerical {
        /// Number referenced
        number: u32,

        /// Raw reference text
        raw: String,

        /// Source position
        tokens: TokenSequence,
    },

    /// Unresolved or malformed reference
    /// Preserved for error reporting and future resolution attempts
    Unresolved {
        /// Raw reference content
        content: String,

        /// Raw reference text including brackets
        raw: String,

        /// Reason for non-resolution (if known)
        reason: Option<String>,

        /// Source position
        tokens: TokenSequence,
    },
}

/// Section identifier types for section references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SectionIdentifier {
    /// Numeric reference: #3, #2.1, #1.a.i
    Numeric {
        /// Section numbers (e.g., [1, 2, 3] for #1.2.3)
        levels: Vec<u32>,

        /// Whether this uses negative indexing (#-1, #-2)
        negative_index: bool,
    },

    /// Named reference: #introduction, #conclusion
    Named {
        /// Section name/slug
        name: String,
    },

    /// Mixed reference: #1.introduction (number + name)
    Mixed {
        /// Numeric prefix
        levels: Vec<u32>,

        /// Name suffix
        name: String,

        /// Whether numeric part uses negative indexing
        negative_index: bool,
    },
}

/// Individual citation entry within a citation reference
///
/// Supports complex citation syntax like [@doe2024; @jones2025] and
/// [@smith2023, p. 45] with multiple keys and locators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CitationEntry {
    /// Citation key from bibliography file
    pub key: String,

    /// Optional locator (page, section, etc.)
    /// Examples: "p. 45", "ch. 3", "§2.1"
    pub locator: Option<String>,

    /// Optional prefix text
    /// Examples: "see", "cf.", "compare"
    pub prefix: Option<String>,

    /// Optional suffix text
    pub suffix: Option<String>,
}

/// Citation reference for academic/technical documents
///
/// Citations reference external sources and can be formatted according to
/// various citation styles. They integrate with the bibliography system for
/// cross-document linking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    /// Citation key/identifier
    pub key: String,

    /// Optional page numbers, sections, etc.
    pub locator: Option<String>,

    /// Citation style prefix (e.g., "see", "cf.")
    pub prefix: Option<String>,

    /// Citation style suffix
    pub suffix: Option<String>,

    /// Raw tokens for positioning
    pub tokens: TokenSequence,
}

/// Cross-reference to document elements
///
/// Cross-references link to other parts of the same document or external
/// documents. They support automatic numbering and title extraction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossReference {
    /// Target element identifier
    pub target: String,

    /// Type of reference (section, figure, table, etc.)
    pub ref_type: ReferenceType,

    /// Custom display text (if not auto-generated)
    pub custom_text: Option<Vec<super::inlines::Inline>>,

    /// Raw tokens for positioning
    pub tokens: TokenSequence,
}

/// Types of cross-references supported
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Reference to a session/section
    Section,

    /// Reference to a figure or image
    Figure,

    /// Reference to a table
    Table,

    /// Reference to a list item
    ListItem,

    /// Reference to a definition
    Definition,

    /// Generic reference (type determined by target)
    Generic,

    /// Custom reference type
    Custom(String),
}

/// Link target information
///
/// Used for both internal and external links, with support for fragment
/// identifiers and link metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkTarget {
    /// Base URL or path
    pub url: String,

    /// Fragment identifier (#section)
    pub fragment: Option<String>,

    /// Link title for tooltips
    pub title: Option<String>,

    /// Additional link metadata
    pub metadata: HashMap<String, String>,
}

/// Bibliography declaration for document-level citation support
///
/// Declares external bibliography files that provide citation data.
/// Typically appears as: :: bibliography :: references.bib
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bibliography {
    /// Paths to bibliography files (BibTeX, etc.)
    pub files: Vec<String>,

    /// Bibliography format (bibtex, json, etc.)
    pub format: BibliographyFormat,

    /// Additional bibliography metadata
    pub metadata: HashMap<String, String>,

    /// Source position
    pub tokens: TokenSequence,
}

/// Supported bibliography formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BibliographyFormat {
    /// BibTeX format (.bib files)
    BibTeX,

    /// JSON bibliography format
    Json,

    /// YAML bibliography format  
    Yaml,

    /// Custom/unknown format
    Custom(String),
}

impl ReferenceTarget {
    /// Get the raw reference text as it appears in source
    pub fn raw_text(&self) -> &str {
        match self {
            ReferenceTarget::File { raw, .. } => raw,
            ReferenceTarget::Section { raw, .. } => raw,
            ReferenceTarget::Url { raw, .. } => raw,
            ReferenceTarget::Citation { raw, .. } => raw,
            ReferenceTarget::NamedAnchor { raw, .. } => raw,
            ReferenceTarget::NakedNumerical { raw, .. } => raw,
            ReferenceTarget::Unresolved { raw, .. } => raw,
        }
    }

    /// Get the source tokens for this reference
    pub fn tokens(&self) -> &TokenSequence {
        match self {
            ReferenceTarget::File { tokens, .. } => tokens,
            ReferenceTarget::Section { tokens, .. } => tokens,
            ReferenceTarget::Url { tokens, .. } => tokens,
            ReferenceTarget::Citation { tokens, .. } => tokens,
            ReferenceTarget::NamedAnchor { tokens, .. } => tokens,
            ReferenceTarget::NakedNumerical { tokens, .. } => tokens,
            ReferenceTarget::Unresolved { tokens, .. } => tokens,
        }
    }

    /// Check if this is a local reference (file or section)
    pub fn is_local(&self) -> bool {
        matches!(
            self,
            ReferenceTarget::File { .. }
                | ReferenceTarget::Section { .. }
                | ReferenceTarget::NamedAnchor { .. }
                | ReferenceTarget::NakedNumerical { .. }
        )
    }

    /// Check if this is an external reference (URL or citation)
    pub fn is_external(&self) -> bool {
        matches!(
            self,
            ReferenceTarget::Url { .. } | ReferenceTarget::Citation { .. }
        )
    }

    /// Check if this reference needs resolution
    pub fn needs_resolution(&self) -> bool {
        !matches!(self, ReferenceTarget::Unresolved { .. })
    }
}

impl SectionIdentifier {
    /// Check if this is a numeric identifier
    pub fn is_numeric(&self) -> bool {
        matches!(self, SectionIdentifier::Numeric { .. })
    }

    /// Check if this is a named identifier
    pub fn is_named(&self) -> bool {
        matches!(self, SectionIdentifier::Named { .. })
    }

    /// Check if this uses negative indexing
    pub fn uses_negative_index(&self) -> bool {
        match self {
            SectionIdentifier::Numeric { negative_index, .. } => *negative_index,
            SectionIdentifier::Mixed { negative_index, .. } => *negative_index,
            _ => false,
        }
    }
}
