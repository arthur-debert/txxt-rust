//! Parser Pipeline Components
//!
//! This module implements the three-phase TXXT parsing pipeline that converts
//! token streams into fully processed AST structures.
//!
//! # Three-Phase Architecture
//!
//! ## Phase 1: Lexer (Completed - in tokenizer)
//! - Verbatim line marking and tokenization
//! - Implemented in `src/tokenizer/`
//! - Output: Stream of positioned tokens
//!
//! ## Phase 2: Parser (To be implemented)
//! ### Phase 2a: Block Grouping
//! - Convert flat token stream into hierarchical block structure
//! - Handle indentation-based nesting
//! - Create container boundaries
//!
//! ### Phase 2b: Parsing  
//! - Convert block groups into typed AST nodes
//! - Apply element-specific parsing rules
//! - Handle inline processing within blocks
//!
//! ## Phase 3: Post-Processing (Planned)
//! - Document assembly and metadata attachment
//! - Cross-reference resolution
//! - Annotation proximity processing
//!
//! # Design Principles
//!
//! - **Single-pass processing**: Each phase processes input once
//! - **Error recovery**: Graceful handling of malformed input
//! - **Incremental**: Foundation for future incremental parsing
//! - **Testable**: Each phase can be tested independently

pub mod block_grouper;
pub mod lexer;
pub mod parser;
pub mod post_processor;

// Re-export main interfaces
pub use block_grouper::BlockGrouper;
pub use parser::Parser;
pub use post_processor::PostProcessor;
