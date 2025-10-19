//! # TXXT Command Line Interface
//!
//! A thin CLI wrapper around the TXXT processing API that provides a clean interface
//! for processing TXXT documents through various stages of the parsing pipeline.
//!
//! ## Design Philosophy
//!
//! This binary follows a strict separation of concerns:
//!
//! - **CLI Layer**: Handles argument parsing, validation, and I/O operations
//! - **API Layer**: Contains pure processing functions with no side effects
//! - **Registry System**: Provides dynamic discovery of available stages and formats
//!
//! This architecture ensures that the core processing logic remains testable and
//! reusable while providing a user-friendly command-line interface.
//!
//! ## Architecture Overview
//!
//! The CLI operates in three phases:
//!
//! 1. **Initialization**: Sets up the global registries for stages, formats, and conversions
//! 2. **Validation**: Validates user inputs against available options in registries
//! 3. **Processing**: Delegates to the pure API functions in `txxt::api::process`
//!
//! ## Pipeline Stages
//!
//! The TXXT processing pipeline consists of multiple stages, each producing different
//! intermediate representations:
//!
//! - **scanner-tokens**: Raw lexical tokens from the scanner
//! - **semantic-tokens**: Tokens with semantic analysis applied
//! - **ast-block**: Block-level Abstract Syntax Tree (no inline processing)
//! - **ast-inlines**: AST with inline elements parsed
//! - **ast-document**: Document-level AST with assembly metadata
//! - **ast-full**: Complete AST with all annotations and metadata
//!
//! ## Output Formats
//!
//! Each stage can be output in different formats:
//!
//! - **json**: Machine-readable JSON representation
//! - **treeviz**: Human-readable tree visualization
//!
//! ## Registry-Driven Help System
//!
//! The `--help-stages` flag dynamically queries the registries to show:
//! - Available processing stages with descriptions
//! - Available output formats with descriptions  
//! - Valid stage-format combinations
//!
//! This ensures the help text always reflects the actual implementation capabilities
//! without requiring manual synchronization.
//!
//! ## Error Handling
//!
//! The CLI provides specific error messages for different failure modes:
//! - Invalid stage or format selection
//! - Unsupported stage-format combinations
//! - Missing or inaccessible input files
//! - Processing errors (tokenization, parsing, serialization)
//!
//! ## Usage Examples
//!
//! ```bash
//! # Show available options
//! txxt --help-stages
//!
//! # Process a file with default settings (ast-full + json)
//! txxt document.txxt
//!
//! # Get raw scanner tokens in JSON
//! txxt --stage scanner-tokens --format json document.txxt
//!
//! # Visualize the AST structure
//! txxt --stage ast-full --format treeviz document.txxt
//! ```

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use txxt::api::{process, ProcessArgs};
use txxt::processing_stages::{
    initialize_registries, CONVERSION_FACTORY, FORMAT_REGISTRY, STAGE_REGISTRY,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to TXXT file to process
    path: Option<String>,

    /// Output stage
    #[arg(long, short, default_value = "ast-full")]
    stage: String,

    /// Output format
    #[arg(long, short, default_value = "json")]
    format: String,

    /// Show available stages and formats
    #[arg(long, help = "Show available stages and formats")]
    help_stages: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_registries();
    let args = Args::parse();

    let stage_registry = STAGE_REGISTRY.lock().unwrap();
    let format_registry = FORMAT_REGISTRY.lock().unwrap();
    let conversion_factory = CONVERSION_FACTORY.lock().unwrap();

    // Handle --help-stages flag
    if args.help_stages {
        print_help_stages(&stage_registry, &format_registry, &conversion_factory);
        return Ok(());
    }

    // Require path for processing
    let path = match args.path {
        Some(p) => p,
        None => {
            eprintln!("Error: Path to TXXT file is required for processing");
            eprintln!("Use --help-stages to see available stages and formats");
            std::process::exit(1);
        }
    };

    // Validate stage
    if stage_registry.get(&args.stage).is_none() {
        eprintln!("Error: Invalid stage '{}'\n", args.stage);
        eprintln!("Available stages:");
        for stage in stage_registry.list() {
            eprintln!("  - {}: {}", stage.name, stage.description);
        }
        std::process::exit(1);
    }

    // Validate format
    if format_registry.get(&args.format).is_none() {
        eprintln!("Error: Invalid format '{}'\n", args.format);
        eprintln!("Available formats:");
        for format in format_registry.list() {
            eprintln!("  - {}: {}", format.name, format.description);
        }
        std::process::exit(1);
    }

    // Validate combination
    if !conversion_factory.is_supported(&args.stage, &args.format, &stage_registry) {
        eprintln!(
            "Error: Format '{}' is not supported for stage '{}'",
            args.format, args.stage
        );
        std::process::exit(1);
    }

    // Validate input file
    if !Path::new(&path).exists() {
        eprintln!("Error: Input file '{}' does not exist", &path);
        std::process::exit(1);
    }

    // Read input file
    let content = fs::read_to_string(&path)?;

    // Call the pure API function
    let process_args = ProcessArgs {
        content,
        source_path: path,
        stage: args.stage,
        format: args.format,
    };

    let output = process(process_args)?;

    // Output to stdout
    print!("{}", output);
    io::stdout().flush()?;

    Ok(())
}

fn print_help_stages(
    stage_registry: &txxt::processing_stages::StageRegistry,
    format_registry: &txxt::processing_stages::FormatRegistry,
    conversion_factory: &txxt::processing_stages::ConversionFactory,
) {
    println!("AVAILABLE STAGES:");
    let mut stages = stage_registry.list();
    stages.sort_by_key(|s| s.name);
    for stage in stages {
        println!("  {:<18} {}", stage.name, stage.description);
    }

    println!("\nAVAILABLE FORMATS:");
    let mut formats = format_registry.list();
    formats.sort_by_key(|f| f.name);
    for format in formats {
        println!("  {:<18} {}", format.name, format.description);
    }

    println!("\nSTAGE-FORMAT COMBINATIONS:");
    let mut stages = stage_registry.list();
    stages.sort_by_key(|s| s.name);
    for stage in stages {
        let mut supported_formats = Vec::new();
        for format in format_registry.list() {
            if conversion_factory.is_supported(stage.name, format.name, stage_registry) {
                supported_formats.push(format.name);
            }
        }
        supported_formats.sort();
        println!("  {:<18} {}", stage.name, supported_formats.join(", "));
    }
}
