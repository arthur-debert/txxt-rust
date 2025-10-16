//! TXXT Command Line Interface
//!
//! A clean, organized CLI tool for processing TXXT files through the three-phase pipeline.
//! Supports multiple output formats corresponding to different pipeline stages.
//!
//! Usage: txxt <path> --format <format>
//!
//! Formats:
//!   Phase 1 (Lexer):
//!     verbatim-marks    - JSON output of verbatim block detection
//!     token-stream      - JSON output of positioned tokens
//!     token-tree        - JSON output of hierarchical token structure
//!
//!   Phase 2 (Parser) - WIP:
//!     ast-no-inline-treeviz - Tree visualization of AST without inlines
//!     ast-no-inline-json    - JSON output of AST without inlines  
//!     ast-treeviz           - Tree visualization of AST with inlines
//!     ast-json              - JSON output of AST with inlines
//!
//!   Phase 3 (Assembly) - Future:
//!     ast-full-json     - Complete document with metadata
//!     ast-full-treeviz  - Complete document visualization

use clap::{Arg, Command};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use txxt::assembler::Assembler;
use txxt::tokenizer::elements::verbatim::verbatim_scanner::VerbatimScanner;
use txxt::tokenizer::pipeline::BlockGrouper;
use txxt::tokenizer::tokenize;

#[derive(Debug, Clone, PartialEq)]
enum OutputFormat {
    // Phase 1: Lexer outputs
    VerbatimMarks,
    TokenStream,
    TokenTree,

    // Phase 2: Parser outputs (WIP)
    AstNoInlineTreeviz,
    AstNoInlineJson,
    AstTreeviz,
    AstJson,

    // Phase 3: Assembly outputs (Now Available)
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("txxt")
        .version("1.0.0")
        .author("TXXT Project")
        .about("TXXT Processing Pipeline CLI")
        .arg(
            Arg::new("path")
                .help("Path to TXXT file to process")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .value_name("FORMAT")
                .help("Output format (see --help for options)")
                .required(true),
        )
        .get_matches();

    let input_path = matches.get_one::<String>("path").unwrap();
    let format_str = matches.get_one::<String>("format").unwrap();

    // Validate input file
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }

    // Parse format
    let format = match format_str.parse::<OutputFormat>() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            print_format_help();
            std::process::exit(1);
        }
    };

    // Read input file
    let content = fs::read_to_string(input_path)?;

    // Process based on format
    let output = match format {
        // Phase 1: Lexer outputs
        OutputFormat::VerbatimMarks => process_verbatim_marks(&content, input_path)?,
        OutputFormat::TokenStream => process_token_stream(&content, input_path)?,
        OutputFormat::TokenTree => process_token_tree(&content, input_path)?,

        // Phase 2: Parser outputs (WIP)
        OutputFormat::AstNoInlineTreeviz => process_ast_no_inline_treeviz(&content, input_path)?,
        OutputFormat::AstNoInlineJson => process_ast_no_inline_json(&content, input_path)?,
        OutputFormat::AstTreeviz => process_ast_treeviz(&content, input_path)?,
        OutputFormat::AstJson => process_ast_json(&content, input_path)?,

        // Phase 3: Assembly outputs (Now Available)
        OutputFormat::AstFullJson => process_ast_full_json(&content, input_path)?,
        OutputFormat::AstFullTreeviz => process_ast_full_treeviz(&content, input_path)?,
    };

    // Output to stdout
    print!("{}", output);
    io::stdout().flush()?;

    Ok(())
}

// Phase 1: Lexer processing functions

fn process_verbatim_marks(
    content: &str,
    source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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

    Ok(serde_json::to_string_pretty(&result)?)
}

