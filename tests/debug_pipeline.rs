//! Debug the parsing pipeline to understand AST structure

use txxt::block_grouping::build_block_tree;
use txxt::parser::parse_document;
use txxt::tokenizer::tokenize;

fn debug_parse(source: &str, description: &str) {
    println!("\n=== {} ===", description);
    println!("Source: {:?}", source);

    let tokens = tokenize(source);
    println!("Tokens: {} items", tokens.len());

    let block_tree = build_block_tree(tokens);
    println!("Block tree: {} children", block_tree.children.len());

    let old_doc = parse_document("test.txxt".to_string(), &block_tree);
    println!("Old AST root type: {}", old_doc.root.node_type);
    println!("Old AST children: {}", old_doc.root.children.len());

    for (i, child) in old_doc.root.children.iter().enumerate() {
        print_node_recursive(child, i, "  ");
    }
}

fn print_node_recursive(node: &txxt::ast::AstNode, index: usize, indent: &str) {
    println!(
        "{}[{}] type: '{}', content: {:?}, attrs: {:?}",
        indent, index, node.node_type, node.content, node.attributes
    );

    for (i, child) in node.children.iter().enumerate() {
        print_node_recursive(child, i, &format!("{}  ", indent));
    }
}

#[test]
fn test_debug_simple_list() {
    debug_parse("- First item\n- Second item", "Simple List");
}

#[test]
fn test_debug_numbered_list() {
    debug_parse("1. First item\n2. Second item", "Numbered List");
}

#[test]
fn test_debug_session() {
    debug_parse("1. Introduction\n\nThis is the introduction section with some content.\n\nHere's another paragraph in the introduction.", "Session");
}

#[test]
fn test_debug_definition() {
    debug_parse("Term:\n    Definition content.", "Definition");
}

#[test]
fn test_debug_mixed() {
    debug_parse(
        "1. Session\n\n    Content here.\n    \n    - List item",
        "Mixed Content",
    );
}

#[test]
fn test_debug_verbatim() {
    debug_parse(
        "Example Code:\n    def hello():\n        print(\"Hello World\")\n        return 42\npython",
        "Verbatim Code Block",
    );
}

#[test]
fn test_debug_nested_complex() {
    debug_parse(
        "1. Main Section\n\n    - Nested list item 1\n    - Nested list item 2",
        "Nested Structure",
    );
}
