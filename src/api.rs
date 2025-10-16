//! TXXT Processing API
//!
//! Pure functions for processing TXXT content through the three-phase pipeline.
//! This module contains no I/O operations, CLI handling, or process exits.
//! All functions take structured input and return structured output for easy testing.

use serde_json;
use std::error::Error;
use std::fmt;

use crate::assembler::Assembler;
use crate::lexer::elements::verbatim::verbatim_scanner::VerbatimScanner;
use crate::lexer::pipeline::TokenTreeBuilder;
use crate::lexer::tokenize;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    // Phase 1: Lexer outputs
    VerbatimMarks,
    TokenStream,
    TokenTree,

    // Phase 2: Parser outputs (WIP)
    AstNoInlineTreeviz,
    AstNoInlineJson,
    AstTreeviz,
    AstJson,

    // Phase 3: Assembly outputs
    AstFullJson,
    AstFullTreeviz,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "verbatim-marks" => Ok(OutputFormat::VerbatimMarks),
            "token-stream" => Ok(OutputFormat::TokenStream),
            "token-tree" => Ok(OutputFormat::TokenTree),
            "ast-no-inline-treeviz" => Ok(OutputFormat::AstNoInlineTreeviz),
            "ast-no-inline-json" => Ok(OutputFormat::AstNoInlineJson),
            "ast-treeviz" => Ok(OutputFormat::AstTreeviz),
            "ast-json" => Ok(OutputFormat::AstJson),
            "ast-full-json" => Ok(OutputFormat::AstFullJson),
            "ast-full-treeviz" => Ok(OutputFormat::AstFullTreeviz),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessArgs {
    pub content: String,
    pub source_path: String,
    pub format: OutputFormat,
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
    match args.format {
        // Phase 1: Lexer outputs
        OutputFormat::VerbatimMarks => process_verbatim_marks(&args.content, &args.source_path),
        OutputFormat::TokenStream => process_token_stream(&args.content, &args.source_path),
        OutputFormat::TokenTree => process_token_tree(&args.content, &args.source_path),

        // Phase 2: Parser outputs (WIP)
        OutputFormat::AstNoInlineTreeviz => Err(ProcessError::NotImplemented(
            "Phase 2a (AST without inlines) not yet implemented. This will output tree visualization of AST nodes without inline processing.".to_string()
        )),
        OutputFormat::AstNoInlineJson => Err(ProcessError::NotImplemented(
            "Phase 2a (AST without inlines) not yet implemented. This will output JSON of AST nodes without inline processing.".to_string()
        )),
        OutputFormat::AstTreeviz => Err(ProcessError::NotImplemented(
            "Phase 2b (AST with inlines) not yet implemented. This will output tree visualization of complete AST with inline processing.".to_string()
        )),
        OutputFormat::AstJson => Err(ProcessError::NotImplemented(
            "Phase 2b (AST with inlines) not yet implemented. This will output JSON of complete AST with inline processing.".to_string()
        )),

        // Phase 3: Assembly outputs
        OutputFormat::AstFullJson => process_ast_full_json(&args.content, &args.source_path),
        OutputFormat::AstFullTreeviz => process_ast_full_treeviz(&args.content, &args.source_path),
    }
}

// Phase 1: Lexer processing functions

