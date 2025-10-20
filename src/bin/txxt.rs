//! TXXT Command Line Interface
//!
//! A thin CLI wrapper around the TXXT processing API.

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use txxt::api::{format_output_unified, process_unified, Format, Stage};

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
    let args = Args::parse();

    // Handle --help-stages flag
    if args.help_stages {
        print_help_stages();
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

    // Parse and validate stage
    let stage = match Stage::from_name(&args.stage) {
        Some(s) => s,
        None => {
            eprintln!("Error: Invalid stage '{}'\n", args.stage);
            eprintln!("Available stages:");
            for s in Stage::all() {
                eprintln!("  - {}: {}", s.name(), s.description());
            }
            std::process::exit(1);
        }
    };

    // Parse and validate format
    let format = match Format::from_name(&args.format) {
        Some(f) => f,
        None => {
            eprintln!("Error: Invalid format '{}'\n", args.format);
            eprintln!("Available formats:");
            for f in Format::all() {
                eprintln!("  - {}: {}", f.name(), f.description());
            }
            std::process::exit(1);
        }
    };

    // Validate stage-format combination
    if !stage.supports_format(format) {
        eprintln!(
            "Error: Format '{}' is not supported for stage '{}'",
            format.name(),
            stage.name()
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

    // Process to the requested stage
    let output = process_unified(&content, stage, Some(path.clone()))?;

    // Format the output
    let formatted = format_output_unified(&output, format, Some(&path))?;

    // Output to stdout
    print!("{}", formatted);
    io::stdout().flush()?;

    Ok(())
}

fn print_help_stages() {
    println!("AVAILABLE STAGES:");
    for stage in Stage::all() {
        println!("  {:<18} {}", stage.name(), stage.description());
    }

    println!("\nAVAILABLE FORMATS:");
    for format in Format::all() {
        println!("  {:<18} {}", format.name(), format.description());
    }

    println!("\nSTAGE-FORMAT COMBINATIONS:");
    for stage in Stage::all() {
        let mut supported_formats = Vec::new();
        for format in Format::all() {
            if stage.supports_format(*format) {
                supported_formats.push(format.name());
            }
        }
        println!("  {:<18} {}", stage.name(), supported_formats.join(", "));
    }
}
