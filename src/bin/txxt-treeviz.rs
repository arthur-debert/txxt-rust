#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]

//! TXXT Tree Visualization CLI Tool
//!
//! A command line tool that takes a TXXT file, parses it, and outputs
//! the tree notation representation of its AST.
//!
//! Usage:
//!   txxt-treeviz `<input-file>` [options]
//!
//! Options:
//!   --format `<format>`     Output format: treeviz (default), json
//!   --config `<file>`       Path to configuration file for custom mappings
//!   --ascii              Use ASCII characters instead of Unicode
//!   --debug              Include debug information in output
//!   --metadata           Include metadata in tree nodes

use clap::{Arg, Command};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use txxt::tools::treeviz::{
    converter::create_demo_notation_data,
    icons::{IconConfig, DEFAULT_ICON_CONFIG},
    renderer::{notation_data_to_json, render_with_options, RenderOptions, TreeChars},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("txxt-treeviz")
        .version("1.0.0")
        .author("TXXT Project")
        .about("TXXT AST Tree Visualization Tool")
        .arg(
            Arg::new("input")
                .help("Input TXXT file")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .help("Output format: treeviz, json")
                .default_value("treeviz"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("Path to configuration file for custom mappings"),
        )
        .arg(
            Arg::new("ascii")
                .long("ascii")
                .help("Use ASCII characters instead of Unicode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Include debug information in output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("metadata")
                .long("metadata")
                .help("Include metadata in tree nodes")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("demo")
                .long("demo")
                .help("Show demo output (for testing)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Handle demo mode
    if matches.get_flag("demo") {
        return run_demo(&matches);
    }

    // Get input file
    let input_file = matches.get_one::<String>("input");
    if input_file.is_none() {
        eprintln!("Error: Input file required (or use --demo for testing)");
        eprintln!("Usage: txxt-treeviz <input-file> [options]");
        eprintln!("       txxt-treeviz --demo");
        std::process::exit(1);
    }

    let input_path = input_file.unwrap();

    // Check if input file exists
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        std::process::exit(1);
    }

    // Load configuration
    let _config = if let Some(config_path) = matches.get_one::<String>("config") {
        load_config(config_path)?
    } else {
        DEFAULT_ICON_CONFIG.clone()
    };

    // Read input file
    let _content = fs::read_to_string(input_path)?;

    // Parse the file (placeholder - would use actual parser when ready)
    eprintln!("Note: Parser not yet implemented, using demo data for visualization");

    // For now, use demo data since parser isn't ready
    let notation_data = create_demo_notation_data();

    // Get output format
    let format = matches.get_one::<String>("format").unwrap();

    // Generate output based on format
    let output = match format.as_str() {
        "json" => notation_data_to_json(&notation_data)?,
        "treeviz" => {
            // Create render options
            let options = RenderOptions {
                include_debug: matches.get_flag("debug"),
                include_metadata: matches.get_flag("metadata"),
                tree_chars: if matches.get_flag("ascii") {
                    TreeChars::ascii()
                } else {
                    TreeChars::default()
                },
                ..Default::default()
            };

            render_with_options(&notation_data, &options)?
        }
        _ => {
            eprintln!("Error: Unknown format '{}'", format);
            std::process::exit(1);
        }
    };

    // Output result
    print!("{}", output);
    io::stdout().flush()?;

    Ok(())
}

fn run_demo(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    println!("TXXT Tree Visualization Demo");
    println!("============================");
    println!();

    // Create demo data
    let demo_data = create_demo_notation_data();

    // Get format
    let format = matches.get_one::<String>("format").unwrap();

    match format.as_str() {
        "json" => {
            println!("JSON Output:");
            println!("{}", notation_data_to_json(&demo_data)?);
        }
        "treeviz" => {
            println!("Tree Visualization:");

            // Create render options
            let options = RenderOptions {
                include_debug: matches.get_flag("debug"),
                include_metadata: matches.get_flag("metadata"),
                tree_chars: if matches.get_flag("ascii") {
                    TreeChars::ascii()
                } else {
                    TreeChars::default()
                },
                ..Default::default()
            };

            if matches.get_flag("ascii") {
                println!("(Using ASCII characters)");
            } else {
                println!("(Using Unicode characters)");
            }

            println!();
            println!("{}", render_with_options(&demo_data, &options)?);
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn load_config(config_path: &str) -> Result<IconConfig, Box<dyn std::error::Error>> {
    // Load configuration from JSON file
    let config_content = fs::read_to_string(config_path)?;
    let config: IconConfig = serde_json::from_str(&config_content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_demo_mode() {
        // Test that demo mode works
        let output = Command::new("cargo")
            .args(["run", "--bin", "txxt-treeviz", "--", "--demo"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("TXXT Tree Visualization Demo"));
        assert!(stdout.contains("Sample Document"));
    }

    #[test]
    fn test_ascii_output() {
        let output = Command::new("cargo")
            .args(["run", "--bin", "txxt-treeviz", "--", "--demo", "--ascii"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("|-")); // ASCII tree characters
        assert!(stdout.contains("`-"));
    }

    #[test]
    fn test_json_output() {
        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "txxt-treeviz",
                "--",
                "--demo",
                "--format",
                "json",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("\"root\"")); // JSON structure
        assert!(stdout.contains("\"config\""));
    }
}
