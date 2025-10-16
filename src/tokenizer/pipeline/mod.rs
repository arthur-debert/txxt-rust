//! Tokenizer Pipeline
//!
//! This module contains the pipeline stages that operate on tokens to produce
//! more structured token representations, but still within the token domain.
//!
//! # Pipeline Stages
//!
//! ## Block Grouping (Phase 1c)
//! - **Input**: Flat token stream with Indent/Dedent markers
//! - **Output**: Hierarchical token tree (BlockGroup)
//! - **Purpose**: Transform flat token stream into structured token tree
//! - **Location**: `block_grouper.rs`
//!
//! This is the final stage of the lexer pipeline, producing a token tree
//! that reflects the document's indentation structure.

pub mod block_grouper;

// Re-export main interfaces
pub use block_grouper::{BlockGroup, BlockGroupError, BlockGrouper};