fn process_token_stream(
    content: &str,
    source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let tokens = tokenize(content);

    let result = serde_json::json!({
        "source": source_path,
        "tokens": tokens
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

fn process_token_tree(
    content: &str,
    source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let tokens = tokenize(content);
    let block_grouper = BlockGrouper::new();
    let block_tree = block_grouper.group_blocks(tokens)?;

    // Convert block tree to serializable format
    let serializable_tree = serialize_block_group(&block_tree);

    let result = serde_json::json!({
        "source": source_path,
        "token_tree": serializable_tree
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

/// Helper function to serialize BlockGroup to JSON
fn serialize_block_group(block: &txxt::tokenizer::pipeline::BlockGroup) -> serde_json::Value {
    serde_json::json!({
        "tokens": block.tokens,
        "children": block.children.iter().map(serialize_block_group).collect::<Vec<_>>()
    })
}

// Phase 2: Parser processing functions (WIP - return placeholder errors)

fn process_ast_no_inline_treeviz(
    _content: &str,
    _source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    Err("Phase 2a (AST without inlines) not yet implemented. This will output tree visualization of AST nodes without inline processing.".into())
}

fn process_ast_no_inline_json(
    _content: &str,
    _source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    Err("Phase 2a (AST without inlines) not yet implemented. This will output JSON of AST nodes without inline processing.".into())
}

fn process_ast_treeviz(
    _content: &str,
    _source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    Err("Phase 2b (AST with inlines) not yet implemented. This will output tree visualization of complete AST with inline processing.".into())
}

fn process_ast_json(
    _content: &str,
    _source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    Err("Phase 2b (AST with inlines) not yet implemented. This will output JSON of complete AST with inline processing.".into())
}

// Phase 3: Assembly processing functions (Future - return placeholder errors)

fn process_ast_full_json(
    content: &str,
    source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let block_grouper = BlockGrouper::new();
    let block_tree = block_grouper.group_blocks(tokens)?;

    // Phase 3a: Assemble document from block tree
    let assembler = Assembler::new();
    let document = assembler.assemble_document(block_tree, Some(source_path.to_string()))?;

    // Serialize to JSON
    let result = serde_json::json!({
        "source": source_path,
        "document": document
    });

    Ok(serde_json::to_string_pretty(&result)?)
}

fn process_ast_full_treeviz(
    content: &str,
    source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Phase 1: Tokenize and group blocks
    let tokens = tokenize(content);
    let block_grouper = BlockGrouper::new();
    let block_tree = block_grouper.group_blocks(tokens)?;

    // Phase 3a: Assemble document from block tree
    let assembler = Assembler::new();
    let document = assembler.assemble_document(block_tree, Some(source_path.to_string()))?;

    // For now, create a simple treeviz representation since Phase 2 parsing isn't implemented
    // TODO: Use proper treeviz when Phase 2 is complete and we have ElementNode
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

fn print_format_help() {
    eprintln!("\nAvailable formats:");
    eprintln!("  Phase 1 (Lexer) - Ready:");
    eprintln!("    verbatim-marks    - JSON output of verbatim block detection");
    eprintln!("    token-stream      - JSON output of positioned tokens");
    eprintln!("    token-tree        - JSON output of hierarchical token structure");
    eprintln!();
    eprintln!("  Phase 2 (Parser) - WIP:");
    eprintln!("    ast-no-inline-treeviz - Tree visualization of AST without inlines");
    eprintln!("    ast-no-inline-json    - JSON output of AST without inlines");
    eprintln!("    ast-treeviz           - Tree visualization of AST with inlines");
    eprintln!("    ast-json              - JSON output of AST with inlines");
    eprintln!();
    eprintln!("  Phase 3 (Assembly) - Available:");
    eprintln!("    ast-full-json     - Complete document with metadata");
    eprintln!("    ast-full-treeviz  - Complete document visualization");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::NamedTempFile;

    #[test]
    fn test_verbatim_marks_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "Some content\n    console.log('test');\n(javascript)"
        )
        .unwrap();

        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "txxt",
                "--",
                temp_file.path().to_str().unwrap(),
                "--format",
                "verbatim-marks",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("verbatim_blocks"));
    }

    #[test]
    fn test_token_stream_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello world").unwrap();

        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "txxt",
                "--",
                temp_file.path().to_str().unwrap(),
                "--format",
                "token-stream",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("tokens"));
    }

    #[test]
    fn test_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();

        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "txxt",
                "--",
                temp_file.path().to_str().unwrap(),
                "--format",
                "invalid-format",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Unknown format"));
    }
}
