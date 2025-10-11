use std::env;
use std::process;

use txxt::{commands, tree};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let result = match args[1].as_str() {
        "validate" => {
            let path = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            commands::validate(path)
        }
        "clean" => {
            let info_file = args.get(2).map(|s| s.as_str()).unwrap_or(".info");
            commands::clean(info_file)
        }
        "add" => {
            if args.len() < 4 {
                eprintln!("Usage: info add <path> <annotation>");
                process::exit(1);
            }
            let annotation = args[3..].join(" ");
            commands::add(&args[2], annotation)
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: info remove <path>");
                process::exit(1);
            }
            commands::remove(&args[2])
        }
        "edit" => {
            if args.len() < 4 {
                eprintln!("Usage: info edit <path> <new-annotation>");
                process::exit(1);
            }
            let annotation = args[3..].join(" ");
            commands::edit(&args[2], annotation)
        }
        "distribute" => {
            let source_dir = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            commands::distribute(source_dir)
        }
        "gather" => {
            let target_dir = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            commands::gather(target_dir)
        }
        "tree" => {
            let mut paths = Vec::new();
            let mut no_color = false;
            let mut json = false;
            let mut i = 2;

            while i < args.len() {
                match args[i].as_str() {
                    "--no-color" => no_color = true,
                    "--json" => json = true,
                    path => paths.push(path.to_string()),
                }
                i += 1;
            }

            tree::tree_command(&paths, no_color, json)
        }
        "help" | "--help" | "-h" => {
            print_usage();
            return;
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn print_usage() {
    println!("txxt - Tool for managing .info annotation files");
    println!();
    println!("USAGE:");
    println!("    info <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    validate [PATH]           Validate .info files in directory (default: current)");
    println!(
        "    clean [INFO_FILE]         Remove invalid entries from .info file (default: .info)"
    );
    println!("    add <PATH> <ANNOTATION>   Add annotation for path");
    println!("    remove <PATH>             Remove annotation for path");
    println!("    edit <PATH> <ANNOTATION>  Update annotation for path");
    println!(
        "    distribute [DIR]          Move annotations to optimal .info files (default: current)"
    );
    println!("    gather [DIR]              Collect all annotations into directory's .info file (default: current)");
    println!("    tree [PATHS...] [OPTIONS] Display directory tree with annotations");
    println!("                              Options: --no-color, --json");
    println!("    help                      Show this help message");
}
