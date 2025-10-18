//! Parser Pipeline Components
//!
//! This module implements Phase 2 of the TXXT parsing pipeline that converts
//! token trees into AST element nodes.
//!
//! # Phase 2: Parser (ScannerTokenList -> AST Tree)
//!
//! ## 2.a Semantic Token Analysis (ScannerTokenList → SemanticTokenList)
//! - Converts scanner tokens into semantic tokens
//! - Handles composition patterns and token grouping
//! - **Input**: `ScannerTokenList` from lexer (Phase 1)
//! - **Output**: `SemanticTokenList` with structured components
//!
//! ## 2.b AST Construction (SemanticTokenList → AST Tree)
//! - Converts semantic tokens into AST element nodes
//! - Handles block elements: paragraphs, lists, definitions, annotations, verbatim blocks, sessions
//! - **Input**: `SemanticTokenList` from Phase 2a
//! - **Output**: AST tree of `ElementNode` variants
//!
//! ## 2.c Inline Parsing (ScannerToken -> AST node)
//! - Processes inline elements within block content
//! - Handles formatting, references, links, inline annotations
//! - **Input**: AST tree with block elements (from Phase 2b)
//! - **Output**: Same AST tree with inline elements processed
//!
//! # Pipeline Stages
//!
//! - [`semantic_analysis`] - Phase 2a: Semantic token analysis
//! - [`parse_inlines`] - Phase 2c: Inline element parsing
//!
//! # Current Status
//!
//! Phase 2a (Semantic Analysis) is implemented.
//! Phase 2b (AST Construction) is pending implementation.
//! Phase 2c (Inline Parsing) is implemented.

pub mod parse_inlines;
pub mod semantic_analysis;

// Re-export main interfaces
pub use parse_inlines::{InlineParseError, InlineParser};
pub use semantic_analysis::{SemanticAnalysisError, SemanticAnalyzer};

/// Error type for block element parsing
#[derive(Debug)]
pub enum BlockParseError {
    InvalidStructure(String),
}

impl std::fmt::Display for BlockParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockParseError::InvalidStructure(msg) => {
                write!(f, "Invalid block structure: {}", msg)
            }
        }
    }
}

impl std::error::Error for BlockParseError {}
