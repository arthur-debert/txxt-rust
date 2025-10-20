//! Phase 2: Parser - AST Construction
//!
//! This module implements the parser phase that converts scanner tokens into AST element nodes.
//!
//! See the crate-level documentation for the complete architecture overview.
//!
//! Parser Steps:
//!
//! - Step 2.a: Semantic analysis - analyzes scanner tokens and produces semantic tokens
//! - Step 2.b: AST construction - builds AST tree from semantic tokens
//! - Step 2.c: Inline parsing - parses inline formatting within text content
//!
//! Processing Steps:
//!
//! - [`semantic_analysis`] - Step 2.a: Semantic token analysis
//! - [`ast_construction`] - Step 2.b: AST tree construction
//! - [`inline_parsing`] - Step 2.c: Inline element parsing
//!
//! Element Parsers:
//!
//! - [`elements`] - Element-specific parsing logic

// Processing steps
pub mod ast_construction;
pub mod inline_parsing;
pub mod semantic_analysis;

// Element parsers
pub mod elements;

// Re-export main interfaces
pub use ast_construction::{AstConstructor, AstNode};
pub use inline_parsing::{InlineParseError, InlineParser};
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
