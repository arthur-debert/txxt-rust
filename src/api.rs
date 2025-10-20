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

use crate::lexer::tokenize;
use crate::lexer::SemanticAnalyzer;

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

use crate::assembler::{AnnotationAttacher, DocumentAssembler};
use crate::ast::{Document, ElementNode};
use crate::cst::{HighLevelTokenList, ScannerToken};
use crate::parser::{AstConstructor, InlineParser};

/// Processing stages in the TXXT pipeline (new unified API).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// 1.b: Scanner tokens - low-level flat token stream
    ScannerTokens,
    /// 1.c: High-level tokens - semantic token grouping
    HighLevelTokens,
    /// 2.a: AST blocks - block-level structure (no inline parsing)
    AstBlock,
    /// 2.b: AST inlines - complete AST with inline elements
    AstInlines,
    /// 3.a: Document - wrapped in document structure (annotations in content)
    AstDocument,
    /// 3.b: Full document - complete with annotations attached
    AstFull,
}

impl Stage {
    pub fn name(&self) -> &'static str {
        match self {
            Stage::ScannerTokens => "scanner-tokens",
            Stage::HighLevelTokens => "high-level-tokens",
            Stage::AstBlock => "ast-block",
            Stage::AstInlines => "ast-inlines",
            Stage::AstDocument => "ast-document",
            Stage::AstFull => "ast-full",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Stage::ScannerTokens => "Raw scanner tokens",
            Stage::HighLevelTokens => "High-level analyzed tokens",
            Stage::AstBlock => "Block-level Abstract Syntax Tree",
            Stage::AstInlines => "AST with parsed inlines",
            Stage::AstDocument => "Document-level AST",
            Stage::AstFull => "Full AST with annotations",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "scanner-tokens" => Some(Stage::ScannerTokens),
            "high-level-tokens" => Some(Stage::HighLevelTokens),
            "ast-block" => Some(Stage::AstBlock),
            "ast-inlines" => Some(Stage::AstInlines),
            "ast-document" => Some(Stage::AstDocument),
            "ast-full" => Some(Stage::AstFull),
            _ => None,
        }
    }

    pub fn all() -> &'static [Stage] {
        &[
            Stage::ScannerTokens,
            Stage::HighLevelTokens,
            Stage::AstBlock,
            Stage::AstInlines,
            Stage::AstDocument,
            Stage::AstFull,
        ]
    }

    pub fn supports_format(&self, format: Format) -> bool {
        match (self, format) {
            (_, Format::Json) => true,
            (Stage::ScannerTokens, Format::TreeViz) => false,
            (Stage::HighLevelTokens, Format::TreeViz) => false,
            _ => true,
        }
    }
}

/// Output formats for displaying processed data (new unified API).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    TreeViz,
}

impl Format {
    pub fn name(&self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::TreeViz => "treeviz",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Format::Json => "JSON output",
            Format::TreeViz => "Tree visualization",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "json" => Some(Format::Json),
            "treeviz" => Some(Format::TreeViz),
            _ => None,
        }
    }

    pub fn all() -> &'static [Format] {
        &[Format::Json, Format::TreeViz]
    }
}

/// Output from processing at a specific stage (new unified API).
#[derive(Debug, Clone)]
pub enum Output {
    ScannerTokens(Vec<ScannerToken>),
    HighLevelTokens(HighLevelTokenList),
    AstBlock(Vec<ElementNode>),
    AstInlines(Vec<ElementNode>),
    AstDocument(Document),
    AstFull(Document),
}

impl Output {
    pub fn stage(&self) -> Stage {
        match self {
            Output::ScannerTokens(_) => Stage::ScannerTokens,
            Output::HighLevelTokens(_) => Stage::HighLevelTokens,
            Output::AstBlock(_) => Stage::AstBlock,
            Output::AstInlines(_) => Stage::AstInlines,
            Output::AstDocument(_) => Stage::AstDocument,
            Output::AstFull(_) => Stage::AstFull,
        }
    }
}

