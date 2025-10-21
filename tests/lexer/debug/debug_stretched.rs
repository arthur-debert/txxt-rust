use txxt::syntax::verbatim_scanning::VerbatimScanner;

#[test]
fn debug_stretched() {
    let scanner = VerbatimScanner::new();
    let text = r#"title:
Content starts at column 0
Another line at column 0
Blank lines are allowed
()"#;

    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{}: '{}'", i + 1, line);
    }

    let blocks = scanner.scan(text);
    println!("\nFound {} blocks:", blocks.len());

    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
    }
}
