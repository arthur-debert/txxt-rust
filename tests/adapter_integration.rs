//! Integration tests for AST adapter functionality
//!
//! These tests verify that the adapter correctly converts from the old AST
//! to the new AST structure, preserving semantic information and enabling
//! end-to-end validation.

#[cfg(feature = "new-ast")]
mod new_ast_tests {
    use txxt::adapters::convert_document;
    use txxt::ast::{AstNode, Document as OldDocument};

    #[test]
    fn test_empty_document_conversion() {
        let old_doc = OldDocument::new("test.txxt".to_string());
        let new_doc = convert_document(&old_doc);

        // Should have empty blocks but valid metadata
        assert!(new_doc.blocks.is_empty());
        assert_eq!(
            new_doc.assembly_info.source_path,
            Some("test.txxt".to_string())
        );
        assert!(!new_doc.assembly_info.parser_version.is_empty());
    }

    #[test]
    fn test_simple_paragraph_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());
        let paragraph = AstNode::with_content("paragraph".to_string(), "Hello world".to_string());
        old_doc.root.add_child(paragraph);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);

        if let txxt::ast::blocks::Block::Paragraph(ref para) = new_doc.blocks[0] {
            assert!(!para.content.is_empty());
            assert!(!para.tokens.tokens.is_empty());
        } else {
            panic!("Expected paragraph block");
        }
    }

    #[test]
    fn test_list_with_numbering_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());

        // Create a numbered list
        let mut list_node = AstNode::new("list".to_string());
        list_node.set_attribute("list_style".to_string(), "numerical".to_string());
        list_node.set_attribute("list_form".to_string(), "short".to_string());

        // Add list items
        let mut item1 = AstNode::new("list_item".to_string());
        item1.set_attribute("marker".to_string(), "1.".to_string());
        item1.content = Some("First item".to_string());
        list_node.add_child(item1);

        let mut item2 = AstNode::new("list_item".to_string());
        item2.set_attribute("marker".to_string(), "2.".to_string());
        item2.content = Some("Second item".to_string());
        list_node.add_child(item2);

        old_doc.root.add_child(list_node);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);

        if let txxt::ast::blocks::Block::List(ref list) = new_doc.blocks[0] {
            assert_eq!(
                list.decoration_type.style,
                txxt::ast::structure::NumberingStyle::Numerical
            );
            assert_eq!(
                list.decoration_type.form,
                txxt::ast::structure::NumberingForm::Short
            );
            assert_eq!(list.items.len(), 2);
            assert_eq!(list.items[0].marker, "1.");
            assert_eq!(list.items[1].marker, "2.");
        } else {
            panic!("Expected list block");
        }
    }

    #[test]
    fn test_session_with_numbering_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());

        let mut session = AstNode::new("session".to_string());
        session.set_attribute("title".to_string(), "Introduction".to_string());
        session.set_attribute("numbering".to_string(), "1.".to_string());

        // Add session content
        let content = AstNode::with_content(
            "paragraph".to_string(),
            "This is the introduction.".to_string(),
        );
        session.add_child(content);

        old_doc.root.add_child(session);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);

        if let txxt::ast::blocks::Block::Session(ref session) = new_doc.blocks[0] {
            assert!(!session.title.content.is_empty());
            assert!(session.title.numbering.is_some());

            if let Some(ref numbering) = session.title.numbering {
                assert_eq!(numbering.marker, "1.");
                assert_eq!(
                    numbering.style,
                    txxt::ast::structure::NumberingStyle::Numerical
                );
            }

            assert!(!session.content.content.is_empty());
        } else {
            panic!("Expected session block");
        }
    }

    #[test]
    fn test_verbatim_block_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());

        let mut verbatim = AstNode::new("verbatim".to_string());
        verbatim.content = Some("let x = 42;\nprintln!(\"{}\", x);".to_string());
        verbatim.set_attribute("format".to_string(), "rust".to_string());
        verbatim.set_attribute("verbatim_type".to_string(), "stretched".to_string());

        old_doc.root.add_child(verbatim);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);

        if let txxt::ast::blocks::Block::VerbatimBlock(ref verbatim) = new_doc.blocks[0] {
            assert_eq!(verbatim.raw, "let x = 42;\nprintln!(\"{}\", x);");
            assert_eq!(verbatim.format_hint, Some("rust".to_string()));
            assert_eq!(
                verbatim.verbatim_type,
                txxt::ast::blocks::VerbatimType::Stretched
            );
        } else {
            panic!("Expected verbatim block");
        }
    }

    #[test]
    fn test_definition_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());

        let mut definition = AstNode::new("definition".to_string());
        definition.set_attribute("term".to_string(), "TXXT".to_string());
        definition.set_attribute("ref".to_string(), "txxt-def".to_string());

        // Add definition content
        let content =
            AstNode::with_content("paragraph".to_string(), "A text markup format.".to_string());
        definition.add_child(content);

        old_doc.root.add_child(definition);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);

        if let txxt::ast::blocks::Block::Definition(ref def) = new_doc.blocks[0] {
            assert!(!def.term.content.is_empty());
            assert!(!def.content.content.is_empty());
            assert!(def.parameters.map.contains_key("ref"));
            assert_eq!(def.parameters.map.get("ref"), Some(&"txxt-def".to_string()));
        } else {
            panic!("Expected definition block");
        }
    }

    #[test]
    fn test_document_metadata_extraction() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());

        // Add document-level metadata
        old_doc
            .root
            .set_attribute("title".to_string(), "Test Document".to_string());
        old_doc
            .root
            .set_attribute("authors".to_string(), "Alice, Bob".to_string());
        old_doc
            .root
            .set_attribute("date".to_string(), "2024-01-01".to_string());
        old_doc
            .root
            .set_attribute("custom_field".to_string(), "custom_value".to_string());

        let new_doc = convert_document(&old_doc);

        // Check metadata extraction
        assert!(new_doc.meta.title.is_some());
        if let Some(txxt::ast::base::MetaValue::String(ref title)) = new_doc.meta.title {
            assert_eq!(title, "Test Document");
        }

        assert_eq!(new_doc.meta.authors.len(), 2);

        assert!(new_doc.meta.date.is_some());
        if let Some(txxt::ast::base::MetaValue::String(ref date)) = new_doc.meta.date {
            assert_eq!(date, "2024-01-01");
        }

        assert!(new_doc.meta.custom.contains_key("custom_field"));
    }

    #[test]
    fn test_nested_structure_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());

        // Create a session with nested list
        let mut session = AstNode::new("session".to_string());
        session.set_attribute("title".to_string(), "Chapter 1".to_string());

        let mut container = AstNode::new("container".to_string());

        let mut list = AstNode::new("list".to_string());
        let item = AstNode::with_content("list_item".to_string(), "Item 1".to_string());
        list.add_child(item);

        container.add_child(list);
        session.add_child(container);
        old_doc.root.add_child(session);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);

        if let txxt::ast::blocks::Block::Session(ref session) = new_doc.blocks[0] {
            assert!(!session.content.content.is_empty());

            // Should have container with nested list
            if let txxt::ast::blocks::Block::Container(ref container) = session.content.content[0] {
                assert!(!container.content.is_empty());
                if let txxt::ast::blocks::Block::List(ref list) = container.content[0] {
                    assert_eq!(list.items.len(), 1);
                } else {
                    panic!("Expected nested list in container");
                }
            } else {
                panic!("Expected container in session");
            }
        } else {
            panic!("Expected session block");
        }
    }

    #[test]
    fn test_unknown_node_fallback() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());

        // Create an unknown node type
        let unknown = AstNode::with_content("unknown_type".to_string(), "Some content".to_string());
        old_doc.root.add_child(unknown);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);

        // Should fallback to paragraph
        if let txxt::ast::blocks::Block::Paragraph(ref para) = new_doc.blocks[0] {
            assert!(!para.content.is_empty());
        } else {
            panic!("Expected fallback to paragraph");
        }
    }
}

#[cfg(not(feature = "new-ast"))]
mod compatibility_tests {
    #[test]
    fn test_new_ast_not_available() {
        // This test just ensures the code compiles when new-ast feature is disabled
        assert!(true);
    }
}
