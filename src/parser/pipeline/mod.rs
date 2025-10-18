//! Parser Pipeline Components
//!
//! This module implements Phase 2 of the TXXT parsing pipeline that converts
//! token trees into AST element nodes.
//!
//! # Phase 2: Parser (Token-Tree -> AST Tree)
//!
//! ## 2.a Block Parsing
//! - Converts token trees into typed AST nodes for block elements
//! - Handles paragraphs, lists, definitions, annotations, verbatim blocks, sessions
//! - **Input**: `ScannerTokenTree` from lexer (Phase 1c)
//! - **Output**: AST tree of `ElementNode` variants
//!
//! ## 2.b Inline Parsing  
//! - Processes inline elements within block content
//! - Handles formatting, references, links, inline annotations
//! - **Input**: AST tree with block elements (from Phase 2a)
//! - **Output**: Same AST tree with inline elements processed
//!
//! # Pipeline Stages
//!
//! - [`parse_blocks`] - Phase 2a: Block element parsing
//! - [`parse_inlines`] - Phase 2b: Inline element parsing
//!
//! # Current Status
//!
//! Phase 2 is currently stubbed out with placeholder implementations.
//! The actual parsing logic will be implemented as the AST element types
//! and parsing requirements are finalized.

pub mod parse_blocks;
pub mod parse_inlines;
pub mod semantic_analysis;

// Re-export main interfaces
pub use parse_blocks::{BlockParseError, BlockParser};
pub use parse_inlines::{InlineParseError, InlineParser};
pub use semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};