/// Unified processing function - processes to a specific stage.
pub fn process_unified(
    source: &str,
    stage: Stage,
    source_path: Option<String>,
) -> Result<Output, ProcessError> {
    // Step 1.b: Tokenization
    let scanner_tokens = tokenize(source);

    if stage == Stage::ScannerTokens {
        return Ok(Output::ScannerTokens(scanner_tokens));
    }

    // Step 1.c: Semantic Analysis
    let semantic_analyzer = SemanticAnalyzer::new();
    let high_level_tokens = semantic_analyzer
        .analyze(scanner_tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    if stage == Stage::HighLevelTokens {
        return Ok(Output::HighLevelTokens(high_level_tokens));
    }

    // Step 2.a: AST Construction (blocks only)
    let ast_blocks = AstConstructor::parse_to_element_nodes(&high_level_tokens)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    if stage == Stage::AstBlock {
        return Ok(Output::AstBlock(ast_blocks));
    }

    // Step 2.b: Inline Parsing
    let inline_parser = InlineParser::new();
    let ast_with_inlines = inline_parser
        .parse_inlines(ast_blocks)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    if stage == Stage::AstInlines {
        return Ok(Output::AstInlines(ast_with_inlines));
    }

    // Step 3.a: Document Assembly
    let document_assembler = DocumentAssembler::new();
    let document = document_assembler
        .assemble_document(ast_with_inlines, source_path)
        .map_err(|e| ProcessError::AssemblyError(e.to_string()))?;

    if stage == Stage::AstDocument {
        return Ok(Output::AstDocument(document));
    }

    // Step 3.b: Annotation Attachment
    let annotation_attacher = AnnotationAttacher::new();
    let full_document = annotation_attacher
        .attach_annotations(document)
        .map_err(|e| ProcessError::AssemblyError(e.to_string()))?;

    Ok(Output::AstFull(full_document))
}

/// Convenience function for full processing.
pub fn process_full_unified(
    source: &str,
    source_path: Option<String>,
) -> Result<Document, ProcessError> {
    match process_unified(source, Stage::AstFull, source_path)? {
        Output::AstFull(doc) => Ok(doc),
        _ => unreachable!(),
    }
}

/// Format processed output for display (new unified API).
pub fn format_output_unified(
    output: &Output,
    format: Format,
    source_path: Option<&str>,
) -> Result<String, ProcessError> {
    if !output.stage().supports_format(format) {
        return Err(ProcessError::NotImplemented(format!(
            "Format '{}' not supported for stage '{}'",
            format.name(),
            output.stage().name()
        )));
    }

    match format {
        Format::Json => format_as_json_unified(output, source_path),
        Format::TreeViz => format_as_treeviz_unified(output, source_path),
    }
}

fn format_as_json_unified(
    output: &Output,
    source_path: Option<&str>,
) -> Result<String, ProcessError> {
    let source = source_path.unwrap_or("(no source)");

    let json_value = match output {
        Output::ScannerTokens(tokens) => {
            serde_json::json!({
                "source": source,
                "stage": "scanner-tokens",
                "tokens": tokens
            })
        }
        Output::HighLevelTokens(tokens) => {
            serde_json::json!({
                "source": source,
                "stage": "high-level-tokens",
                "tokens": tokens.tokens
            })
        }
        Output::AstBlock(elements) => {
            serde_json::json!({
                "source": source,
                "stage": "ast-block",
                "elements": elements
            })
        }
        Output::AstInlines(elements) => {
            serde_json::json!({
                "source": source,
                "stage": "ast-inlines",
                "elements": elements
            })
        }
        Output::AstDocument(doc) => {
            serde_json::json!({
                "source": source,
                "stage": "ast-document",
                "document": {
                    "content": doc.content,
                    "assembly_info": doc.assembly_info
                }
            })
        }
        Output::AstFull(doc) => {
            serde_json::json!({
                "source": source,
                "stage": "ast-full",
                "document": {
                    "content": doc.content,
                    "assembly_info": doc.assembly_info
                }
            })
        }
    };

    serde_json::to_string_pretty(&json_value)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

fn format_as_treeviz_unified(
    output: &Output,
    source_path: Option<&str>,
) -> Result<String, ProcessError> {
    let source = source_path.unwrap_or("(no source)");

    match output {
        Output::ScannerTokens(_) | Output::HighLevelTokens(_) => {
            Err(ProcessError::NotImplemented(format!(
                "TreeViz not supported for stage '{}'",
                output.stage().name()
            )))
        }
        Output::AstBlock(elements) | Output::AstInlines(elements) => {
            let stage_label = match output.stage() {
                Stage::AstBlock => "AST (Blocks Only)",
                Stage::AstInlines => "AST (With Inlines)",
                _ => "AST",
            };

            let mut result = format!("📄 {}: {}\n", stage_label, source);
            if elements.is_empty() {
                result.push_str("└─ (no elements)\n");
            } else {
                for (i, element) in elements.iter().enumerate() {
                    let is_last = i == elements.len() - 1;
                    let prefix = if is_last { "└─" } else { "├─" };
                    result.push_str(&format!("{} {}\n", prefix, format_element_node(element)));
                }
            }
            Ok(result)
        }
        Output::AstDocument(doc) | Output::AstFull(doc) => {
            let document_element = ElementNode::SessionContainer(doc.content.clone());
            let treeviz_output = crate::tools::treeviz::ast_to_tree_notation(&document_element)
                .map_err(|e| {
                    ProcessError::SerializationError(format!("Treeviz rendering failed: {}", e))
                })?;

            let stage_label = match output.stage() {
                Stage::AstDocument => "Document (No Annotations)",
                Stage::AstFull => "Document (Full)",
                _ => "Document",
            };

            Ok(format!("⧉ {}: {}\n{}", stage_label, source, treeviz_output))
        }
    }
}
