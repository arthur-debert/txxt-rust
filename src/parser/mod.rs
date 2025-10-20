//! Phase 2: Parser - AST Construction
//!
//! This module implements the parser phase that converts scanner tokens into AST element nodes.
//! See src/lib.rs for the full architecture overview.
//!
//! - [`semantic_analysis`] - Step 2.a: Semantic token analysis
//!   - Analyzes scanner tokens and produces semantic tokens with meaning attached
//!   - Input: Vec<ScannerToken>
//!   - Output: SemanticTokenList
//!
//! - [`ast_construction`] - Step 2.b: AST tree construction
//!   - Builds AST tree from semantic tokens
//!   - Input: SemanticTokenList
//!   - Output: Vec<ElementNode>
//!
//! - [`inline_parsing`] - Step 2.c: Inline element parsing
//!   - Parses inline formatting within text content
//!   - Input: Vec<ElementNode> with unparsed inline text
//!   - Output: Vec<ElementNode> with parsed inline elements
//!
//! ## Element Modules (Organized by Specification)
//!
//! - [`elements`] - All element parsing organized by type
//!   - [`elements::formatting`] - Text formatting elements (bold, italic, code, math)
//!   - [`elements::inlines`] - Inline element parsing with text transform layer
//!     - [`elements::inlines::references`] - Reference and citation elements
//!     - Citation parsing, footnote parsing, page references, session references
//!

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
