//! Test to understand token spans

use txxt::lexer::tokenize;

#[test]
fn analyze_token_spans() {
    let original = "Hello, world!";
    let tokens = tokenize(original);

    println!("Original: {:?}", original);
    println!("Original bytes: {:?}", original.as_bytes());
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  [{}] {:?}", i, token);
    }

    // Check the gaps
    println!("\nAnalyzing gaps:");
    for i in 0..tokens.len() - 1 {
        let span1 = tokens[i].span();
        let span2 = tokens[i + 1].span();

        if span1.end.row == span2.start.row {
            let gap = span2.start.column - span1.end.column;
            if gap > 0 {
                println!("  Gap of {} chars between token {} and {}", gap, i, i + 1);
                // Extract the gap content
                let start_byte = span1.end.column;
                let end_byte = span2.start.column;
                let gap_content = &original[start_byte..end_byte];
                println!("    Gap content: {:?}", gap_content);
            }
        }
    }
}
