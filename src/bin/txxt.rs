//! TXXT Command Line Interface
//!
//! A thin CLI wrapper around the TXXT processing API.
//! This binary handles argument parsing and I/O, delegating all logic to the API module.

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use txxt::processing_stages::{initialize_registries, STAGE_REGISTRY, FORMAT_REGISTRY, CONVERSION_FACTORY};
use txxt::api::{process, ProcessArgs};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to TXXT file to process
    path: String,

    /// Output stage
    #[arg(long, short, default_value = "ast-full")]
    stage: String,

    /// Output format
    #[arg(long, short, default_value = "json")]
    format: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_registries();
    let args = Args::parse();

    let stage_registry = STAGE_REGISTRY.lock().unwrap();
    let format_registry = FORMAT_REGISTRY.lock().unwrap();
    let conversion_factory = CONVERSION_FACTORY.lock().unwrap();

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
        eprintln!("Error: Format '{}' is not supported for stage '{}'", args.format, args.stage);
        std::process::exit(1);
    }

    // Validate input file
    if !Path::new(&args.path).exists() {
        eprintln!("Error: Input file '{}' does not exist", &args.path);
        std::process::exit(1);
    }

    // Read input file
    let content = fs::read_to_string(&args.path)?;

    // Call the pure API function
    let process_args = ProcessArgs {
        content,
        source_path: args.path,
        stage: args.stage,
        format: args.format,
    };

    let output = process(process_args)?;

    // Output to stdout
    print!("{}", output);
    io::stdout().flush()?;

    Ok(())
}
