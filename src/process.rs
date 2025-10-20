//! Top-Level Process Orchestration
//!
//! This module orchestrates the three main phases of TXXT processing:
//! Phase 1 (Lexer), Phase 2 (Parser), and Phase 3 (Assembler).
//!
//! ## Terminology
//!
//! - **Phase**: One of three high-level stages (Lexer, Parser, Assembler)
//! - **Step**: Sub-operations within a phase (e.g., semantic analysis, AST construction)
//! - **Stage**: CLI/test concept for where to stop processing for inspection
//!
//! ## Usage
//!
//! ```rust,ignore
//! use txxt::process::process_full;
//!
//! let document = process_full(source_text, Some("file.txxt".to_string()))?;
//! ```

use crate::assembler::{AnnotationAttacher, DocumentAssembler};
use crate::ast::Document;
use crate::lexer::{tokenize, ScannerTokenTree, ScannerTokenTreeBuilder};
use crate::parser::{AstConstructor, InlineParser, SemanticAnalyzer};

/// Processing error type that encompasses all phase errors
#[derive(Debug)]
pub enum ProcessError {
    /// Lexer phase error
    Lexer(String),
    /// Parser phase error
    Parser(String),
    /// Assembler phase error
    Assembler(String),
    /// I/O error
    Io(std::io::Error),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::Lexer(msg) => write!(f, "Lexer error: {}", msg),
            ProcessError::Parser(msg) => write!(f, "Parser error: {}", msg),
            ProcessError::Assembler(msg) => write!(f, "Assembler error: {}", msg),
            ProcessError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<std::io::Error> for ProcessError {
    fn from(err: std::io::Error) -> Self {
        ProcessError::Io(err)
    }
}

/// Execute Phase 1: Lexer
///
/// Converts source text through the lexer steps:
/// Step 1.a: Verbatim Scanning → Step 1.b: Tokenization → Step 1.c: Token Tree Building
///
/// # Arguments
/// * `source_text` - The TXXT source text to process
///
/// # Returns
/// * `Result<ScannerTokenTree, ProcessError>` - The hierarchical token tree
pub fn process_lexer(source_text: &str) -> Result<ScannerTokenTree, ProcessError> {
    // Step 1.a: Verbatim Scanning (handled internally by tokenize)
    // Step 1.b: Tokenization
    let tokens = tokenize(source_text);

    // Step 1.c: Token Tree Building
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|err| ProcessError::Lexer(err.to_string()))?;

    Ok(token_tree)
}

/// Execute Phase 2: Parser
///
/// Converts scanner tokens through the parser steps:
/// Step 2.a: Semantic Analysis → Step 2.b: AST Construction → Step 2.c: Inline Parsing
///
/// # Arguments
/// * `token_tree` - Scanner token tree from Phase 1
///
/// # Returns
/// * `Result<Vec<ElementNode>, ProcessError>` - The AST element nodes
pub fn process_parser(
    token_tree: ScannerTokenTree,
) -> Result<Vec<crate::ast::ElementNode>, ProcessError> {
    // Extract tokens from the tree for semantic analysis
    let tokens = token_tree.tokens;

    // Step 2.a: Semantic Analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let semantic_tokens = semantic_analyzer
        .analyze(tokens)
        .map_err(|err| ProcessError::Parser(err.to_string()))?;

    // Step 2.b: AST Construction
    let ast_elements = AstConstructor::parse_to_element_nodes(&semantic_tokens)
        .map_err(|err| ProcessError::Parser(err.to_string()))?;

    // Step 2.c: Inline Parsing
    let inline_parser = InlineParser::new();
    let ast = inline_parser
        .parse_inlines(ast_elements)
        .map_err(|err| ProcessError::Parser(err.to_string()))?;

    Ok(ast)
}

/// Execute Phase 3: Assembler
///
/// Converts AST elements through the assembler steps:
/// Step 3.a: Document Assembly → Step 3.b: Annotation Attachment
///
/// # Arguments
/// * `elements` - The AST element nodes from Phase 2
/// * `source_path` - Optional source file path for metadata
///
/// # Returns
/// * `Result<Document, ProcessError>` - The final document
pub fn process_assembler(
    elements: Vec<crate::ast::ElementNode>,
    source_path: Option<String>,
) -> Result<Document, ProcessError> {
    // Step 3.a: Document Assembly
    let document_assembler = DocumentAssembler::new();
    let document = document_assembler
        .assemble_document(elements, source_path)
        .map_err(|err| ProcessError::Assembler(err.to_string()))?;

    // Step 3.b: Annotation Attachment
    let annotation_attacher = AnnotationAttacher::new();
    let document = annotation_attacher
        .attach_annotations(document)
        .map_err(|err| ProcessError::Assembler(err.to_string()))?;

    Ok(document)
}

/// Execute Full Processing: All Three Phases
///
/// Processes source text through the complete three-phase pipeline:
/// String → ScannerTokenTree → AST Elements → Document
///
/// # Arguments
/// * `source_text` - The TXXT source text to process
/// * `source_path` - Optional source file path for metadata
///
/// # Returns
/// * `Result<Document, ProcessError>` - The final document
pub fn process_full(
    source_text: &str,
    source_path: Option<String>,
) -> Result<Document, ProcessError> {
    // Phase 1: Lexer (String → ScannerTokenTree)
    let token_tree = process_lexer(source_text)?;

    // Phase 2: Parser (ScannerTokenTree → AST Elements)
    let elements = process_parser(token_tree)?;

    // Phase 3: Assembler (AST Elements → Document)
    let document = process_assembler(elements, source_path)?;

    Ok(document)
}

/// Execute Full Processing with File Input
///
/// Convenience function that reads from a file and processes it through
/// all three phases.
///
/// # Arguments
/// * `file_path` - Path to the TXXT file to process
///
/// # Returns
/// * `Result<Document, ProcessError>` - The final document
pub fn process_from_file(file_path: &str) -> Result<Document, ProcessError> {
    let source_text = std::fs::read_to_string(file_path)?;
    process_full(&source_text, Some(file_path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_lexer_basic() {
        let source = "Hello, world!";
        let result = process_lexer(source);
        assert!(result.is_ok());

        let token_tree = result.unwrap();
        assert!(!token_tree.tokens.is_empty());
    }

    #[test]
    fn test_process_parser_placeholder() {
        let token_tree = ScannerTokenTree {
            tokens: vec![],
            children: vec![],
        };

        let result = process_parser(token_tree);
        assert!(result.is_ok());

        let elements = result.unwrap();
        assert!(elements.is_empty()); // Placeholder returns empty
    }

    #[test]
    fn test_process_assembler_placeholder() {
        let elements = vec![];
        let result = process_assembler(elements, Some("test.txxt".to_string()));
        assert!(result.is_ok());

        let document = result.unwrap();
        assert_eq!(
            document.assembly_info.source_path,
            Some("test.txxt".to_string())
        );
    }

    #[test]
    fn test_process_full_placeholder() {
        let source = "Hello, world!";
        let result = process_full(source, Some("test.txxt".to_string()));
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
