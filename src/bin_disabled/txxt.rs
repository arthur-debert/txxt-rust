use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use std::env;
use std::fs;
use std::io::Cursor;
use std::io::{self, Read};
use txxt::ast::Document;
use txxt::block_grouping::{build_block_tree, TokenBlock};
use txxt::parser::parse_document;
use txxt::tokenizer::{tokenize, Token, TokenType};

#[cfg(feature = "new-ast")]
use txxt::adapters::convert_document;
#[cfg(feature = "new-ast")]
use txxt::ast_debug::AstTreeVisualizer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "lex" => {
            if args.len() != 3 {
                eprintln!("Usage: {} lex <input.txxt>", args[0]);
                std::process::exit(1);
            }
            handle_lex(&args[2]);
        }
        "block" => {
            if args.len() != 3 {
                eprintln!("Usage: {} block <input.txxt|input.tokens.xml>", args[0]);
                std::process::exit(1);
            }
            handle_block(&args[2]);
        }
        "parse" => {
            if args.len() < 3 || args.len() > 4 {
                eprintln!(
                    "Usage: {} parse [--new-ast] <input.txxt|input.tokens.xml|input.blocks.xml>",
                    args[0]
                );
                std::process::exit(1);
            }

            let (use_new_ast, input_path) = if args.len() == 4 && args[2] == "--new-ast" {
                (true, &args[3])
            } else if args.len() == 3 {
                (false, &args[2])
            } else {
                eprintln!(
                    "Usage: {} parse [--new-ast] <input.txxt|input.tokens.xml|input.blocks.xml>",
                    args[0]
                );
                std::process::exit(1);
            };

            handle_parse(input_path, use_new_ast);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <command> [args...]", program_name);
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  lex <input.txxt>                          Tokenize a TXXT file and output XML");
    eprintln!(
        "  block <input.txxt|input.tokens.xml>       Group tokens into blocks and output XML"
    );
    eprintln!(
        "  parse [--new-ast] <input.txxt|input.tokens.xml|input.blocks.xml> Parse into AST and output XML or tree"
    );
}

fn handle_lex(input_path: &str) {
    let result = if input_path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        tokenize_content(&buffer, "stdin")
    } else {
        tokenize_file(input_path)
    };

    match result {
        Ok(xml) => println!("{}", xml),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_block(input_path: &str) {
    let result = if input_path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        let tokens = tokenize(&buffer);
        let block_tree = build_block_tree(tokens);
        format_blocks_as_xml(&block_tree, "stdin")
    } else if input_path.ends_with(".txxt") {
        process_txxt_file_for_blocks(input_path)
    } else if input_path.ends_with(".tokens.xml") {
        process_tokens_xml_file(input_path)
    } else {
        Err("Input file must be .txxt or .tokens.xml".into())
    };

    match result {
        Ok(xml) => println!("{}", xml),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_parse(input_path: &str, use_new_ast: bool) {
    let result = if input_path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        let tokens = tokenize(&buffer);
        let block_tree = build_block_tree(tokens);
        let document = parse_document("stdin".to_string(), &block_tree);

        if use_new_ast {
            format_new_ast_as_tree(&document)
        } else {
            format_ast_as_xml(&document)
        }
    } else if input_path.ends_with(".txxt") {
        // TXXT file - tokenize, block group, then parse
        process_txxt_file_for_parse(input_path, use_new_ast)
    } else if input_path.ends_with(".tokens.xml") {
        // XML tokens file - parse tokens, block group, then parse
        process_tokens_xml_file_for_parse(input_path, use_new_ast)
    } else if input_path.ends_with(".blocks.xml") {
        // XML blocks file - parse blocks then parse to AST
        process_blocks_xml_file_for_parse(input_path, use_new_ast)
    } else {
        Err("Input file must be .txxt, .tokens.xml, or .blocks.xml".into())
    };

    match result {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn tokenize_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    tokenize_content(&content, path)
}

fn tokenize_content(
    content: &str,
    source_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let tokens = tokenize(content);
    format_tokens_as_xml(&tokens, source_name)
}

fn process_txxt_file_for_blocks(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let tokens = tokenize(&content);
    let block_tree = build_block_tree(tokens);
    format_blocks_as_xml(&block_tree, path)
}

fn process_tokens_xml_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string(path)?;
    let tokens = parse_tokens_from_xml(&xml_content)?;
    let block_tree = build_block_tree(tokens);
    format_blocks_as_xml(&block_tree, path)
}

fn format_tokens_as_xml(
    tokens: &[Token],
    source_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut writer = Writer::new_with_indent(Cursor::new(&mut buffer), b' ', 2);

    // Write XML declaration
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )))?;

    // Root element
    let tokens_elem = BytesStart::new("tokens");
    writer.write_event(Event::Start(tokens_elem))?;

    // Source element
    let source_elem = BytesStart::new("source");
    writer.write_event(Event::Start(source_elem))?;
    writer.write_event(Event::Text(BytesText::new(source_name)))?;
    writer.write_event(Event::End(BytesEnd::new("source")))?;

    // Write each token as an item
    for token in tokens {
        write_token_as_item(&mut writer, token)?;
    }

    writer.write_event(Event::End(BytesEnd::new("tokens")))?;

    Ok(String::from_utf8(buffer)?)
}

