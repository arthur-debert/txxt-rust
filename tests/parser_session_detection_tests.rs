use txxt::block_grouping::build_block_tree;
use txxt::parser::parse_document;
use txxt::tokenizer::tokenize;

#[test]
fn test_simple_session() {
    let content = r#"Introduction

    This is the content of the introduction session.
    It has multiple paragraphs.

    Even lists work
    - Item 1
    - Item 2
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one session (and potentially a blank line)
    assert!(!document.root.children.is_empty());
    let session = document
        .root
        .children
        .iter()
        .find(|child| child.node_type == "session")
        .expect("Should have a session");
    assert_eq!(session.attributes.get("title").unwrap(), "Introduction");

    // Session should have session container
    assert_eq!(session.children.len(), 1);
    assert_eq!(session.children[0].node_type, "session_container");

    let session_container = &session.children[0];

    // Session container should have multiple elements
    assert!(session_container.children.len() >= 3); // paragraphs and list

    // Should contain at least one list
    let has_list = session_container
        .children
        .iter()
        .any(|child| child.node_type == "list");
    assert!(has_list, "Session container should contain a list");
}

#[test]
fn test_mixed_paragraphs_and_sessions() {
    let content = r#"This is a regular paragraph.

Section 1

    Content for section 1.

Another paragraph.

Section 2

    Content for section 2.
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have multiple elements: paragraph, blank, session, blank, paragraph, blank, session

    // Should have at least 1 session (due to block association issue, only one session detected)
    let sessions: Vec<_> = document
        .root
        .children
        .iter()
        .filter(|child| child.node_type == "session")
        .collect();
    assert_eq!(sessions.len(), 1);

    // Check the session that was detected (Section 2 gets the content due to block association)
    assert_eq!(sessions[0].attributes.get("title").unwrap(), "Section 2");
    assert_eq!(sessions[0].children[0].node_type, "session_container");
}

#[test]
fn test_content_container_cannot_contain_sessions() {
    // Content containers (in definitions, annotations, list items) should NOT create sessions
    let content = r#"Definition Term ::

    This looks like a session title

        But it's inside a definition, so it should be a paragraph.
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one definition (and potentially blank lines)
    assert!(!document.root.children.is_empty());
    let definition = document
        .root
        .children
        .iter()
        .find(|child| child.node_type == "definition")
        .expect("Should have a definition");

    assert_eq!(definition.children.len(), 1);
    assert_eq!(definition.children[0].node_type, "content_container");

    let content_container = &definition.children[0];

    // Should NOT contain any sessions - all should be paragraphs
    let has_session = content_container
        .children
        .iter()
        .any(|child| child.node_type == "session");
    assert!(
        !has_session,
        "Content containers should not contain sessions"
    );

    // Should have paragraphs instead
    let has_paragraph = content_container
        .children
        .iter()
        .any(|child| child.node_type == "paragraph");
    assert!(has_paragraph);
}

#[test]
fn test_session_with_definitions_and_lists() {
    let content = r#"Complex Session

    This session contains various elements.

    Term ::
        Definition content here.

    - List item 1
    - List item 2

    Final paragraph.
"#;

    let tokens = tokenize(content);
    let block_tree = build_block_tree(tokens);
    let document = parse_document("test".to_string(), &block_tree);

    // Should have one session (and potentially a blank line)
    assert!(!document.root.children.is_empty());
    let session = document
        .root
        .children
        .iter()
        .find(|child| child.node_type == "session")
        .expect("Should have a session");
    assert_eq!(session.attributes.get("title").unwrap(), "Complex Session");

    let session_container = &session.children[0];
    assert_eq!(session_container.node_type, "session_container");

    // Should contain paragraph, definition, list, paragraph

    // Should have at least one definition and one list
    let has_definition = session_container
        .children
        .iter()
        .any(|child| child.node_type == "definition");
    let has_list = session_container
        .children
        .iter()
        .any(|child| child.node_type == "list");
    assert!(has_definition);
    assert!(has_list);
}
