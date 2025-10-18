//! TXXT Command Line Interface
//!
//! A thin CLI wrapper around the TXXT processing API.
//! This binary handles argument parsing and I/O, delegating all logic to the API module.

use clap::{Arg, Command};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use txxt::api::{process, OutputFormat, ProcessArgs};

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

    // Call the pure API function
    let args = ProcessArgs {
        content,
        source_path: input_path.to_string(),
        format,
    };

    let output = process(args).map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

    // Output to stdout
    print!("{}", output);
    io::stdout().flush()?;

    Ok(())
}

fn print_format_help() {
    eprintln!("\nAvailable formats:");
    eprintln!("  Phase 1 (Lexer) - Ready:");
    eprintln!("    verbatim-marks    - JSON output of verbatim block detection");
    eprintln!("    token-stream      - JSON output of positioned tokens");
    eprintln!("    token-tree        - JSON output of hierarchical token structure");
    eprintln!();
    eprintln!("  Phase 2a (Semantic Analysis) - Ready:");
    eprintln!("    semantic-tokens   - JSON output of semantic tokens");
    eprintln!();
    eprintln!("  Phase 2 (Parser) - Available:");
    eprintln!("    ast-no-inline-treeviz - Tree visualization of AST without inlines");
    eprintln!("    ast-no-inline-json    - JSON output of AST without inlines");
    eprintln!("    ast-treeviz           - Tree visualization of AST with inlines (stubbed)");
    eprintln!("    ast-json              - JSON output of AST with inlines (stubbed)");
    eprintln!();
    eprintln!("  Phase 3 (Assembly) - Available:");
    eprintln!("    ast-full-json     - Complete document with metadata");
    eprintln!("    ast-full-treeviz  - Complete document visualization");
}
