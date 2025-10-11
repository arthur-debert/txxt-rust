use txxt::tokenizer::verbatim_scanner::VerbatimScanner;

#[test]
fn debug_parameters() {
    let scanner = VerbatimScanner::new();
    let text = r#"Code with metadata:
    def calculate_pi():
        return 3.14159
(python: version=3.11, author="Jane Doe")


Code with metadata, but no label:
    def calculate_pi():
        return 3.14159
(: version=3.11, author="Jane Doe")

Code with metadata, but no label:
    def calculate_pi():
        return 3.14159
(: version=3.11, author="Jane Doe")
 No newline at end of file"#;

    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{:2}: '{}'", i + 1, line);
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
        println!("  Terminator: {}", block.block_end);
        println!("  Type: {:?}", block.block_type);
    }
}
