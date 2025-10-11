use std::env;
use std::fs;
use txxt::block_grouping::{build_block_tree, Block, BlockType};
use txxt::tokenizer::tokenize;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input.txxt>", args[0]);
        eprintln!("       {} <tokens.xml>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    let result = if input_path.ends_with(".txxt") {
        process_txxt_file(input_path)
    } else if input_path.ends_with(".tokens.xml") {
        process_tokens_xml_file(input_path)
    } else {
        eprintln!("Error: Input file must be .txxt or .tokens.xml");
        std::process::exit(1);
    };

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

fn process_tokens_xml_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Read tokens from XML and convert to our token format
    let xml_content = fs::read_to_string(path)?;
    let tokens = parse_tokens_from_xml(&xml_content)?;
    let block_tree = build_block_tree(tokens);
    Ok(format_blocks_as_xml(&block_tree, path))
}

fn parse_tokens_from_xml(
    xml_content: &str,
) -> Result<Vec<txxt::tokenizer::Token>, Box<dyn std::error::Error>> {
    use regex::Regex;
    use txxt::tokenizer::{Token, TokenType};

    let mut tokens = Vec::new();

    // Simple XML parsing - look for <item> blocks
    let item_pattern = Regex::new(r"(?s)<item>(.*?)</item>")?;
    let type_pattern = Regex::new(r"<type>(.*?)</type>")?;
    let line_pattern = Regex::new(r"<line>(\d+)</line>")?;
    let column_pattern = Regex::new(r"<column>(\d+)</column>")?;
    let value_pattern = Regex::new(r"(?s)<value>(.*?)</value>")?;

    for item_match in item_pattern.find_iter(xml_content) {
        let item_content = item_match.as_str();

        let token_type_str = type_pattern
            .find(item_content)
            .and_then(|m| type_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .unwrap_or("");

        let line = line_pattern
            .find(item_content)
            .and_then(|m| line_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        let column = column_pattern
            .find(item_content)
            .and_then(|m| column_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        let value = value_pattern
            .find(item_content)
            .and_then(|m| value_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .map(|m| decode_xml_entities(m.as_str()))
            .unwrap_or_default();

        // Convert token type string to TokenType enum
        let token_type = match token_type_str {
            "TEXT" => TokenType::Text,
            "NEWLINE" => TokenType::Newline,
            "BLANK_LINE" => TokenType::BlankLine,
            "INDENT" => TokenType::Indent,
            "DEDENT" => TokenType::Dedent,
            "SEQUENCE_MARKER" => TokenType::SequenceMarker,
            "DASH" => TokenType::Dash,
            "PRAGMA_MARKER" => TokenType::PragmaMarker,
            "VERBATIM_START" => TokenType::VerbatimStart,
            "VERBATIM_CONTENT" => TokenType::VerbatimContent,
            "VERBATIM_END" => TokenType::VerbatimEnd,
            "IDENTIFIER" => TokenType::Identifier,
            "STRING" => TokenType::String,
            "EQUALS" => TokenType::Equals,
            "COMMA" => TokenType::Comma,
            "COLON" => TokenType::Colon,
            "EMPHASIS_MARKER" => TokenType::EmphasisMarker,
            "STRONG_MARKER" => TokenType::StrongMarker,
            "CODE_MARKER" => TokenType::CodeMarker,
            "MATH_MARKER" => TokenType::MathMarker,
            "REF_MARKER" => TokenType::RefMarker,
            "SESSION_NUMBER" => TokenType::SessionNumber,
            "FOOTNOTE_NUMBER" => TokenType::FootnoteNumber,
            "CITATION" => TokenType::Citation,
            "DEFINITION_MARKER" => TokenType::DefinitionMarker,
            "VERBATIM_PLACEHOLDER" => TokenType::VerbatimPlaceholder,
            "EOF" => TokenType::Eof,
            _ => {
                eprintln!("Warning: Unknown token type: {}", token_type_str);
                continue;
            }
        };

        let token_value = if value.is_empty() { None } else { Some(value) };
        tokens.push(Token::new(token_type, token_value, line, column));
    }

    Ok(tokens)
}

fn decode_xml_entities(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#x27;", "'")
}

fn format_blocks_as_xml(block: &Block, source_path: &str) -> String {
    let mut xml = String::new();
    xml.push_str("<root>\n");
    xml.push_str(&format!("  <source>{}</source>\n", source_path));
    xml.push_str("  <blocks>\n");

    format_block_recursive(block, &mut xml, 2);

    xml.push_str("  </blocks>\n");
    xml.push_str("</root>\n");
    xml
}

fn format_block_recursive(block: &Block, xml: &mut String, indent_level: usize) {
    let indent = "  ".repeat(indent_level);

    match block {
        Block::Node(node) => {
            let block_type_name = match &node.block_type {
                BlockType::Root => "root",
                BlockType::Session { .. } => "session",
                BlockType::Paragraph { .. } => "paragraph",
                BlockType::List { .. } => "list",
                BlockType::ListItem { .. } => "list-item",
                BlockType::Definition { .. } => "definition",
                BlockType::Annotation { .. } => "annotation",
                BlockType::Verbatim { .. } => "verbatim",
                BlockType::TextLine { .. } => "text-line",
                BlockType::BlankLine { .. } => "blank-line",
            };

            xml.push_str(&format!("{}<block type=\"{}\"", indent, block_type_name));

            if let (Some(start), Some(end)) = (node.start_line, node.end_line) {
                xml.push_str(&format!(" start-line=\"{}\" end-line=\"{}\"", start, end));
            }

            xml.push_str(">\n");

            // Add block-specific content
            match &node.block_type {
                BlockType::Session { title_tokens } => {
                    if !title_tokens.is_empty() {
                        xml.push_str(&format!("{}  <title>", indent));
                        for token in title_tokens {
                            if let Some(value) = &token.value {
                                xml.push_str(&escape_xml(value));
                            }
                        }
                        xml.push_str("</title>\n");
                    }
                }
                BlockType::Annotation { label, .. } => {
                    xml.push_str(&format!(
                        "{}  <label>{}</label>\n",
                        indent,
                        escape_xml(label)
                    ));
                }
                BlockType::Definition { term_tokens, .. } => {
                    xml.push_str(&format!("{}  <term>", indent));
                    for token in term_tokens {
                        if let Some(value) = &token.value {
                            xml.push_str(&escape_xml(value));
                        }
                    }
                    xml.push_str("</term>\n");
                }
                BlockType::ListItem { marker_token, .. } => {
                    if let Some(marker) = &marker_token.value {
                        xml.push_str(&format!(
                            "{}  <marker>{}</marker>\n",
                            indent,
                            escape_xml(marker)
                        ));
                    }
                }
                BlockType::Paragraph { tokens } => {
                    xml.push_str(&format!("{}  <content>", indent));
                    for token in tokens {
                        if let Some(value) = &token.value {
                            xml.push_str(&escape_xml(value));
                        }
                    }
                    xml.push_str("</content>\n");
                }
                _ => {}
            }

            // Add container if present
            if let Some(container) = &node.container {
                match container.as_ref() {
                    txxt::block_grouping::blocks::Container::Content(content) => {
                        xml.push_str(&format!(
                            "{}  <content-container indent-level=\"{}\">\n",
                            indent, content.indent_level
                        ));
                        for child in &content.children {
                            format_block_recursive(child, xml, indent_level + 2);
                        }
                        xml.push_str(&format!("{}  </content-container>\n", indent));
                    }
                    txxt::block_grouping::blocks::Container::Session(session) => {
                        xml.push_str(&format!(
                            "{}  <session-container indent-level=\"{}\">\n",
                            indent, session.indent_level
                        ));
                        for child in &session.children {
                            format_block_recursive(child, xml, indent_level + 2);
                        }
                        xml.push_str(&format!("{}  </session-container>\n", indent));
                    }
                }
            }

            xml.push_str(&format!("{}</block>\n", indent));
        }

        Block::Container(container) => match container {
            txxt::block_grouping::blocks::Container::Content(content) => {
                xml.push_str(&format!(
                    "{}  <content-container indent-level=\"{}\">\n",
                    indent, content.indent_level
                ));
                for child in &content.children {
                    format_block_recursive(child, xml, indent_level + 2);
                }
                xml.push_str(&format!("{}  </content-container>\n", indent));
            }
            txxt::block_grouping::blocks::Container::Session(session) => {
                xml.push_str(&format!(
                    "{}  <session-container indent-level=\"{}\">\n",
                    indent, session.indent_level
                ));
                for child in &session.children {
                    format_block_recursive(child, xml, indent_level + 2);
                }
                xml.push_str(&format!("{}  </session-container>\n", indent));
            }
        },
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
