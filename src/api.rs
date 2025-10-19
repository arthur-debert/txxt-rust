//! TXXT Processing API
//!
//! Pure functions for processing TXXT content through the three-phase pipeline.
//! This module contains no I/O operations, CLI handling, or process exits.
//! All functions take structured input and return structured output for easy testing.
//!
//! src/parser/mod.rs has the full architecture overview.

use serde_json;
use std::error::Error;
use std::fmt;

use crate::lexer::elements::verbatim::verbatim_scanner::VerbatimScanner;
use crate::lexer::pipeline::ScannerTokenTreeBuilder;
use crate::lexer::tokenize;
use crate::parser::pipeline::{InlineParser, SemanticAnalyzer};

#[derive(Debug, Clone)]
pub struct ProcessArgs {
    pub content: String,
    pub source_path: String,
    pub stage: String,
    pub format: String,
}

#[derive(Debug)]
pub enum ProcessError {
    TokenizationError(String),
    ParseError(String),
    AssemblyError(String),
    SerializationError(String),
    NotImplemented(String),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::TokenizationError(msg) => write!(f, "Tokenization error: {}", msg),
            ProcessError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ProcessError::AssemblyError(msg) => write!(f, "Assembly error: {}", msg),
            ProcessError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ProcessError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
        }
    }
}

impl Error for ProcessError {}

/// Main processing function - pure, no I/O or side effects
pub fn process(args: ProcessArgs) -> Result<String, ProcessError> {
    match (args.stage.as_str(), args.format.as_str()) {
        ("scanner-tokens", "json") => process_token_stream(&args.content, &args.source_path),
        ("semantic-tokens", "json") => process_semantic_tokens(&args.content, &args.source_path),
        ("ast-block", "json") => process_ast_no_inline_json(&args.content, &args.source_path),
        ("ast-block", "treeviz") => process_ast_no_inline_treeviz(&args.content, &args.source_path),
        ("ast-inlines", "json") => process_ast_json(&args.content, &args.source_path),
        ("ast-inlines", "treeviz") => process_ast_treeviz(&args.content, &args.source_path),
        ("ast-document", "json") => process_ast_full_json(&args.content, &args.source_path),
        ("ast-document", "treeviz") => process_ast_full_treeviz(&args.content, &args.source_path),
        ("ast-full", "json") => process_ast_full_json(&args.content, &args.source_path),
        ("ast-full", "treeviz") => process_ast_full_treeviz(&args.content, &args.source_path),
        _ => Err(ProcessError::NotImplemented(format!(
            "Combination of stage '{}' and format '{}' is not supported",
            args.stage, args.format
        ))),
    }
}

