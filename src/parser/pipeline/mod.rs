//! Parser Pipeline Components
//!
//! This module implements Phase 2 of the TXXT parsing pipeline that converts
//! token trees into AST element nodes.
//!
//! src/parser/mod.rs has the full architecture overview.

pub mod ast_construction;
pub mod parse_inlines;
pub mod semantic_analysis;

// Re-export main interfaces
pub use ast_construction::{AstConstructor, AstNode};
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