fn process_verbatim_marks(content: &str, source_path: &str) -> Result<String, ProcessError> {
    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(content);

    // Convert blocks to serializable format
    let serializable_blocks: Vec<serde_json::Value> = blocks
        .iter()
        .map(|block| {
            serde_json::json!({
                "block_start": block.block_start,
                "block_end": block.block_end,
                "block_type": format!("{:?}", block.block_type),
                "title_indent": block.title_indent,
                "content_start": block.content_start,
                "content_end": block.content_end
            })
        })
        .collect();

    let result = serde_json::json!({
        "source": source_path,
        "verbatim_blocks": serializable_blocks
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

fn process_token_stream(content: &str, source_path: &str) -> Result<String, ProcessError> {
    let tokens = tokenize(content);

    let result = serde_json::json!({
        "source": source_path,
        "tokens": tokens
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

fn process_token_tree(content: &str, source_path: &str) -> Result<String, ProcessError> {
    let tokens = tokenize(content);
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Convert block tree to serializable format
    let serializable_tree = serialize_token_tree(&token_tree);

    let result = serde_json::json!({
        "source": source_path,
        "token_tree": serializable_tree
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

/// Helper function to serialize TokenTree to JSON
fn serialize_token_tree(tree: &crate::lexer::pipeline::TokenTree) -> serde_json::Value {
    serde_json::json!({
        "tokens": tree.tokens,
        "children": tree.children.iter().map(serialize_token_tree).collect::<Vec<_>>()
    })
}

// Phase 3: Assembly processing functions

fn process_ast_full_json(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 3a: Assemble document from block tree
    let assembler = Assembler::new();
    let document = assembler
        .assemble_document(token_tree, Some(source_path.to_string()))
        .map_err(|e| ProcessError::AssemblyError(e.to_string()))?;

    // Serialize to JSON
    let result = serde_json::json!({
        "source": source_path,
        "document": document
    });

    serde_json::to_string_pretty(&result)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

fn process_ast_full_treeviz(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 3a: Assemble document from block tree
    let assembler = Assembler::new();
    let document = assembler
        .assemble_document(token_tree, Some(source_path.to_string()))
        .map_err(|e| ProcessError::AssemblyError(e.to_string()))?;

    // For now, create a simple treeviz representation since Phase 2 parsing isn't implemented
    let result = format!(
        "⧉ Document: {}\n\
         ├─ Ψ SessionContainer (placeholder)\n\
         │   └─ 📋 Phase 2 parsing not yet implemented\n\
         │       └─ Raw tokens: {} total\n\
         └─ 📊 Assembly Info:\n\
             ├─ Parser: {}\n\
             ├─ Processed: {}\n\
             └─ Stats: {} tokens, {} blocks, depth {}\n",
        source_path,
        document.assembly_info.stats.token_count,
        document.assembly_info.parser_version,
        document
            .assembly_info
            .processed_at
            .unwrap_or("unknown".to_string()),
        document.assembly_info.stats.token_count,
        document.assembly_info.stats.block_count,
        document.assembly_info.stats.max_depth
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_verbatim_marks() {
        let args = ProcessArgs {
            content: "Some content\n    console.log('test');\n(javascript)".to_string(),
            source_path: "test.txxt".to_string(),
            format: OutputFormat::VerbatimMarks,
        };

        let result = process(args).unwrap();
        assert!(result.contains("verbatim_blocks"));
        assert!(result.contains("test.txxt"));
    }

    #[test]
    fn test_process_token_stream() {
        let args = ProcessArgs {
            content: "Hello world".to_string(),
            source_path: "test.txxt".to_string(),
            format: OutputFormat::TokenStream,
        };

        let result = process(args).unwrap();
        assert!(result.contains("tokens"));
        assert!(result.contains("test.txxt"));
    }

    #[test]
    fn test_process_token_tree() {
        let args = ProcessArgs {
            content: "Hello world".to_string(),
            source_path: "test.txxt".to_string(),
            format: OutputFormat::TokenTree,
        };

        let result = process(args).unwrap();
        assert!(result.contains("token_tree"));
        assert!(result.contains("test.txxt"));
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!(
            "verbatim-marks".parse::<OutputFormat>().unwrap(),
            OutputFormat::VerbatimMarks
        );
        assert_eq!(
            "token-stream".parse::<OutputFormat>().unwrap(),
            OutputFormat::TokenStream
        );
        assert!("invalid-format".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_unimplemented_formats() {
        let args = ProcessArgs {
            content: "test".to_string(),
            source_path: "test.txxt".to_string(),
            format: OutputFormat::AstJson,
        };

        let result = process(args);
        assert!(matches!(result, Err(ProcessError::NotImplemented(_))));
    }
}
