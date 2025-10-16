//! Parser Pipeline Components
//!
//! This module implements the three-phase TXXT parsing pipeline that converts
//! token streams into fully processed AST structures.
//!
//! # Three-Phase Architecture
//!
//!  Phase 1: Lexer (string -> Stream of positioned tokens)
//!      1.a Verbatim line marking and tokenization
//!      1.b Tokenization-Stream -> Stream of tokens with source positions
//!      1.c Tokenization-Tree -> Convert flat list into a token list tree
//!
//!  Phase 2: Parser  (Token-Tree -> AST Tree)
//!     1.a  Block-Parsing Convert block groups into typed AST nodes-> ast tree of ast element nodes.
//!     1.b Inline-Parsing Handle inlines within blocks (the same tree, but with inlines)
//!
//!  Phase 3: Assembly (AST Tree -> Document)
//! - Document assembly and metadata attachment
//! - Annotation attachment
//!
pub mod block_grouper;
pub mod lexer;
pub mod parser;
pub mod post_processor;

// Re-export main interfaces
pub use block_grouper::{BlockGroup, BlockGrouper};
pub use parser::Parser;
pub use post_processor::PostProcessor;
