//! Debug the verbatim scanner to understand what it's finding

use txxt::lexer::elements::verbatim::verbatim_scanner::VerbatimScanner;

#[test]
fn debug_simple_verbatim() {
    let scanner = VerbatimScanner::new();
    let text = r#"Simple verbatim with title and label:
    print("Hello World")
    return 42
(python)"#;

    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{}: {}", i + 1, line);
    }

    let blocks = scanner.scan(text);
    println!("\nFound {} blocks:", blocks.len());

    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
        println!(
            "  Block: {}-{} (indent: {})",
            block.block_start, block.block_end, block.title_indent
        );
        match (block.content_start, block.content_end) {
            (Some(start), Some(end)) => println!("  Content: {}-{}", start, end),
            _ => println!("  Content: (empty)"),
        }
        println!("  Type: {:?}", block.block_type);
    }
}
