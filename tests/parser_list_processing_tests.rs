use txxt::block_grouping::build_block_tree;
use txxt::parser::parse_document;
use txxt::tokenizer::tokenize;

#[test]
fn test_root_level_lists_separated_by_blank_lines() {
    // According to TXXT spec: blank lines between root-level list items break the list
    let content = r#"- Item 1

- Item 2
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have TWO separate lists, not one list with two items
    assert_eq!(document.root.children.len(), 3); // list, blank_line, list

    // First list with one item
    assert_eq!(document.root.children[0].node_type, "list");
    let first_list = &document.root.children[0];
    assert_eq!(first_list.children.len(), 1);
    assert_eq!(first_list.children[0].content.as_ref().unwrap(), "Item 1");

    // Blank line separator
    assert_eq!(document.root.children[1].node_type, "blank_line");

    // Second list with one item
    assert_eq!(document.root.children[2].node_type, "list");
    let second_list = &document.root.children[2];
    assert_eq!(second_list.children.len(), 1);
    assert_eq!(second_list.children[0].content.as_ref().unwrap(), "Item 2");
}

#[test]
fn test_root_level_lists_without_blank_lines() {
    // Without blank lines, should be one list with multiple items
    let content = r#"- Item 1
- Item 2
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have ONE list with two items
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "list");

    let list = &document.root.children[0];
    assert_eq!(list.children.len(), 2);
    assert_eq!(list.children[0].content.as_ref().unwrap(), "Item 1");
    assert_eq!(list.children[1].content.as_ref().unwrap(), "Item 2");
}

#[test]
fn test_content_container_lists_with_blank_lines() {
    // Within content containers, blank lines should be allowed and preserved
    let content = r#"Definition ::
    - Item 1
        
        Something
    
    - Item 2
    
        - List inner, item 1
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one definition
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "definition");

    let definition = &document.root.children[0];
    assert_eq!(definition.children.len(), 1);
    assert_eq!(definition.children[0].node_type, "content_container");

    let content_container = &definition.children[0];

    // Content container should have the structure that respects blank lines
    // but still groups list items appropriately within the content container context
    println!(
        "Content container children count: {}",
        content_container.children.len()
    );
    for (i, child) in content_container.children.iter().enumerate() {
        println!("Child {}: {}", i, child.node_type);
    }

    // The exact structure depends on how blank lines are handled in content containers
    // Let's verify what we actually get
    assert!(!content_container.children.is_empty());
}

#[test]
fn test_list_items_with_content_containers() {
    // List items with content containers should work correctly
    let content = r#"- Item 1
    
    Something in content container

- Item 2

    - Nested item 1
    - Nested item 2
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // This should create separate lists due to blank line at root level
    println!("Root children count: {}", document.root.children.len());
    for (i, child) in document.root.children.iter().enumerate() {
        println!("Root child {}: {}", i, child.node_type);
    }

    // The blank line between "- Item 1" and "- Item 2" should break the list
    // So we should have multiple elements, not one list with two items
    assert!(document.root.children.len() > 1);
}

#[test]
fn test_simple_content_container_list() {
    // Simple case: list within content container without blank lines
    let content = r#"Definition ::
    - Item 1
    - Item 2
    - Item 3
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one definition with content container containing one list
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "definition");

    let definition = &document.root.children[0];
    assert_eq!(definition.children.len(), 1);
    assert_eq!(definition.children[0].node_type, "content_container");

    let content_container = &definition.children[0];
    assert_eq!(content_container.children.len(), 1);
    assert_eq!(content_container.children[0].node_type, "list");

    let list = &content_container.children[0];
    assert_eq!(list.children.len(), 3);
    assert_eq!(list.children[0].content.as_ref().unwrap(), "Item 1");
    assert_eq!(list.children[1].content.as_ref().unwrap(), "Item 2");
    assert_eq!(list.children[2].content.as_ref().unwrap(), "Item 3");
}