// Phase 1 processing functions
fn process_token_stream(content: &str, source_path: &str) -> Result<String, ProcessError> {
    let tokens = tokenize(content);

    let result = serde_json::json!({
        "source": source_path,
        "tokens": tokens
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

// Phase 2a processing functions
fn process_semantic_tokens(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Step 1: Apply lexer to get scanner tokens
    let scanner_tokens = tokenize(content);

    // Step 2: Apply semantic analysis to get semantic tokens
    let analyzer = SemanticAnalyzer::new();
    let semantic_tokens = analyzer
        .analyze(scanner_tokens)
        .map_err(|e| ProcessError::ParseError(format!("Semantic analysis error: {}", e)))?;

    // Step 3: Create output with source information
    let result = serde_json::json!({
        "source": source_path,
        "semantic_tokens": semantic_tokens.tokens
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

// Phase 3 processing functions
fn process_ast_full_json(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let _token_tree = token_tree_builder
        .build_tree(tokens.clone())
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Semantic analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let _semantic_tokens = semantic_analyzer
        .analyze(tokens)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: AST Construction (pending implementation)
    // TODO: Implement AST construction phase
    let ast_elements: Vec<crate::ast::ElementNode> = vec![]; // Placeholder

    // Phase 2c: Parse inlines (stubbed - returns unchanged)
    let inline_parser = InlineParser::new();
    let ast_elements_with_inlines = inline_parser
        .parse_inlines(ast_elements)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 3: Create document structure with parsed AST elements
    let result = serde_json::json!({
        "source": source_path,
        "document": {
            "content": {
                "elements": ast_elements_with_inlines,
                "element_count": ast_elements_with_inlines.len()
            },
            "assembly_info": {
                "parser_version": env!("CARGO_PKG_VERSION"),
                "processed_at": chrono::Utc::now().to_rfc3339()
            }
        }
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

fn process_ast_full_treeviz(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let _token_tree = token_tree_builder
        .build_tree(tokens.clone())
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Semantic analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let _semantic_tokens = semantic_analyzer
        .analyze(tokens)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: AST Construction (pending implementation)
    // TODO: Implement AST construction phase
    let ast_elements: Vec<crate::ast::ElementNode> = vec![]; // Placeholder

    // Phase 2c: Parse inlines (stubbed - returns unchanged)
    let inline_parser = InlineParser::new();
    let ast_elements_with_inlines = inline_parser
        .parse_inlines(ast_elements)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Convert ElementNode to SessionContainerElement
    let container_elements: Vec<crate::ast::elements::session::session_container::SessionContainerElement> = ast_elements_with_inlines
        .into_iter()
        .map(|element| match element {
            crate::ast::ElementNode::ParagraphBlock(paragraph) => {
                crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(paragraph)
            }
            crate::ast::ElementNode::SessionBlock(session) => {
                crate::ast::elements::session::session_container::SessionContainerElement::Session(session)
            }
            // For now, we'll convert other types to paragraphs as placeholders
            // TODO: Implement proper conversion for all ElementNode types
            _ => {
                // Create a placeholder paragraph for unsupported element types
                let placeholder_paragraph = crate::ast::elements::paragraph::block::ParagraphBlock {
                    content: vec![], // Empty content for now
                    annotations: Vec::new(),
                    parameters: crate::ast::elements::components::parameters::Parameters::new(),
                    tokens: crate::ast::scanner_tokens::ScannerTokenSequence::new(),
                };
                crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(placeholder_paragraph)
            }
        })
        .collect();

    // Create a document root node containing all the parsed elements
    let document_root = crate::ast::ElementNode::SessionContainer(
        crate::ast::elements::session::session_container::SessionContainer {
            content: container_elements,
            annotations: Vec::new(),
            parameters: crate::ast::elements::components::parameters::Parameters::new(),
            tokens: crate::ast::scanner_tokens::ScannerTokenSequence::new(),
        },
    );

    // Use the proper treeviz system to render the AST
    let treeviz_output = crate::tools::treeviz::ast_to_tree_notation(&document_root)
        .map_err(|e| ProcessError::AssemblyError(format!("Treeviz rendering failed: {}", e)))?;

    // Format with document header
    let result = format!("⧉ Document: {}\n{}", source_path, treeviz_output);

    Ok(result)
}

// Phase 2 processing functions
fn process_ast_no_inline_json(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let _token_tree = token_tree_builder
        .build_tree(tokens.clone())
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Semantic analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let _semantic_tokens = semantic_analyzer
        .analyze(tokens.clone())
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: AST Construction (pending implementation)
    // TODO: Implement AST construction phase
    let ast_elements: Vec<crate::ast::ElementNode> = vec![]; // Placeholder

    // Serialize AST elements to JSON
    let result = serde_json::json!({
        "source": source_path,
        "ast_elements": ast_elements
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

fn process_ast_no_inline_treeviz(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let _token_tree = token_tree_builder
        .build_tree(tokens.clone())
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Semantic analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let _semantic_tokens = semantic_analyzer
        .analyze(tokens.clone())
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: AST Construction (pending implementation)
    // TODO: Implement AST construction phase
    let ast_elements: Vec<crate::ast::ElementNode> = vec![]; // Placeholder

    // Create treeviz representation
    let mut result = format!("📄 AST (No Inlines): {}\n", source_path);
    if ast_elements.is_empty() {
        result.push_str("└─ (no elements parsed)\n");
    } else {
        for (i, element) in ast_elements.iter().enumerate() {
            let is_last = i == ast_elements.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            result.push_str(&format!("{} {}\n", prefix, format_element_node(element)));
        }
    }

    Ok(result)
}

fn process_ast_json(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let _token_tree = token_tree_builder
        .build_tree(tokens.clone())
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Semantic analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let _semantic_tokens = semantic_analyzer
        .analyze(tokens.clone())
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: AST Construction (pending implementation)
    // TODO: Implement AST construction phase
    let ast_elements: Vec<crate::ast::ElementNode> = vec![]; // Placeholder

    // Phase 2b: Parse inlines (stubbed - returns unchanged)
    let inline_parser = InlineParser::new();
    let ast_elements_with_inlines = inline_parser
        .parse_inlines(ast_elements)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Serialize AST elements to JSON
    let result = serde_json::json!({
        "source": source_path,
        "ast_elements": ast_elements_with_inlines,
        "inline_processing": "stubbed (unchanged)"
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

fn process_ast_treeviz(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = ScannerTokenTreeBuilder::new();
    let _token_tree = token_tree_builder
        .build_tree(tokens.clone())
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Semantic analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let _semantic_tokens = semantic_analyzer
        .analyze(tokens.clone())
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: AST Construction (pending implementation)
    // TODO: Implement AST construction phase
    let ast_elements: Vec<crate::ast::ElementNode> = vec![]; // Placeholder

    // Phase 2b: Parse inlines (stubbed - returns unchanged)
    let inline_parser = InlineParser::new();
    let ast_elements_with_inlines = inline_parser
        .parse_inlines(ast_elements)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Create treeviz representation
    let mut result = format!("📄 AST (With Inlines - Stubbed): {}\n", source_path);
    if ast_elements_with_inlines.is_empty() {
        result.push_str("└─ (no elements parsed)\n");
    } else {
        for (i, element) in ast_elements_with_inlines.iter().enumerate() {
            let is_last = i == ast_elements_with_inlines.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            result.push_str(&format!("{} {}\n", prefix, format_element_node(element)));
        }
    }

    Ok(result)
}

/// Helper function to format ElementNode for treeviz output
fn format_element_node(element: &crate::ast::ElementNode) -> String {
    match element {
        crate::ast::ElementNode::ParagraphBlock(paragraph) => {
            format!(
                "📝 ParagraphBlock ({} content items)",
                paragraph.content.len()
            )
        }
        crate::ast::ElementNode::SessionBlock(session) => {
            format!(
                "📖 SessionBlock ({} content items)",
                session.content.content.len()
            )
        }
        // Add other element types as they're implemented
        _ => format!("❓ {:?}", element),
    }
}
