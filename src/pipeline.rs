//! Global Pipeline Orchestration
//!
//! This module provides the centralized pipeline that orchestrates the three
//! main phases of TXXT processing: Lexer, Parser, and Assembler.
//!
//! # Three-Phase Pipeline Architecture
//!
//! The TXXT processing pipeline follows a three-phase architecture:
//!
//! ## Phase 1: Lexer (String → Token Tree)
//! - **1.a Verbatim Scanning**: Mark verbatim lines in the source text
//! - **1.b Tokenization**: Convert text to stream of positioned tokens
//! - **1.c Token Tree Building**: Convert flat token stream to hierarchical tree
//!
//! ## Phase 2: Parser (Token Tree → AST Tree)
//! - **2.a Block Parsing**: Convert token tree to AST block elements
//! - **2.b Inline Parsing**: Process inline elements within blocks
//!
//! ## Phase 3: Assembler (AST Tree → Document)
//! - **3.a Document Assembly**: Wrap AST in Session container and Document node
//! - **3.b Annotation Attachment**: Apply proximity rules to attach annotations
//!
//! # Pipeline Stages
//!
//! Each phase can be executed independently for testing and debugging:
//! - `lexer_pipeline()` - Execute Phase 1 (Lexer) only
//! - `parser_pipeline()` - Execute Phase 2 (Parser) only  
//! - `assembler_pipeline()` - Execute Phase 3 (Assembler) only
//! - `full_pipeline()` - Execute all phases in sequence
//!
//! # Usage
//!
//! ```rust,ignore
//! use txxt::pipeline::{full_pipeline, lexer_pipeline};
//!
//! // Full pipeline: String → Document
//! let document = full_pipeline("Hello, world!")?;
//!
//! // Partial pipeline: String → ScannerTokenTree (for testing)
//! let token_tree = lexer_pipeline("Hello, world!")?;
//! ```

use crate::assembler::pipeline::{AnnotationAttacher, DocumentAssembler};
use crate::ast::base::Document;
use crate::lexer::pipeline::ScannerTokenTreeBuilder;
use crate::lexer::tokenize;
use crate::parser::pipeline::{BlockParser, InlineParser};

/// Global pipeline error type that encompasses all phase errors
#[derive(Debug)]
pub enum PipelineError {
    /// Lexer phase error
    Lexer(String),
    /// Parser phase error  
    Parser(String),
    /// Assembler phase error
    Assembler(String),
    /// I/O error
    Io(std::io::Error),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::Lexer(msg) => write!(f, "Lexer error: {}", msg),
            PipelineError::Parser(msg) => write!(f, "Parser error: {}", msg),
            PipelineError::Assembler(msg) => write!(f, "Assembler error: {}", msg),
            PipelineError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for PipelineError {}

impl From<std::io::Error> for PipelineError {
    fn from(err: std::io::Error) -> Self {
        PipelineError::Io(err)
    }
}

/// Execute Phase 1: Lexer Pipeline
///
/// Converts source text through the lexer phases:
/// 1.a Verbatim Scanning → 1.b Tokenization → 1.c Token Tree Building
///
/// # Arguments
/// * `source_text` - The TXXT source text to process
///
/// # Returns
/// * `Result<ScannerTokenTree, PipelineError>` - The hierarchical token tree
pub fn lexer_pipeline(
    source_text: &str,
) -> Result<crate::lexer::pipeline::ScannerTokenTree, PipelineError> {
    // Phase 1.a: Verbatim Scanning (handled internally by tokenize)
    // Phase 1.b: Tokenization
    let tokens = tokenize(source_text);

    // Phase 1.c: Token Tree Building
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|err| PipelineError::Lexer(err.to_string()))?;

    Ok(token_tree)
}

/// Execute Phase 2: Parser Pipeline
///
/// Converts token tree through the parser phases:
/// 2.a Block Parsing → 2.b Inline Parsing
///
/// # Arguments
/// * `token_tree` - The hierarchical token tree from Phase 1
///
/// # Returns
/// * `Result<Vec<ElementNode>, PipelineError>` - The AST element nodes
pub fn parser_pipeline(
    token_tree: crate::lexer::pipeline::ScannerTokenTree,
) -> Result<Vec<crate::ast::ElementNode>, PipelineError> {
    // Phase 2.a: Block Parsing
    let block_parser = BlockParser::new();
    let blocks = block_parser
        .parse_blocks(token_tree)
        .map_err(|err| PipelineError::Parser(err.to_string()))?;

    // Phase 2.b: Inline Parsing
    let inline_parser = InlineParser::new();
    let ast = inline_parser
        .parse_inlines(blocks)
        .map_err(|err| PipelineError::Parser(err.to_string()))?;

    Ok(ast)
}

