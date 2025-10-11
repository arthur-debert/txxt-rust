//! TXXT Block Grouping
//!
//! This module implements the block grouping stage of the TXXT parser, which organizes tokens
//! into a hierarchical tree of semantic blocks with proper container structures.
//!
//! ## Architecture
//!
//! The block grouping module consists of:
//!
//! ### [`builder`] - Block Tree Construction
//! - [`BlockTreeBuilder`] - main algorithm for converting tokens to blocks
//! - Three-stage process: token tree → blank line splitting → semantic blocks
//! - Handles session detection, list grouping, and container assignment
//! - [`TokenBlock`] - intermediate representation for token groups
//!
//! ## Key Features
//!
//! ### Hierarchical Structure
//! Converts flat token streams into hierarchical block trees:
//! - Groups tokens by indentation levels
//! - Identifies semantic block boundaries
//! - Establishes parent-child relationships
//! - Handles blank line separation
//!
//! ### Block Detection
//! Recognizes all TXXT semantic elements:
//! - Sessions - sections with titles and indented content
//! - Paragraphs - text content blocks
//! - Lists / List Items - structured list content  
//! - Definitions - term definitions with content
//! - Annotations - pragma-style metadata
//! - Verbatim - code/literal blocks
//! - Blank Lines - structural separators
//!
//! ### Integration
//!
//! The block grouping stage fits between tokenization and final parsing:
//!
//! 1. **Tokenizer** → Stream of tokens with INDENT/DEDENT
//! 2. **Block Grouping** → Hierarchical block tree with containers  
//! 3. **Parser** → Final AST with semantic validation
//!
//! This intermediate representation makes the final parsing stage much simpler by:
//! - Resolving structural ambiguities
//! - Establishing container relationships
//! - Organizing content into logical blocks
//!
//! ## Usage
//!
//! ### Basic Block Grouping
//! ```rust
//! use txxt::tokenizer::tokenize;
//! use txxt::block_grouping::build_block_tree;
//!
//! let text = ":: title :: My Document\n\nThis is content.";
//! let tokens = tokenize(text);
//! let block_tree = build_block_tree(tokens);
//! ```
//!
//! ### Advanced Usage
//! ```rust
//! use txxt::block_grouping::BlockTreeBuilder;
//! use txxt::tokenizer::tokenize;
//!
//! let tokens = tokenize("Some TXXT content");
//! let mut builder = BlockTreeBuilder::new(tokens);
//! let block_tree = builder.build();
//! ```
//!
//! ## Implementation Notes
//!
//! ### Ambiguity Resolution
//! The module resolves TXXT's structural ambiguities:
//! - **Sessions vs Paragraphs**: Requires indented children
//! - **Lists vs Paragraphs**: Single items become paragraphs
//! - **Blank Line Handling**: Proper separation and grouping
//!
//! ### Performance
//! - Single-pass token processing
//! - Recursive tree construction
//! - Minimal memory allocation
//! - Efficient blank line splitting

pub mod builder;

#[cfg(test)]
mod tests;

pub use builder::{build_block_tree, BlockTreeBuilder, TokenBlock};
