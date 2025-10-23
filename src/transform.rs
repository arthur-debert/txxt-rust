//! Top-Level Process Orchestration
//!
//! This module orchestrates the three main phases of TXXT processing:
//! Phase 1 (Lexer), Phase 2 (Parser), and Phase 3 (Assembler).
//!
//! See the crate-level documentation for the complete architecture overview
//! including detailed phase breakdowns, step definitions, and data flow.
//!
//! Entry Points:
//!
//! - process_lexer: Execute Phase 1 (Lexer)
//! - process_parser: Execute Phase 2 (Parser)
//! - process_assembler: Execute Phase 3 (Assembler)
//! - run_all: Execute all three phases
//! - run_from_file: Convenience function for file input
//!

use crate::assembly::{AnnotationAttacher, DocumentAssembler};
use crate::ast::Document;
use crate::cst::ScannerToken;
use crate::semantic::{AstConstructor, InlineParser};
use crate::syntax::tokenize;
use crate::syntax::SemanticAnalyzer;

/// Processing error type that encompasses all phase errors
#[derive(Debug)]
pub enum TransformError {
    /// Lexer phase error
    Lexer(String),
    /// Parser phase error
    Parser(String),
    /// Assembler phase error
    Assembler(String),
    /// I/O error
    Io(std::io::Error),
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::Lexer(msg) => write!(f, "Lexer error: {}", msg),
            TransformError::Parser(msg) => write!(f, "Parser error: {}", msg),
            TransformError::Assembler(msg) => write!(f, "Assembler error: {}", msg),
            TransformError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for TransformError {}

impl From<std::io::Error> for TransformError {
    fn from(err: std::io::Error) -> Self {
        TransformError::Io(err)
    }
}

/// Execute Phase 1: Lexer
///
/// Converts source text through the lexer steps:
/// Step 1.a: Verbatim Scanning → Step 1.b: Tokenization
///
/// # Arguments
/// * `source_text` - The TXXT source text to process
///
/// # Returns
/// * `Result<Vec<ScannerToken>, TransformError>` - The flat scanner token stream
pub fn run_lexer(source_text: &str) -> Result<Vec<ScannerToken>, TransformError> {
    // Step 1.a: Verbatim Scanning (handled internally by tokenize)
    // Step 1.b: Tokenization
    let tokens = tokenize(source_text);

    Ok(tokens)
}

/// Execute Phase 2: Parser
///
/// Converts scanner tokens through the parser steps:
/// Step 2.a: Semantic Analysis → Step 2.b: AST Construction → Step 2.c: Inline Parsing
///
/// # Arguments
/// * `tokens` - Scanner tokens from Phase 1
///
/// # Returns
/// * `Result<Vec<ElementNode>, TransformError>` - The AST element nodes
pub fn run_parser(
    tokens: Vec<ScannerToken>,
) -> Result<Vec<crate::ast::ElementNode>, TransformError> {
    // Step 2.a: Semantic Analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let semantic_tokens = semantic_analyzer
        .analyze(tokens)
        .map_err(|err| TransformError::Parser(err.to_string()))?;

    // Step 2.b: AST Construction
    let ast_elements = AstConstructor::parse_to_element_nodes(&semantic_tokens)
        .map_err(|err| TransformError::Parser(err.to_string()))?;

    // Step 2.c: Inline Parsing
    let inline_parser = InlineParser::new();
    let ast = inline_parser
        .parse_inlines(ast_elements)
        .map_err(|err| TransformError::Parser(err.to_string()))?;

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
/// * `Result<Document, TransformError>` - The final document
pub fn run_assembler(
    elements: Vec<crate::ast::ElementNode>,
    source_path: Option<String>,
) -> Result<Document, TransformError> {
    // Step 3.a: Document Assembly
    let document_assembler = DocumentAssembler::new();
    let document = document_assembler
        .assemble_document(elements, source_path)
        .map_err(|err| TransformError::Assembler(err.to_string()))?;

    // Step 3.b: Annotation Attachment
    let annotation_attacher = AnnotationAttacher::new();
    let document = annotation_attacher
        .attach_annotations(document)
        .map_err(|err| TransformError::Assembler(err.to_string()))?;

    Ok(document)
}

/// Execute Full Processing: All Three Phases
///
/// Processes source text through the complete three-phase pipeline:
/// String → Vec<ScannerToken> → Vec<HighLevelToken> → Vec<ElementNode> → Document
///
/// # Arguments
/// * `source_text` - The TXXT source text to process
/// * `source_path` - Optional source file path for metadata
///
/// # Returns
/// * `Result<Document, TransformError>` - The final document
pub fn run_all(source_text: &str, source_path: Option<String>) -> Result<Document, TransformError> {
    // Phase 1: Lexer (String → Vec<ScannerToken>)
    let tokens = run_lexer(source_text)?;

    // Phase 2: Parser (Vec<ScannerToken> → Vec<ElementNode>)
    let elements = run_parser(tokens)?;

    // Phase 3: Assembler (AST Elements → Document)
    let document = run_assembler(elements, source_path)?;

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
/// * `Result<Document, TransformError>` - The final document
pub fn run_from_file(file_path: &str) -> Result<Document, TransformError> {
    let source_text = std::fs::read_to_string(file_path)?;
    run_all(&source_text, Some(file_path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_lexer_basic() {
        let source = "Hello, world!";
        let result = run_lexer(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_process_assembler_placeholder() {
        let elements = vec![];
        let result = run_assembler(elements, Some("test.txxt".to_string()));
        assert!(result.is_ok());

        let document = result.unwrap();
        assert_eq!(
            document.assembly_info.source_path,
            Some("test.txxt".to_string())
        );
    }
}
