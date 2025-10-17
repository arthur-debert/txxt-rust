//! TXXT Processing API
//!
//! Pure functions for processing TXXT content through the three-phase pipeline.
//! This module contains no I/O operations, CLI handling, or process exits.
//! All functions take structured input and return structured output for easy testing.

use serde_json;
use std::error::Error;
use std::fmt;

use crate::lexer::elements::verbatim::verbatim_scanner::VerbatimScanner;
use crate::lexer::pipeline::TokenTreeBuilder;
use crate::lexer::tokenize;
use crate::parser::pipeline::{BlockParser, InlineParser};

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

        // Phase 2: Parser outputs
        OutputFormat::AstNoInlineTreeviz => {
            process_ast_no_inline_treeviz(&args.content, &args.source_path)
        }
        OutputFormat::AstNoInlineJson => {
            process_ast_no_inline_json(&args.content, &args.source_path)
        }
        OutputFormat::AstTreeviz => process_ast_treeviz(&args.content, &args.source_path),
        OutputFormat::AstJson => process_ast_json(&args.content, &args.source_path),

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

    // Phase 2: Parse blocks (our session parsing implementation)
    let block_parser = BlockParser::new();
    let ast_elements = block_parser
        .parse_blocks(token_tree)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: Parse inlines (stubbed - returns unchanged)
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
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2: Parse blocks (our session parsing implementation)
    let block_parser = BlockParser::new();
    let ast_elements = block_parser
        .parse_blocks(token_tree)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

    // Phase 2b: Parse inlines (stubbed - returns unchanged)
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
                    tokens: crate::ast::tokens::TokenSequence::new(),
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
            tokens: crate::ast::tokens::TokenSequence::new(),
        },
    );

    // Use the proper treeviz system to render the AST
    let treeviz_output = crate::tools::treeviz::ast_to_tree_notation(&document_root)
        .map_err(|e| ProcessError::AssemblyError(format!("Treeviz rendering failed: {}", e)))?;

    // Format with document header
    let result = format!("⧉ Document: {}\n{}", source_path, treeviz_output);

    Ok(result)
}

// Phase 2: Parser processing functions

fn process_ast_no_inline_json(content: &str, source_path: &str) -> Result<String, ProcessError> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Parse blocks (no inline processing)
    let block_parser = BlockParser::new();
    let ast_elements = block_parser
        .parse_blocks(token_tree)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

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
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Parse blocks (no inline processing)
    let block_parser = BlockParser::new();
    let ast_elements = block_parser
        .parse_blocks(token_tree)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

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
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Parse blocks
    let block_parser = BlockParser::new();
    let ast_elements = block_parser
        .parse_blocks(token_tree)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

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
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .map_err(|e| ProcessError::TokenizationError(e.to_string()))?;

    // Phase 2a: Parse blocks
    let block_parser = BlockParser::new();
    let ast_elements = block_parser
        .parse_blocks(token_tree)
        .map_err(|e| ProcessError::ParseError(e.to_string()))?;

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
    fn test_phase2_formats_implemented() {
        // Test that all Phase 2 formats now work instead of returning NotImplemented
        let content = "Hello world";
        let source_path = "test.txxt";

        let formats = vec![
            OutputFormat::AstNoInlineJson,
            OutputFormat::AstNoInlineTreeviz,
            OutputFormat::AstJson,
            OutputFormat::AstTreeviz,
        ];

        for format in formats {
            let args = ProcessArgs {
                content: content.to_string(),
                source_path: source_path.to_string(),
                format: format.clone(),
            };

            let result = process(args);
            assert!(result.is_ok(), "Format {:?} should be implemented", format);
        }
    }
}
