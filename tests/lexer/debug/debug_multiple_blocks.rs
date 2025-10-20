//! Debug multiple blocks

use txxt::lexer::verbatim_scanning::VerbatimScanner;

#[test]
fn debug_multiple_blocks() {
    let scanner = VerbatimScanner::new();
    let text = r#"First block:
    first content
()

Second block:
    second content
()"#;

    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{}: {}", i + 1, line);
    }

    let blocks = scanner.scan(text);
    println!("\nFound {} blocks:", blocks.len());

    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
    }
}
