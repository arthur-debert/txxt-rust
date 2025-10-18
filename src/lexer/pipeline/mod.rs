//! Tokenizer Pipeline
//!
//! This module contains the pipeline stages that operate on tokens to produce
//! more structured token representations, but still within the token domain.
//!
//! src/parser/mod.rs has the full architecture overview.

pub mod token_tree_builder;

// Re-export main interfaces
pub use token_tree_builder::{ScannerTokenTree, ScannerTokenTreeBuilder, ScannerTokenTreeError};