/// Execute Phase 3: Assembler Pipeline
///
/// Converts AST elements through the assembler phases:
/// 3.a Document Assembly → 3.b Annotation Attachment
///
/// # Arguments
/// * `elements` - The AST element nodes from Phase 2
/// * `source_path` - Optional source file path for metadata
///
/// # Returns
/// * `Result<Document, PipelineError>` - The final document
pub fn assembler_pipeline(
    elements: Vec<crate::ast::ElementNode>,
    source_path: Option<String>,
) -> Result<Document, PipelineError> {
    // Phase 3.a: Document Assembly
    let document_assembler = DocumentAssembler::new();
    let document = document_assembler
        .assemble_document(elements, source_path)
        .map_err(|err| PipelineError::Assembler(err.to_string()))?;

    // Phase 3.b: Annotation Attachment
    let annotation_attacher = AnnotationAttacher::new();
    let document = annotation_attacher
        .attach_annotations(document)
        .map_err(|err| PipelineError::Assembler(err.to_string()))?;

    Ok(document)
}

/// Execute Full Pipeline: All Three Phases
///
/// Processes source text through the complete pipeline:
/// String → ScannerTokenTree → AST Elements → Document
///
/// # Arguments
/// * `source_text` - The TXXT source text to process
/// * `source_path` - Optional source file path for metadata
///
/// # Returns
/// * `Result<Document, PipelineError>` - The final document
pub fn full_pipeline(
    source_text: &str,
    source_path: Option<String>,
) -> Result<Document, PipelineError> {
    // Phase 1: Lexer (String → ScannerTokenTree)
    let token_tree = lexer_pipeline(source_text)?;

    // Phase 2: Parser (ScannerTokenTree → AST Elements)
    let elements = parser_pipeline(token_tree)?;

    // Phase 3: Assembler (AST Elements → Document)
    let document = assembler_pipeline(elements, source_path)?;

    Ok(document)
}

/// Execute Full Pipeline with File Input
///
/// Convenience function that reads from a file and processes it through
/// the complete pipeline.
///
/// # Arguments
/// * `file_path` - Path to the TXXT file to process
///
/// # Returns
/// * `Result<Document, PipelineError>` - The final document
pub fn pipeline_from_file(file_path: &str) -> Result<Document, PipelineError> {
    let source_text = std::fs::read_to_string(file_path)?;
    full_pipeline(&source_text, Some(file_path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_pipeline_basic() {
        let source = "Hello, world!";
        let result = lexer_pipeline(source);
        assert!(result.is_ok());

        let token_tree = result.unwrap();
        assert!(!token_tree.tokens.is_empty());
    }

    #[test]
    fn test_parser_pipeline_placeholder() {
        let token_tree = crate::lexer::pipeline::ScannerTokenTree {
            tokens: vec![],
            children: vec![],
        };

        let result = parser_pipeline(token_tree);
        assert!(result.is_ok());

        let elements = result.unwrap();
        assert!(elements.is_empty()); // Placeholder returns empty
    }

    #[test]
    fn test_assembler_pipeline_placeholder() {
        let elements = vec![];
        let result = assembler_pipeline(elements, Some("test.txxt".to_string()));
        assert!(result.is_ok());

        let document = result.unwrap();
        assert_eq!(
            document.assembly_info.source_path,
            Some("test.txxt".to_string())
        );
    }

    #[test]
    fn test_full_pipeline_placeholder() {
        let source = "Hello, world!";
        let result = full_pipeline(source, Some("test.txxt".to_string()));
        assert!(result.is_ok());

        let document = result.unwrap();
        assert_eq!(
            document.assembly_info.source_path,
            Some("test.txxt".to_string())
        );
        assert_eq!(
            document.assembly_info.parser_version,
            env!("CARGO_PKG_VERSION")
        );
    }
}
