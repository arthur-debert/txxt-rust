use std::env;
use std::fs;
use txxt::block_grouping::{build_block_tree, TokenBlock};
use txxt::tokenizer::tokenize;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input.txxt>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    let result = process_txxt_file(input_path);

    match result {
        Ok(xml) => println!("{}", xml),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn process_txxt_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let tokens = tokenize(&content);
    let block_tree = build_block_tree(tokens);
    Ok(format_blocks_as_xml(&block_tree, path))
}

fn format_blocks_as_xml(block: &TokenBlock, source_path: &str) -> String {
    let mut xml = String::new();
    xml.push_str("<root>\n");
    xml.push_str(&format!("  <source>{}</source>\n", source_path));
    xml.push_str("  <blocks>\n");

    format_token_block_recursive(block, &mut xml, 2);

    xml.push_str("  </blocks>\n");
    xml.push_str("</root>\n");
    xml
}

fn format_token_block_recursive(block: &TokenBlock, xml: &mut String, indent_level: usize) {
    let indent = "  ".repeat(indent_level);

    // If this block has children, wrap it in a <block> tag
    if !block.children.is_empty() {
        xml.push_str(&format!("{}<block", indent));

        if let (Some(start), Some(end)) = (block.start_line, block.end_line) {
            xml.push_str(&format!(" start-line=\"{}\" end-line=\"{}\"", start, end));
        }

        xml.push_str(&format!(" indent-level=\"{}\"", block.indent_level));
        xml.push_str(">\n");

        // Add tokens for this block if any
        if !block.tokens.is_empty() {
            xml.push_str(&format!("{}  <tokens>\n", indent));
            for token in &block.tokens {
                xml.push_str(&format!(
                    "{}    <token type=\"{:?}\"",
                    indent, token.token_type
                ));
                if let Some(value) = &token.value {
                    xml.push_str(&format!(" value=\"{}\"", escape_xml(value)));
                }
                xml.push_str(&format!(
                    " line=\"{}\" column=\"{}\"/>\n",
                    token.line, token.column
                ));
            }
            xml.push_str(&format!("{}  </tokens>\n", indent));
        }

        // Add child blocks
        for child in &block.children {
            format_token_block_recursive(child, xml, indent_level + 1);
        }

        xml.push_str(&format!("{}</block>\n", indent));
    } else {
        // Leaf block - just show tokens
        xml.push_str(&format!("{}<block", indent));

        if let (Some(start), Some(end)) = (block.start_line, block.end_line) {
            xml.push_str(&format!(" start-line=\"{}\" end-line=\"{}\"", start, end));
        }

        xml.push_str(&format!(" indent-level=\"{}\"", block.indent_level));
        xml.push_str(">\n");

        if !block.tokens.is_empty() {
            xml.push_str(&format!("{}  <tokens>\n", indent));
            for token in &block.tokens {
                xml.push_str(&format!(
                    "{}    <token type=\"{:?}\"",
                    indent, token.token_type
                ));
                if let Some(value) = &token.value {
                    xml.push_str(&format!(" value=\"{}\"", escape_xml(value)));
                }
                xml.push_str(&format!(
                    " line=\"{}\" column=\"{}\"/>\n",
                    token.line, token.column
                ));
            }
            xml.push_str(&format!("{}  </tokens>\n", indent));
        }

        xml.push_str(&format!("{}</block>\n", indent));
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
