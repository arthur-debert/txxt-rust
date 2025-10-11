use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: txxt <command>");
        process::exit(1);
    }

    match args[1].as_str() {
        "help" => println!("txxt - A parser and processor for the txxt markup language"),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            process::exit(1);
        }
    }
}
