use std::fs;
use txxt::tokenizer::verbatim_scanner::VerbatimScanner;

fn main() {
    let content = fs::read_to_string("tests/verbatim_scanner/correct/simple_stretched.txxt").unwrap();
    let lines: Vec<&str> = content.lines().collect();
    
    // Extract TXXT content (everything after first line)
    let txxt_content = lines[1..].join("\n");
    
    println!("TXXT content:");
    for (i, line) in txxt_content.lines().enumerate() {
        println!("{:2}: '{}'", i+1, line);
    }
    
    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(&txxt_content);
    
    println!("\nFound {} blocks:", blocks.len());
    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
        println!("  Block: {}-{} (indent: {})", block.block_start, block.block_end, block.title_indent);
    }
}