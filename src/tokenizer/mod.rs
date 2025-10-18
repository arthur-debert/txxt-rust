//! TXXT Tokenizer - Organized by core/elements/infrastructure structure
//!
//! The tokenizer is organized to mirror the AST/parser structure with clear
//! separation between core tokenization logic, element-specific implementations,
//! and supporting infrastructure. This provides consistent organization across
//! all domains in the codebase.
//!
//! ## Core Modules
//!
//! - [`core`] - Fundamental tokenization components
//!   - [`core::lexer`] - Main tokenization engine
//!   - [`core::indentation`] - Indentation tracking and container boundaries
//!   - [`core::patterns`] - Core pattern matching utilities
//!
//! - [`pipeline`] - Token processing pipeline stages
//!   - [`pipeline::token_tree_builder`] - Transform flat tokens into hierarchical token tree
//!
//! ## Element Modules (Organized by Specification)
//!
//! - [`elements`] - All element tokenization organized by type
//!   - [`elements::annotation`] - Annotation elements
//!   - [`elements::containers`] - Container elements  
//!   - [`elements::definition`] - Definition elements
//!   - [`elements::document`] - Document-level elements
//!   - [`elements::formatting`] - Text formatting elements
//!   - [`elements::list`] - List-related elements
//!   - [`elements::paragraph`] - Paragraph elements
//!   - [`elements::references`] - Reference and link elements
//!   - [`elements::session`] - Session-related elements
//!   - [`elements::verbatim`] - Verbatim elements
//!   - [`elements::components`] - Shared component elements
//!
//! ## Infrastructure Modules
//!
//! - [`infrastructure`] - Marker detection and supporting utilities
//! - [`verbatim_scanner`] - Pre-parsing verbatim detection
//!
//! ## Architecture
//!
//! This design achieves consistent organization across domains (AST/parser/tokenizer)
//! while maintaining clear separation between core logic, element implementations,
//! and supporting infrastructure.

// Core tokenization logic
pub mod core;

// Token processing pipeline
pub mod pipeline;

// Element modules organized by specification structure
pub mod elements;

// Infrastructure and utilities
pub mod infrastructure;

// Re-export main interfaces
pub use core::Lexer;
pub use elements::verbatim::{VerbatimBlock, VerbatimScanner, VerbatimType};
pub use pipeline::{ScannerTokenTree, ScannerTokenTreeBuilder};

// Re-export formatting functionality
pub use elements::formatting::{read_inline_delimiter, InlineDelimiterLexer};

// Re-export reference functionality
pub use elements::references::{
    read_citation_ref, read_page_ref, read_session_ref, CitationRefLexer, PageRefLexer,
    ReferenceLexer, SessionRefLexer,
};

// Re-export new AST scanner token types
pub use crate::ast::scanner_tokens::{Position, ScannerToken, ScannerTokenSequence, SourceSpan};

/// Main tokenization entry point
///
/// Processes TXXT text and returns ScannerToken enum variants with precise source positions
pub fn tokenize(text: &str) -> Vec<ScannerToken> {
    let mut lexer = Lexer::new(text);
    lexer.tokenize()
}
