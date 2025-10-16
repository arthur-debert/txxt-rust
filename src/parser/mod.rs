//! TXXT Parser Module
//!
//! This module provides the parsing infrastructure for converting TXXT tokens
//! into Abstract Syntax Trees (AST). The parser follows a three-phase pipeline design.
//!
//! # Architecture Overview
//!
//! ## Three-Phase Pipeline
//!
//! ### Phase 1: Lexer (COMPLETED ✅)
//! - **Verbatim Line Marking**: Stateful isolation of verbatim content
//! - **Tokenization**: Character-precise token generation
//! - **Token Tree**: Hierarchical block grouping from indentation
//! - **Location**: Implemented in `src/tokenizer/` and `src/parser/pipeline/block_grouper.rs`
//! - **Output**: Stream of tokens with exact source positioning + hierarchical block structure
//!
//! ### Phase 2: Parser (TO BE IMPLEMENTED 🔄)
//! - **Block Parsing**: Convert block groups into typed AST nodes
//! - **Inline Parsing**: Handle inlines within blocks
//! - **Location**: `src/parser/pipeline/` (to be implemented)
//! - **Output**: Rich type-safe AST structure
//!
//! ### Phase 3: Assembly (IMPLEMENTED ✅)
//! - **Document Assembly**: Document metadata and annotation attachment
//! - **Validation**: Cross-reference resolution and validation
//! - **Location**: `src/assembler/`
//! - **Output**: Complete document with all relationships resolved
//!
//! # Current Implementation Status
//!
//! - ✅ **Phase 1**: Complete (tokenizer + block grouping)
//! - 🔄 **Phase 2**: To be implemented (parser stubs removed)
//! - ✅ **Phase 3**: Complete (assembler implemented)
//!
//! # Usage
//!
//! ```rust,ignore
//! use txxt::assembler::Assembler;
//! use txxt::tokenizer::pipeline::BlockGrouper;
//! use txxt::tokenizer::tokenize;
//!
//! // Phase 1: Tokenization + Block Grouping (working)
//! let tokens = tokenize(input_text);
//! let block_grouper = BlockGrouper::new();
//! let blocks = block_grouper.group_blocks(tokens)?;
//!
//! // Phase 2: Parsing (to be implemented)
//! // let parser = Parser::new();
//! // let ast = parser.parse(blocks)?;
//!
//! // Phase 3: Assembly (working)
//! let assembler = Assembler::new();
//! let document = assembler.assemble_document(blocks, Some("source.txxt".to_string()))?;
//! ```

// Pipeline modules
pub mod pipeline;

// Re-export main interfaces
// Note: BlockGroup and BlockGrouper moved to src/tokenizer/pipeline/
