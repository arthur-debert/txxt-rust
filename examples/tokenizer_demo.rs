use txxt::tokenizer::tokenize;

fn main() {
    // Example txxt content that demonstrates various features
    let sample_text = r#":: title :: Sample Document

This is a simple paragraph with *bold* and _italic_ text.

1. First item in a list
2. Second item with [a reference]
    - Nested unordered item
    - Another nested item with `code`

Code example:
    console.log("Hello, world!");
(javascript)

Term :: 
    This is a definition of the term.

- Math example: #E = mc^2#
- Citation example: [@smith2023]
- Footnote reference: [1]
"#;

    println!("Tokenizing sample txxt content...\n");

    let tokens = tokenize(sample_text);

    println!("Generated {} tokens:", tokens.len());
    println!("{:-<60}", "");

    for (i, token) in tokens.iter().enumerate() {
        let value_str = match &token.value {
            Some(v) if v.is_empty() => "<empty>".to_string(),
            Some(v) => format!("\"{}\"", v.replace('\n', "\\n")),
            None => "<none>".to_string(),
        };

        println!(
            "{:3}: {:>20} | {:>8}:{:<3} | {}",
            i + 1,
            format!("{:?}", token.token_type),
            token.line,
            token.column,
            value_str
        );
    }

    // Summary of token types
    println!("\n{:-<60}", "");
    println!("Token type summary:");

    let mut type_counts = std::collections::HashMap::new();
    for token in &tokens {
        *type_counts.entry(&token.token_type).or_insert(0) += 1;
    }

    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

    for (token_type, count) in sorted_types {
        println!("  {:>20}: {}", format!("{:?}", token_type), count);
    }
}
