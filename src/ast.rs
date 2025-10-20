//! AST module for TXXT format
//!
//! This module defines a comprehensive, type-safe AST structure for TXXT documents
//! that serves multiple tooling needs including language servers, formatters, linters,
//! and converters.
//!
//! # Core Design Principles
//!
//! ## Token-Level Precision Architecture
//!
//! The AST handles both tokens and parsed structure to facilitate language server features,
//! source mapping, and round-tripping. Every text element maintains character-level precision
//! through [`ScannerTokenSequence`] for:
//! - Hover information at exact cursor positions
//! - Autocomplete triggering within identifiers  
//! - Precise syntax highlighting and error underlining
//! - Perfect source reconstruction
//!
//! ## Container Indentation Pattern
//!
//! Critical architectural insight: The container is what gets indented, not the parent element.
//! This explains why flat lists don't need indentation - only nested content requires a [`Container`].
//!
//! Example:
//! ```txxt
//! - Item 1
//! - Item 2
//!   - Nested item    // This creates a Container
//! ```
//!
//! AST Structure:
//! ```text
//! List
//! ├── ListItem("Item 1")
//! ├── ListItem("Item 2")
//! └── Container
//!     └── List
//!         └── ListItem("Nested item")
//! ```
//!
//! ## Uniform Type System
//!
//! We enforce a homogeneous tree model where any element node can have parameters
//! or annotations. While some combinations don't occur in practice (language has no
//! syntactic construct for them), the type system remains uniform for consistency.
//!
//! ## Text Transform Layer
//!
//! Every piece of text goes through a uniform transform layer via [`TextTransform`]:
//! - `Identity(Text("banana"))` for plain text
//! - `Emphasis(vec![Identity(Text("important"))])` for *important*
//! - `Strong(vec![Emphasis(vec![Identity(Text("both"))])])` for **_both_**
//!
//! This provides consistent text handling across all contexts while maintaining
//! character-level precision for tooling.
//!

//!
//! # Node Design Decision
//!
//! ## Custom Implementation with Pandoc Inspiration
//!
//! We chose a **custom AST implementation** rather than extending existing libraries because:
//!
//! ### Why Not Existing Libraries?
//! - **Markdown libraries** (pulldown-cmark, comrak) are optimized for CommonMark/GFM, not extensible text formats
//! - **Generic tree libraries** lack domain-specific semantics for text processing
//! - **Pandoc's Haskell AST** provides excellent design patterns but needs Rust-native implementation
//!
//! ### Rowan-Inspired Red-Green Trees + Type safety
//! - **Red-green pattern**: Inspired by rowan (used by rust-analyzer) for efficient tree operations
//! - **Structural sharing**: Memory efficiency through shared immutable nodes  
//! - **Lossless representation**: Preserves all source information including whitespace
//! - **Incremental updates**: Foundation for future language server incremental parsing
//! - **Enum-based nodes**: Compile-time verification of AST structure
//!
//! # Module Organization
//!
//! The AST follows a layered architecture that mirrors the specification structure
//! and provides clear separation of concerns for maintainability and extensibility.
//!
//! ## Layer 1: Node Definitions (`nodes/`)
//! **Element-specific AST nodes that mirror `docs/specs/elements/` structure:**
//! - `nodes/annotation.rs` - Annotation AST nodes
//! - `nodes/container.rs` - Container AST nodes (verbatim, content, session)
//! - `nodes/definition.rs` - Definition AST nodes
//! - `nodes/list.rs` - List AST nodes with sequence markers
//! - `nodes/paragraph.rs` - Paragraph AST nodes (default element)
//! - `nodes/session.rs` - Session AST nodes with numbering
//! - `nodes/verbatim.rs` - Verbatim AST nodes
//! - `nodes/inlines/` - Inline element AST nodes
//!   - `nodes/inlines/formatting.rs` - Text formatting AST
//!   - `nodes/inlines/text.rs` - Plain text content AST
//!   - `nodes/inlines/references/` - Reference AST nodes
//!
//! ## Layer 2: Core Systems (existing structure)
//! **Foundational AST infrastructure:**
//! - [`annotations`] - Metadata attachment system with proximity rules
//! - [`base`] - Core document structure and assembly information  
//! - [`blocks`] - Block-level elements (verbatim, lists, definitions)
//! - [`inlines`] - Inline elements with text transform layer
//! - [`parameters`] - Shared metadata system (ref=, id=, severity=)
//! - [`reference_types`] - References and citations ([file.txxt], [@smith2023])
//! - [`structure`] - Hierarchical elements (containers, sessions, paragraphs)
//! - [`tokens`] - Character-precise positioning for language servers
//!
//! # Design Philosophy
//!
//! ## Specification Alignment
//! The AST structure perfectly mirrors the specification to ensure:
//! - Easy navigation between docs, code, and tests
//! - Consistent naming and organization
//! - Perfect alignment with parser and tokenizer modules
//!
//! ## Parser Integration
//! Each AST node type corresponds to:
//! - A specification in `docs/specs/elements/`
//! - A tokenizer in `src/tokenizer/elements/`
//! - A parser in `src/parser/elements/`
//! - Test cases via `TxxtCorpora` extraction
//!
//! ## Testing Integration
//! All AST nodes integrate with the specification-driven testing framework:
//! ```rust,ignore
//! use tests::corpora::{TxxtCorpora, ProcessingStage};
//!
//! let corpus = TxxtCorpora::load_with_processing(
//!     "txxt.core.spec.paragraph.valid.simple",
//!     ProcessingStage::ParsedAst
//! )?;
//! let ast_node = corpus.ast().unwrap();
//! ```

// ============================================================================
// NEW AST SYSTEM - Modern typed AST with token-level precision
// ============================================================================

// NEW: Spec-aligned element structure (replaces nodes/)
pub mod debug;
pub mod elements;
pub mod semantic_tokens;

// REMOVED: Legacy nodes/ structure - replaced by spec-aligned elements/

// Re-export spec-aligned element types as the canonical AST
pub use elements::{
    annotation::{AnnotationBlock, AnnotationContent},
    containers::ContentContainer,
    core::{BlankLine, ContainerType, ElementNode, ElementType, TxxtElement},
    definition::{DefinitionBlock, DefinitionTerm},
    document::{AssemblyInfo, Document, Meta, MetaValue, ProcessingStats},
    formatting::{BoldSpan, CodeSpan, ItalicSpan, MathSpan},
    inlines::{Link, Reference, ReferenceSpan, TextLine, TextSpan, TextTransform},
    list::{ListBlock, ListDecorationType, ListItem, NumberingForm, NumberingStyle},
    paragraph::ParagraphBlock,
    references::{CitationSpan, FootnoteReferenceSpan, PageReferenceSpan, SessionReferenceSpan},
    session::SessionContainer,
    session::{SessionBlock, SessionNumbering, SessionTitle},
    verbatim::IgnoreContainer,
    verbatim::{VerbatimBlock, VerbatimType},
};

// Core AST infrastructure
pub use elements::{scanner_tokens, traversal};

// Semantic tokens infrastructure
pub use semantic_tokens::{
    FromScannerToken, SemanticNumberingForm, SemanticNumberingStyle, SemanticToken,
    SemanticTokenBuilder, SemanticTokenList, SemanticTokenSpan, ToScannerToken,
};

// Legacy re-exports for backward compatibility - REMOVED
// All callers have been updated to use the new element paths

// Advanced query and traversal API (Unist-compatible) - temporarily disabled
// pub mod query;
