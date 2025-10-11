use txxt::tokenizer::verbatim_scanner::VerbatimScanner;

fn main() {
    let content = r#"title:
Content starts at column 0
Another line at column 0
Blank lines are allowed
()"#;

    println!("Input content:");
    for (i, line) in content.lines().enumerate() {
        println!("{:2}: '{}' (indent: {})", i+1, line, 
                line.chars().take_while(|c| c.is_whitespace()).count());
    }

    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(content);
    
    println!("\nFound {} blocks:", blocks.len());
    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
    }
}