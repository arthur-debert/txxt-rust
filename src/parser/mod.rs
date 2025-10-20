//! TXXT Parser Module
//!
//! This module provides the parsing infrastructure for converting TXXT tokens
//! into Abstract Syntax Trees (AST). The parser follows a three-phase pipeline design.
//!
//!  Architecture Overview
//!
// 1. Lexer (txxt str -> ScannerTokenList)
//     a. Verbatim Scanner: marks verbatim lines that are off-limits for processing as txxt
//     b. Token List: creates the token stream at low level tokens -> scanner token list
// 2. Parser (ScannerTokenList -> AST tree node)
//     a. Semantic Token Analysis (ScannerTokenList → SemanticTokenList)
//     b. AST Construction : With the ast nodes + dedent construct the final ast tree (SemanticTokenList -> AST tree node) - Buggy
//     c. Inline Parsing: Handle inlines within blocks (ScannerToken -> AST node) -- Stubbed
// 3. Assembly (AST tree node -> AST document node)
//     a. Document Wrapping: wraps the parsed AST in a Document node
//     b. Annotations Attachments: from the content tree to node's annotation's field
//

// Pipeline modules
pub mod pipeline;

// Element parsers
pub mod elements;

// Re-export main interfaces
pub use pipeline::{InlineParseError, InlineParser, SemanticAnalysisError, SemanticAnalyzer};
