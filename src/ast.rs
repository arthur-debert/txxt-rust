//! AST module for TXXT format
//!
//! This module defines a comprehensive, type-safe AST structure for TXXT documents
//! that serves multiple tooling needs including language servers, formatters, linters,
//! and converters.
//!  
//!
//! Core Design Principles
//!
//! - The same ast handles tokens and parsed structure to faciliate language server features,
//!  source mapping, and round-tripping.
//! - Clear separation between containers (indentation-based) and content (semantic elements).
//! - From a type point of view we enforce a homogenous tree model. For example, any element node
//!   can have params or annotations. In practice several combinatios never happen as the language does
//!  not have a syntatical construct for them, but the type system is uniform.
//!
//!
//!   Current Parsing Pipeline
//!   Phase 1: Lexer
//!   1.a. Verbatim Line Marking
//!   - Stateful isolation: Perfect design - verbatim content is sacred and needs special handling
//!   - Critical insight: This is the ONLY stateful part, keeping complexity contained
//!   - AST alignment: Maps to VerbatimContent with exact preservation
//!
//!   1.b. Tokenization
//!   - Token generation: Produces the character-precise tokens needed for language server
//!   - AST alignment: Maps directly to Token enum with SourceSpan positioning
//!
//!   Phase 2: Parser
//!
//!   2.a. Block Grouping
//!   - Indent/dedent processing: Creates the hierarchical structure using container pattern
//!   - Tree of token lists: Perfect for the container indentation architecture
//!   - AST alignment: Maps to Container nodes with proper nesting
//!
//!   2.b. Parsing
//!   - Token list → AST nodes: Converts grouped tokens into semantic structure
//!   - Recursive processing: Handles nested containers correctly
//!   - AST alignment: Produces the rich type-safe AST we designed

//!   Phase 3: Post-Processing

//!   3.a. Assembly (Not yet implemented)
//!   - Document metadata: Parser version, file path, timestamps → AssemblyInfo
//!   - Annotation attachment: Critical for the proximity-based annotation system
//!   - Final document: Raw AST → fully assembled Document

pub mod annotations;
pub mod base;
pub mod blocks;
pub mod inlines;
pub mod parameters;
pub mod reference_types;
pub mod structure;
pub mod tokens;

// Re-export main types for convenience
pub use annotations::*;
pub use base::*;
pub use blocks::*;
pub use inlines::*;
pub use parameters::*;
pub use reference_types::*;
pub use structure::*;
pub use tokens::*;
