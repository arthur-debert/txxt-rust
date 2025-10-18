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
//! - **Verbatim Scanner**: marks verbatim lines that are off-limits for processing as txxt
//! - **Token List**: creates the token stream at low level tokens -> scanner token list
//! - **Location**: Implemented in `src/lexer/`
//! - **Output**: Stream of tokens with exact source positioning
//!
//! ### Phase 2: Parser (ScannerTokenList -> AST tree node)
//! - **2.a Semantic Token Analysis**: ScannerTokenList → SemanticTokenList
//! - **2.b AST Construction**: With the ast nodes + dedent construct the final ast tree
//! - **2.c Inline Parsing**: Handle inlines within blocks (ScannerToken -> AST node)
//! - **Location**: `src/parser/pipeline/` (2.a and 2.c implemented, 2.b pending)
//! - **Output**: Rich type-safe AST structure
//!
//! ### Phase 3: Assembly (AST tree node -> AST document node)
//! - **Document Wrapping**: wraps the parsed AST in a Document node
//! - **Annotations Attachments**: from the content tree to node's annotation's field
//! - **Location**: `src/assembler/`
//! - **Output**: Complete document with all relationships resolved
//!
//! # Current Implementation Status
//!
//! - ✅ **Phase 1**: Complete (lexer)
//! - 🔄 **Phase 2**: Partial (2.a semantic analysis ✅, 2.b AST construction ⏳, 2.c inline parsing ✅)
//! - ✅ **Phase 3**: Complete (assembler)
//!
//! # Usage
//!
//! ```rust,ignore
//! use txxt::assembler::Assembler;
//! use txxt::lexer::tokenize;
//! use txxt::parser::pipeline::{SemanticAnalyzer, InlineParser};
//!
//! // Phase 1: Lexer (working)
//! let scanner_tokens = tokenize(input_text);
//!
//! // Phase 2: Parser (2.a semantic analysis working, 2.b pending, 2.c working)
//! let semantic_analyzer = SemanticAnalyzer::new();
//! let semantic_tokens = semantic_analyzer.analyze(scanner_tokens)?;
//! // TODO: Phase 2.b AST Construction (pending implementation)
//! let inline_parser = InlineParser::new();
//! let ast = inline_parser.parse_inlines(semantic_tokens)?;
//!
//! // Phase 3: Assembly (working)
//! let assembler = Assembler::new();
//! let document = assembler.assemble_document(ast, Some("source.txxt".to_string()))?;
//! ```

// Pipeline modules
pub mod pipeline;

// Element parsers
pub mod elements;

// Re-export main interfaces
pub use pipeline::{InlineParseError, InlineParser, SemanticAnalysisError, SemanticAnalyzer};
