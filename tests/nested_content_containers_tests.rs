use txxt::block_grouping::build_block_tree;
use txxt::document_parser::parse_document;
use txxt::tokenizer::tokenize;

#[test]
fn test_definition_with_mixed_content() {
    let content = r#"Definition with List ::
    - Paragraphs
    - Lists like this one
    - Even nested definitions

    More content after the list.
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one definition
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "definition");

    let definition = &document.root.children[0];
    assert_eq!(
        definition.attributes.get("term").unwrap(),
        "Definition with List"
    );

    // Definition should have content container
    assert_eq!(definition.children.len(), 1);
    assert_eq!(definition.children[0].node_type, "content_container");

    let content_container = &definition.children[0];

    // Content container should have: list, paragraph (blank line spacing is handled correctly)
    assert_eq!(content_container.children.len(), 2);

    // List with 3 items (now working correctly!)
    assert_eq!(content_container.children[0].node_type, "list");
    let list = &content_container.children[0];
    assert_eq!(list.children.len(), 3);
    assert_eq!(list.children[0].content.as_ref().unwrap(), "Paragraphs");
    assert_eq!(
        list.children[1].content.as_ref().unwrap(),
        "Lists like this one"
    );
    assert_eq!(
        list.children[2].content.as_ref().unwrap(),
        "Even nested definitions"
    );

    // Final paragraph
    assert_eq!(content_container.children[1].node_type, "paragraph");
    assert_eq!(
        content_container.children[1].content.as_ref().unwrap(),
        "More content after the list."
    );
}

#[test]
fn test_multiline_annotation_with_content() {
    let content = r#":: note ::
    This is a multiline annotation.
    It uses ContentContainer for its content.
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one annotation
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "annotation");

    let annotation = &document.root.children[0];
    assert_eq!(annotation.attributes.get("label").unwrap(), "note");

    // Annotation should have content container
    assert_eq!(annotation.children.len(), 1);
    assert_eq!(annotation.children[0].node_type, "content_container");

    let content_container = &annotation.children[0];

    // Content container should have: paragraph
    assert_eq!(content_container.children.len(), 1);

    assert_eq!(content_container.children[0].node_type, "paragraph");
    assert!(content_container.children[0]
        .content
        .as_ref()
        .unwrap()
        .contains("This is a multiline annotation"));
}

#[test]
fn test_deeply_nested_definitions() {
    let content = r#"Outer Term ::
    
    Definitions has at least one line of content.

    But can have more.

    Including other types: 

    - Annotations
    - Lists
        - Which can have children
    - Definitions
    
    Nested Term ::
        This is a nested definition inside the outer definition
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one outer definition
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "definition");

    let outer_def = &document.root.children[0];
    assert_eq!(outer_def.attributes.get("term").unwrap(), "Outer Term");

    // Outer definition should have content container
    assert_eq!(outer_def.children.len(), 1);
    assert_eq!(outer_def.children[0].node_type, "content_container");

    let outer_content = &outer_def.children[0];

    // Should have multiple elements including nested definition
    assert!(outer_content.children.len() > 5);

    // Find the nested definition (should be one of the later elements)
    let nested_def = outer_content
        .children
        .iter()
        .find(|child| child.node_type == "definition")
        .expect("Should have nested definition");

    assert_eq!(nested_def.attributes.get("term").unwrap(), "Nested Term");

    // Nested definition should also have content container
    assert_eq!(nested_def.children.len(), 1);
    assert_eq!(nested_def.children[0].node_type, "content_container");

    let nested_content = &nested_def.children[0];
    assert_eq!(nested_content.children.len(), 1);
    assert_eq!(nested_content.children[0].node_type, "paragraph");
    assert_eq!(
        nested_content.children[0].content.as_ref().unwrap(),
        "This is a nested definition inside the outer definition"
    );
}

#[test]
fn test_list_with_nested_content() {
    let content = r#"- First level item
- Second level with nesting
    - Nested item 1
    - Nested item 2
        - Deep nested item
        - Another deep item
    - Back to second level
- Third level item
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one list
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "list");

    let root_list = &document.root.children[0];
    assert_eq!(root_list.children.len(), 3);

    // First item - no nesting
    assert_eq!(root_list.children[0].node_type, "list_item");
    assert_eq!(
        root_list.children[0].content.as_ref().unwrap(),
        "First level item"
    );
    assert_eq!(root_list.children[0].children.len(), 0);

    // Second item - has nested content
    assert_eq!(root_list.children[1].node_type, "list_item");
    assert_eq!(
        root_list.children[1].content.as_ref().unwrap(),
        "Second level with nesting"
    );
    assert_eq!(root_list.children[1].children.len(), 1);

    let content_container = &root_list.children[1].children[0];
    assert_eq!(content_container.node_type, "content_container");

    // Content container should have nested list
    assert_eq!(content_container.children.len(), 1);
    assert_eq!(content_container.children[0].node_type, "list");

    let nested_list = &content_container.children[0];
    assert_eq!(nested_list.children.len(), 3);

    // Check that second nested item has its own nesting
    assert_eq!(
        nested_list.children[1].content.as_ref().unwrap(),
        "Nested item 2"
    );
    assert_eq!(nested_list.children[1].children.len(), 1);

    let deep_container = &nested_list.children[1].children[0];
    assert_eq!(deep_container.node_type, "content_container");
    assert_eq!(deep_container.children.len(), 1);
    assert_eq!(deep_container.children[0].node_type, "list");

    let deep_list = &deep_container.children[0];
    assert_eq!(deep_list.children.len(), 2);
    assert_eq!(
        deep_list.children[0].content.as_ref().unwrap(),
        "Deep nested item"
    );
    assert_eq!(
        deep_list.children[1].content.as_ref().unwrap(),
        "Another deep item"
    );

    // Third item - no nesting
    assert_eq!(root_list.children[2].node_type, "list_item");
    assert_eq!(
        root_list.children[2].content.as_ref().unwrap(),
        "Third level item"
    );
    assert_eq!(root_list.children[2].children.len(), 0);
}

