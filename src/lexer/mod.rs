//! Phase 1: Lexer - Tokenization
//!
//! This module implements the lexer phase that converts source text into tokens.
//!
//! ## Lexer Steps
//!
//! Step 1.a: Verbatim scanning (handled internally by tokenize)
//! Step 1.b: Tokenization - converts text to flat token stream
//! Step 1.c: Token tree building - organizes tokens hierarchically
//!
//! ## Core Modules
//!
//! - [`core`] - Fundamental tokenization components
//!   - [`core::lexer`] - Main tokenization engine
//!   - [`core::indentation`] - Indentation tracking and container boundaries
//!   - [`core::patterns`] - Core pattern matching utilities
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
//! ## Processing Steps
//!
//! - [`token_tree_builder`] - Step 1.c: Transform flat tokens into hierarchical token tree
//!
//! ## Architecture
//!
//! This design achieves consistent organization across domains (AST/parser/tokenizer)
//! while maintaining clear separation between core logic, element implementations,
//! and supporting infrastructure.

// Core tokenization logic
pub mod core;

// Element modules organized by specification structure
pub mod elements;

// Processing step: Token tree builder
pub mod token_tree_builder;

// Infrastructure and utilities
// pub mod infrastructure; // TODO: Add infrastructure modules when needed

// Re-export main interfaces
pub use core::Lexer;
pub use elements::verbatim::{VerbatimBlock, VerbatimScanner, VerbatimType};
pub use token_tree_builder::{ScannerTokenTree, ScannerTokenTreeBuilder};

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
