//! TXXT Parser Module
//!
//! This module provides the complete parsing infrastructure for converting TXXT tokens
//! into Abstract Syntax Trees (AST). The parser follows a three-phase pipeline design
//! that mirrors the specification structure and maintains perfect alignment with the
//! tokenizer architecture.
//!
//! # Architecture Overview
//!
//! ## Three-Phase Pipeline
//!
//! ### Phase 1: Lexer (COMPLETED)
//! - **Verbatim Line Marking**: Stateful isolation of verbatim content
//! - **Tokenization**: Character-precise token generation
//! - **Location**: Implemented in `src/tokenizer/`
//! - **Output**: Stream of tokens with exact source positioning
//!
//! ### Phase 2: Parser (TO BE IMPLEMENTED)
//! - **Block Grouping**: Hierarchical structure from indentation
//! - **Parsing**: Token sequences → semantic AST nodes  
//! - **Location**: `src/parser/pipeline/`
//! - **Output**: Rich type-safe AST structure
//!
//! ### Phase 3: Post-Processing (PLANNED)
//! - **Assembly**: Document metadata and annotation attachment
//! - **Validation**: Cross-reference resolution and validation
//! - **Location**: `src/parser/pipeline/post_processor.rs`
//! - **Output**: Complete document with all relationships resolved
//!
//! # Module Organization
//!
//! This module follows the established pattern of mirroring the specification
//! structure to ensure perfect alignment between documentation and implementation.
//!
//! ## Core Pipeline (`pipeline/`)
//! - `lexer.rs`: Phase 1 wrapper (delegates to tokenizer)
//! - `block_grouper.rs`: Phase 2a - Convert token stream to block hierarchy
//! - `parser.rs`: Phase 2b - Convert blocks to AST nodes
//! - `post_processor.rs`: Phase 3 - Assembly and final processing
//!
//! ## Element Parsers (`elements/`)
//! **Mirrors `docs/specs/elements/` and `src/tokenizer/` structure:**
//! - `annotation.rs`: Annotation parsing logic
//! - `container.rs`: Container parsing (verbatim, content, session)
//! - `definition.rs`: Definition parsing
//! - `labels.rs`: Label parsing utilities
//! - `list.rs`: List parsing with sequence markers
//! - `paragraph.rs`: Paragraph parsing (default element)
//! - `parameters.rs`: Parameter parsing utilities
//! - `session.rs`: Session parsing with numbering
//! - `verbatim.rs`: Verbatim block parsing
//! - `inlines/`: Inline element parsers (formatting, references)
//!
//! ## Core Utilities (`core/`)
//! **Parsing-specific utilities that complement tokenizer core:**
//! - `indentation.rs`: Indentation analysis for block grouping
//! - `line_grouping.rs`: Line-to-block grouping logic
//! - `span_utils.rs`: Source span manipulation utilities
//!
//! ## Infrastructure (`infrastructure/`)
//! **Parser support systems:**
//! - `error.rs`: Parser-specific error types and handling
//! - `context.rs`: Parser state and context management
//! - `validation.rs`: Semantic validation utilities
//!
//! ## Utilities
//! - `detokenizer.rs`: Round-trip verification (tokens → source)
//!
//! # Testing Integration
//!
//! All parser components integrate with the `TxxtCorpora` specification-driven
//! testing framework. Test cases are extracted directly from `docs/specs/`
//! files using the `:: txxt.core.spec.* ::` syntax.
//!
//! ## Example Usage:
//! ```rust,ignore
//! use tests::corpora::{TxxtCorpora, ProcessingStage};
//!
//! let corpus = TxxtCorpora::load_with_processing(
//!     "txxt.core.spec.paragraph.valid.simple",
//!     ProcessingStage::ParsedAst
//! )?;
//! let ast = corpus.ast().unwrap();
//! ```
//!
//! # Design Principles
//!
//! 1. **Spec Alignment**: Perfect 1:1 mapping between specs and implementation
//! 2. **Phase Separation**: Clear boundaries between pipeline phases
//! 3. **Error Recovery**: Graceful handling of malformed input
//! 4. **Performance**: Efficient single-pass parsing where possible
//! 5. **Testability**: Comprehensive spec-driven test coverage
//!
//! # Implementation Status
//!
//! - ✅ **Phase 1**: Complete (tokenizer)
//! - 🔄 **Phase 2a**: Block grouping (to be implemented)
//! - 🔄 **Phase 2b**: Parsing (to be implemented)  
//! - 📋 **Phase 3**: Post-processing (planned)
//! - 📋 **Detokenizer**: Round-trip verification (planned)

// Pipeline modules
pub mod pipeline;

// Element-specific parsers (mirrors docs/specs/elements/)
pub mod elements;

// Core parsing utilities
pub mod core;

// Parser infrastructure
pub mod infrastructure;

// Utilities
pub mod detokenizer;

// Legacy parser (to be phased out)
mod document_parser;

// Re-export main interfaces
pub use document_parser::{parse_document, DocumentParser};
pub use pipeline::{BlockGrouper, Parser, PostProcessor}; // Legacy interface