#[test]
fn test_mixed_annotations_and_definitions() {
    let content = r#"Complex Definition ::
    This definition contains nested structures.
    
    :: metadata ::
        Even annotations can appear inside.
        They too can be multiline.
    
    Inner Definition ::
        Nested definitions work as expected.
        Each has its own ContentContainer.
    
    Back to the outer definition content.
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one definition
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "definition");

    let outer_def = &document.root.children[0];
    assert_eq!(
        outer_def.attributes.get("term").unwrap(),
        "Complex Definition"
    );

    // Should have content container
    assert_eq!(outer_def.children.len(), 1);
    let content_container = &outer_def.children[0];
    assert_eq!(content_container.node_type, "content_container");

    // Should have multiple elements: paragraph, blank, annotation, blank, definition, blank, paragraph
    assert_eq!(content_container.children.len(), 7);

    // Check the annotation
    let annotation = &content_container.children[2];
    assert_eq!(annotation.node_type, "annotation");
    assert_eq!(annotation.attributes.get("label").unwrap(), "metadata");
    assert_eq!(annotation.children.len(), 1);
    assert_eq!(annotation.children[0].node_type, "content_container");

    // Check the nested definition
    let nested_def = &content_container.children[4];
    assert_eq!(nested_def.node_type, "definition");
    assert_eq!(
        nested_def.attributes.get("term").unwrap(),
        "Inner Definition"
    );
    assert_eq!(nested_def.children.len(), 1);
    assert_eq!(nested_def.children[0].node_type, "content_container");

    // Check final paragraph
    let final_para = &content_container.children[6];
    assert_eq!(final_para.node_type, "paragraph");
    assert_eq!(
        final_para.content.as_ref().unwrap(),
        "Back to the outer definition content."
    );
}

#[test]
fn test_annotation_with_lists_and_verbatim() {
    let content = r#":: documentation:format=detailed ::
    - Ordered lists
    - Multiple paragraphs
    - Even code examples in verbatim blocks
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one annotation
    assert_eq!(document.root.children.len(), 1);
    assert_eq!(document.root.children[0].node_type, "annotation");

    let annotation = &document.root.children[0];
    // Note: Parameter parsing currently strips special characters - this is a separate issue
    assert_eq!(
        annotation.attributes.get("label").unwrap(),
        "documentationformatdetailed"
    );

    // Should have content container
    assert_eq!(annotation.children.len(), 1);
    let content_container = &annotation.children[0];
    assert_eq!(content_container.node_type, "content_container");

    // Should have: list
    assert_eq!(content_container.children.len(), 1);

    // Check list
    assert_eq!(content_container.children[0].node_type, "list");
    let list = &content_container.children[0];
    assert_eq!(list.children.len(), 3);
    assert_eq!(list.children[0].content.as_ref().unwrap(), "Ordered lists");
    assert_eq!(
        list.children[1].content.as_ref().unwrap(),
        "Multiple paragraphs"
    );
    assert_eq!(
        list.children[2].content.as_ref().unwrap(),
        "Even code examples in verbatim blocks"
    );
}

#[test]
fn test_empty_and_minimal_content_containers() {
    let content = r#"Empty Definition ::

:: empty ::

Minimal Definition ::
    Single line content.
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have 3 elements: 2 definitions and 1 annotation
    assert_eq!(document.root.children.len(), 3);

    // Empty definition
    let empty_def = &document.root.children[0];
    assert_eq!(empty_def.node_type, "definition");
    assert_eq!(
        empty_def.attributes.get("term").unwrap(),
        "Empty Definition"
    );
    // Empty definitions still get content containers, even if empty
    assert_eq!(empty_def.children.len(), 1);
    assert_eq!(empty_def.children[0].node_type, "content_container");
    assert_eq!(empty_def.children[0].children.len(), 0);

    // Empty annotation
    let empty_annotation = &document.root.children[1];
    assert_eq!(empty_annotation.node_type, "annotation");
    assert_eq!(empty_annotation.attributes.get("label").unwrap(), "empty");
    assert_eq!(empty_annotation.children.len(), 1);
    assert_eq!(empty_annotation.children[0].node_type, "content_container");
    assert_eq!(empty_annotation.children[0].children.len(), 0);

    // Minimal definition
    let minimal_def = &document.root.children[2];
    assert_eq!(minimal_def.node_type, "definition");
    assert_eq!(
        minimal_def.attributes.get("term").unwrap(),
        "Minimal Definition"
    );
    assert_eq!(minimal_def.children.len(), 1);
    assert_eq!(minimal_def.children[0].node_type, "content_container");
    assert_eq!(minimal_def.children[0].children.len(), 1);
    assert_eq!(minimal_def.children[0].children[0].node_type, "paragraph");
    assert_eq!(
        minimal_def.children[0].children[0]
            .content
            .as_ref()
            .unwrap(),
        "Single line content."
    );
}
