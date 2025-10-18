//! Tokenizer Pipeline
//!
//! This module contains the pipeline stages that operate on tokens to produce
//! more structured token representations, but still within the token domain.
//!
//! # Pipeline Stages
//!
//! ## Token Tree Building (Phase 1c)
//! - **Input**: Flat token stream with Indent/Dedent markers
//! - **Output**: Hierarchical token tree (ScannerTokenTree)
//! - **Purpose**: Transform flat token stream into structured token tree
//! - **Location**: `token_tree_builder.rs`
//!
//! This is the final stage of the lexer pipeline, producing a token tree
//! that reflects the document's indentation structure.

pub mod token_tree_builder;

// Re-export main interfaces
pub use token_tree_builder::{ScannerTokenTree, ScannerTokenTreeBuilder, ScannerTokenTreeError};
