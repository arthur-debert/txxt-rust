use txxt::block_grouping::build_block_tree;
use txxt::document_parser::parse_document;
use txxt::tokenizer::tokenize;

#[test]
fn test_paragraph_parsing() {
    let text = "This is a simple paragraph.";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have a document root with a paragraph child
    assert_eq!(document.root.node_type, "document");
    assert_eq!(document.root.children.len(), 1);

    let paragraph = &document.root.children[0];
    assert_eq!(paragraph.node_type, "paragraph");
    assert_eq!(
        paragraph.content,
        Some("This is a simple paragraph.".to_string())
    );
}

#[test]
fn test_annotation_parsing() {
    let text = ":: author :: John Doe";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have a document root with an annotation child
    assert_eq!(document.root.node_type, "document");
    assert_eq!(document.root.children.len(), 1);

    let annotation = &document.root.children[0];
    assert_eq!(annotation.node_type, "annotation");
    assert_eq!(
        annotation.attributes.get("label"),
        Some(&"author".to_string())
    );
    assert_eq!(annotation.content, Some("John Doe".to_string()));
}

#[test]
fn test_definition_parsing() {
    let text = "Parser ::\n\n    A program that analyzes text.";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have a document root with a definition child
    assert_eq!(document.root.node_type, "document");
    assert!(!document.root.children.is_empty());

    // Find the definition node
    let definition = document
        .root
        .children
        .iter()
        .find(|child| child.node_type == "definition")
        .expect("Should have a definition node");

    assert_eq!(
        definition.attributes.get("term"),
        Some(&"Parser".to_string())
    );

    // Should have a content container with the definition content
    assert!(!definition.children.is_empty());
    let content_container = &definition.children[0];
    assert_eq!(content_container.node_type, "content_container");
}

#[test]
fn test_list_item_parsing() {
    let text = "- First item";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have a document root with a list child containing a list item
    assert_eq!(document.root.node_type, "document");
    assert_eq!(document.root.children.len(), 1);

    let list = &document.root.children[0];
    assert_eq!(list.node_type, "list");
    assert_eq!(list.children.len(), 1);

    let list_item = &list.children[0];
    assert_eq!(list_item.node_type, "list_item");
    assert_eq!(list_item.attributes.get("marker"), Some(&"- ".to_string()));
    assert_eq!(list_item.content, Some("First item".to_string()));
}

#[test]
fn test_blank_line_parsing() {
    let text = "First paragraph\n\nSecond paragraph";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have multiple children including a blank line
    assert_eq!(document.root.node_type, "document");
    assert!(document.root.children.len() >= 2);

    // Should have at least one blank line
    let has_blank_line = document
        .root
        .children
        .iter()
        .any(|child| child.node_type == "blank_line");
    assert!(has_blank_line);
}

#[test]
fn test_mixed_elements() {
    let text = ":: title :: Test Document\n\nThis is a paragraph.\n\n- List item\n\nTerm ::\n    Definition content";
    let tokens = tokenize(text);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should parse multiple different element types
    assert_eq!(document.root.node_type, "document");

    let element_types: Vec<&String> = document
        .root
        .children
        .iter()
        .map(|child| &child.node_type)
        .collect();

    // Should have annotation, paragraph, list, and definition
    assert!(element_types.contains(&&"annotation".to_string()));
    assert!(element_types.contains(&&"paragraph".to_string()));
    assert!(element_types.contains(&&"list".to_string()));
    assert!(element_types.contains(&&"definition".to_string()));
}
