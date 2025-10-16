//! Parser Pipeline Components
//!
//! This module implements the three-phase TXXT parsing pipeline that converts
//! token streams into fully processed AST structures.
//!
//! # Three-Phase Architecture
//!
//!  Phase 1: Lexer (string -> Stream of positioned tokens)
//!      a Verbatim line marking and tokenization
//!      b Tokenization-Stream -> Stream of tokens with source positions
//!      c Tokenization-Tree -> Convert flat list into a token list tree
//!
//!  Phase 2: Parser  (Token-Tree -> AST Tree)
//!     a  Block-Parsing Convert block groups into typed AST nodes-> ast tree of ast element nodes.
//!     b Inline-Parsing Handle inlines within blocks (the same tree, but with inlines)
//!
//!  Phase 3: Assembly (AST Tree -> Document)
//!     a Document assembly (may include non content related metadata)
//!     b Annotation attachment
//!
pub mod block_grouper;
pub mod lexer;
pub mod parser;
pub mod post_processor;

// Re-export main interfaces
pub use block_grouper::{BlockGroup, BlockGrouper};
pub use parser::Parser;
pub use post_processor::PostProcessor;
