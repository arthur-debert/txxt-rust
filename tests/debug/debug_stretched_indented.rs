use txxt::tokenizer::verbatim_scanner::VerbatimScanner;

#[test]
fn debug_stretched_indented() {
    let scanner = VerbatimScanner::new();
    let text = r#"title:
Content starts at column 0
Another line at column 0
Blank lines are allowed
 No newline at end of file
()


            title:
Content starts at column 0
Another line at column 0
Blank lines are allowed
            ()
 No newline at end of file"#;

    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{:2}: '{}'", i + 1, line);
        if !line.is_empty() {
            println!(
                "     indent: {}",
                line.chars().take_while(|c| c.is_whitespace()).count()
            );
        }
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