fn write_token_as_item(
    writer: &mut Writer<Cursor<&mut Vec<u8>>>,
    token: &Token,
) -> Result<(), Box<dyn std::error::Error>> {
    let item_elem = BytesStart::new("item");
    writer.write_event(Event::Start(item_elem))?;

    // Type
    let type_elem = BytesStart::new("type");
    writer.write_event(Event::Start(type_elem))?;
    writer.write_event(Event::Text(BytesText::new(&format!(
        "{:?}",
        token.token_type
    ))))?;
    writer.write_event(Event::End(BytesEnd::new("type")))?;

    // Line
    let line_elem = BytesStart::new("line");
    writer.write_event(Event::Start(line_elem))?;
    writer.write_event(Event::Text(BytesText::new(&token.line.to_string())))?;
    writer.write_event(Event::End(BytesEnd::new("line")))?;

    // Column
    let column_elem = BytesStart::new("column");
    writer.write_event(Event::Start(column_elem))?;
    writer.write_event(Event::Text(BytesText::new(&token.column.to_string())))?;
    writer.write_event(Event::End(BytesEnd::new("column")))?;

    // Value
    let value_elem = BytesStart::new("value");
    writer.write_event(Event::Start(value_elem))?;
    if let Some(value) = &token.value {
        writer.write_event(Event::Text(BytesText::new(value)))?;
    }
    writer.write_event(Event::End(BytesEnd::new("value")))?;

    writer.write_event(Event::End(BytesEnd::new("item")))?;
    Ok(())
}

fn format_blocks_as_xml(
    block: &TokenBlock,
    source_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut writer = Writer::new_with_indent(Cursor::new(&mut buffer), b' ', 2);

    // Write XML declaration
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )))?;

    // Root element - use "tokens" to match the lex output structure
    let tokens_elem = BytesStart::new("tokens");
    writer.write_event(Event::Start(tokens_elem))?;

    // Source element
    let source_elem = BytesStart::new("source");
    writer.write_event(Event::Start(source_elem))?;
    writer.write_event(Event::Text(BytesText::new(source_path)))?;
    writer.write_event(Event::End(BytesEnd::new("source")))?;

    // Write the block structure
    write_token_block_recursive(&mut writer, block)?;

    writer.write_event(Event::End(BytesEnd::new("tokens")))?;

    Ok(String::from_utf8(buffer)?)
}

fn write_token_block_recursive(
    writer: &mut Writer<Cursor<&mut Vec<u8>>>,
    block: &TokenBlock,
) -> Result<(), Box<dyn std::error::Error>> {
    // If this block has children, wrap everything in a <block> tag
    if !block.children.is_empty() {
        let block_elem = BytesStart::new("block");
        writer.write_event(Event::Start(block_elem))?;

        // Write tokens for this block first (excluding INDENT/DEDENT/EOF)
        for token in &block.tokens {
            if !matches!(
                token.token_type,
                TokenType::Indent | TokenType::Dedent | TokenType::Eof
            ) {
                write_token_as_item(writer, token)?;
            }
        }

        // Write child blocks recursively
        for child in &block.children {
            write_token_block_recursive(writer, child)?;
        }

        writer.write_event(Event::End(BytesEnd::new("block")))?;
    } else {
        // Leaf block - just write tokens as items (excluding INDENT/DEDENT/EOF)
        for token in &block.tokens {
            if !matches!(
                token.token_type,
                TokenType::Indent | TokenType::Dedent | TokenType::Eof
            ) {
                write_token_as_item(writer, token)?;
            }
        }
    }

    Ok(())
}

