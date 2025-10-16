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
//! - **Location**: Implemented in `src/lexer/`
//! - **Output**: Stream of tokens with exact source positioning + hierarchical block structure
//!
//! ### Phase 2: Parser (STRUCTURED 🔄)
//! - **Block Parsing**: Convert token trees into typed AST nodes
//! - **Inline Parsing**: Handle inlines within blocks
//! - **Location**: `src/parser/pipeline/` (structured, implementation pending)
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
//! - ✅ **Phase 1**: Complete (lexer + token tree building)
//! - 🔄 **Phase 2**: Structured (parser pipeline ready, implementation pending)
//! - ✅ **Phase 3**: Complete (assembler implemented)
//!
//! # Usage
//!
//! ```rust,ignore
//! use txxt::assembler::Assembler;
//! use txxt::lexer::pipeline::TokenTreeBuilder;
//! use txxt::lexer::tokenize;
//! use txxt::parser::pipeline::{BlockParser, InlineParser};
//!
//! // Phase 1: Lexer (working)
//! let tokens = tokenize(input_text);
//! let token_tree_builder = TokenTreeBuilder::new();
//! let token_tree = token_tree_builder.build_tree(tokens)?;
//!
//! // Phase 2: Parser (structured, implementation pending)
//! let block_parser = BlockParser::new();
//! let blocks = block_parser.parse_blocks(token_tree)?;
//! let inline_parser = InlineParser::new();
//! let ast = inline_parser.parse_inlines(blocks)?;
//!
//! // Phase 3: Assembly (working)
//! let assembler = Assembler::new();
//! let document = assembler.assemble_document(token_tree, Some("source.txxt".to_string()))?;
//! ```

// Pipeline modules
pub mod pipeline;

// Re-export main interfaces
pub use pipeline::{BlockParseError, BlockParser, InlineParseError, InlineParser};
