use txxt::lexer::verbatim_scanning::VerbatimScanner;

#[test]
fn debug_false_start_then_real() {
    let scanner = VerbatimScanner::new();
    let text = r#"false start title:
    some content
    but no terminator

real title:
    actual content
(python)"#;

    println!("=== False start then real title ===");
    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{:2}: '{}'", i + 1, line);
    }

    let blocks = scanner.scan(text);
    println!("\nFound {} blocks:", blocks.len());

    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
    }
}

#[test]
fn debug_false_start_then_paragraph() {
    let scanner = VerbatimScanner::new();
    let text = r#"false start title:
    some content
    but no terminator

This is just a regular paragraph.
More text here."#;

    println!("\n=== False start then paragraph ===");
    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{:2}: '{}'", i + 1, line);
    }

    let blocks = scanner.scan(text);
    println!("\nFound {} blocks:", blocks.len());

    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
    }
}

#[test]
fn debug_multiple_false_starts() {
    let scanner = VerbatimScanner::new();
    let text = r#"first false start:
    content without terminator

second false start:
    more content without terminator

finally a real one:
    actual verbatim content
(label)"#;

    println!("\n=== Multiple false starts ===");
    println!("Input text:");
    for (i, line) in text.lines().enumerate() {
        println!("{:2}: '{}'", i + 1, line);
    }

    let blocks = scanner.scan(text);
    println!("\nFound {} blocks:", blocks.len());

    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
    }
}