fn parse_tokens_from_xml(xml_content: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
    use regex::Regex;

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
            "Text" => TokenType::Text,
            "Newline" => TokenType::Newline,
            "BlankLine" => TokenType::BlankLine,
            "Indent" => TokenType::Indent,
            "Dedent" => TokenType::Dedent,
            "SequenceMarker" => TokenType::SequenceMarker,
            "Dash" => TokenType::Dash,
            "PragmaMarker" => TokenType::PragmaMarker,
            "VerbatimStart" => TokenType::VerbatimStart,
            "VerbatimContent" => TokenType::VerbatimContent,
            "VerbatimEnd" => TokenType::VerbatimEnd,
            "Identifier" => TokenType::Identifier,
            "String" => TokenType::String,
            "Equals" => TokenType::Equals,
            "Comma" => TokenType::Comma,
            "Colon" => TokenType::Colon,
            "EmphasisMarker" => TokenType::EmphasisMarker,
            "StrongMarker" => TokenType::StrongMarker,
            "CodeMarker" => TokenType::CodeMarker,
            "MathMarker" => TokenType::MathMarker,
            "RefMarker" => TokenType::RefMarker,
            "SessionNumber" => TokenType::SessionNumber,
            "FootnoteNumber" => TokenType::FootnoteNumber,
            "Citation" => TokenType::Citation,
            "DefinitionMarker" => TokenType::DefinitionMarker,
            "VerbatimPlaceholder" => TokenType::VerbatimPlaceholder,
            "Eof" => TokenType::Eof,
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

// Parse command helper functions
fn process_txxt_file_for_parse(
    path: &str,
    use_new_ast: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let tokens = tokenize(&content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document(path.to_string(), &block_tree);

    if use_new_ast {
        format_new_ast_as_tree(&document)
    } else {
        format_ast_as_xml(&document)
    }
}

fn process_tokens_xml_file_for_parse(
    path: &str,
    use_new_ast: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string(path)?;
    let tokens = parse_tokens_from_xml(&xml_content)?;
    let block_tree = build_block_tree(tokens);
    let document = parse_document(path.to_string(), &block_tree);

    if use_new_ast {
        format_new_ast_as_tree(&document)
    } else {
        format_ast_as_xml(&document)
    }
}

fn process_blocks_xml_file_for_parse(
    _path: &str,
    _use_new_ast: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Parse blocks XML format
    // For now, just return an error since we haven't implemented blocks XML parsing yet
    Err("Parsing from .blocks.xml not yet implemented".into())
}

/// Format document using new AST adapter and tree visualizer
fn format_new_ast_as_tree(document: &Document) -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(feature = "new-ast")]
    {
        let new_doc = convert_document(document);
        let visualizer = AstTreeVisualizer::new();
        Ok(visualizer.visualize_document(&new_doc))
    }

    #[cfg(not(feature = "new-ast"))]
    {
        Err(
            "New AST feature is not enabled. Compile with --features new-ast to use this option."
                .into(),
        )
    }
}

fn format_ast_as_xml(document: &Document) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut writer = Writer::new_with_indent(Cursor::new(&mut buffer), b' ', 2);

    // Write XML declaration
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )))?;

    // Root element
    let ast_elem = BytesStart::new("ast");
    writer.write_event(Event::Start(ast_elem))?;

    // Source element
    let source_elem = BytesStart::new("source");
    writer.write_event(Event::Start(source_elem))?;
    writer.write_event(Event::Text(BytesText::new(&document.source)))?;
    writer.write_event(Event::End(BytesEnd::new("source")))?;

    // Write the AST structure
    write_ast_node_recursive(&mut writer, &document.root)?;

    writer.write_event(Event::End(BytesEnd::new("ast")))?;

    Ok(String::from_utf8(buffer)?)
}

fn write_ast_node_recursive(
    writer: &mut Writer<Cursor<&mut Vec<u8>>>,
    node: &txxt::ast::AstNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut node_elem = BytesStart::new("node");
    node_elem.push_attribute(("type", node.node_type.as_str()));

    if let Some(start) = node.start_line {
        node_elem.push_attribute(("start-line", start.to_string().as_str()));
    }
    if let Some(end) = node.end_line {
        node_elem.push_attribute(("end-line", end.to_string().as_str()));
    }

    writer.write_event(Event::Start(node_elem))?;

    // Write attributes
    if !node.attributes.is_empty() {
        let attrs_elem = BytesStart::new("attributes");
        writer.write_event(Event::Start(attrs_elem))?;

        for (key, value) in &node.attributes {
            let mut attr_elem = BytesStart::new("attribute");
            attr_elem.push_attribute(("key", key.as_str()));
            attr_elem.push_attribute(("value", value.as_str()));
            writer.write_event(Event::Empty(attr_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("attributes")))?;
    }

    // Write content
    if let Some(content) = &node.content {
        let content_elem = BytesStart::new("content");
        writer.write_event(Event::Start(content_elem))?;
        writer.write_event(Event::Text(BytesText::new(content)))?;
        writer.write_event(Event::End(BytesEnd::new("content")))?;
    }

    // Write children
    if !node.children.is_empty() {
        let children_elem = BytesStart::new("children");
        writer.write_event(Event::Start(children_elem))?;

        for child in &node.children {
            write_ast_node_recursive(writer, child)?;
        }

        writer.write_event(Event::End(BytesEnd::new("children")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("node")))?;
    Ok(())
}
