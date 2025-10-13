//! TXXT document parser package
//!
//! This package provides modular parsing functionality for converting Tokens
//! into Abstract Syntax Trees (AST).
//!
//! # Parsing Pipeline
//!
//! ## Phase 1: Lexer (COMPLETED)
//!
//! ### 1.a. Verbatim Line Marking
//! - Stateful isolation: Verbatim content is sacred and needs special handling
//! - Critical insight: This is the ONLY stateful part, keeping complexity contained
//! - AST mapping: Maps to [`VerbatimContent`] with exact preservation
//!
//! ### 1.b. Tokenization  
//! - Token generation: Produces character-precise tokens needed for language server
//! - AST mapping: Maps directly to [`Token`] enum with [`SourceSpan`] positioning
//!
//! ## Phase 2: Parser (TODO - TO BE IMPLEMENTED)
//!
//! ### 2.a. Block Grouping
//! - Indent/dedent processing: Creates hierarchical structure using container pattern
//! - Tree of token lists: Perfect for the container indentation architecture
//! - AST mapping: Maps to [`Container`] nodes with proper nesting
//!
//! ### 2.b. Parsing
//! - Token list → AST nodes: Converts grouped tokens into semantic structure
//! - Recursive processing: Handles nested containers correctly
//! - AST output: Produces the rich type-safe AST defined in this module
//!
//! ## Phase 3: Post-Processing (TODO)
//!
//! ### 3.a. Assembly (Not yet implemented)
//! - Document metadata: Parser version, file path, timestamps → [`AssemblyInfo`]
//! - Annotation attachment: Critical for the proximity-based annotation system
//! - Final document: Raw AST → fully assembled [`Document`]

mod document_parser;

// Re-export the main entry point
pub use document_parser::{parse_document, DocumentParser};
