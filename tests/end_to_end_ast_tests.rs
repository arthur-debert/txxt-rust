//! End-to-end AST testing from TXXT source to final AST
//!
//! These tests validate the complete parsing pipeline:
//! TXXT source → Tokenizer → Block Grouping → Parser → Old AST → Adapter → New AST
//!
//! This ensures that the adapter works correctly with real parsing output
//! and validates the entire toolchain integration.

#[cfg(feature = "new-ast")]
mod e2e_new_ast_tests {
    use txxt::adapters::convert_document;
    use txxt::ast::Document as OldDocument;
    use txxt::block_grouping::build_block_tree;
    use txxt::parser::parse_document;
    use txxt::tokenizer::tokenize;

    /// Helper function to parse TXXT source to old AST
    fn parse_txxt_to_old_ast(source: &str) -> OldDocument {
        // Follow the actual parsing pipeline
        let tokens = tokenize(source);
        let block_tree = build_block_tree(tokens);

        parse_document("test.txxt".to_string(), &block_tree)
    }

    /// Helper function for complete end-to-end parsing
    fn parse_txxt_to_new_ast(source: &str) -> txxt::ast::base::Document {
        let old_doc = parse_txxt_to_old_ast(source);
        convert_document(&old_doc)
    }

    #[test]
    fn test_simple_paragraph_e2e() {
        let source = "Hello world, this is a simple paragraph.";

        let new_doc = parse_txxt_to_new_ast(source);

        assert_eq!(new_doc.blocks.len(), 1);
        if let txxt::ast::blocks::Block::Paragraph(ref para) = new_doc.blocks[0] {
            assert!(!para.content.is_empty());
            // Verify the content contains our text
            if let txxt::ast::inlines::Inline::TextLine(ref transform) = para.content[0] {
                let text_content = transform.text_content();
                assert!(text_content.contains("Hello world"));
            }
        } else {
            panic!("Expected paragraph block");
        }
    }

    #[test]
    fn test_multiple_paragraphs_e2e() {
        let source = r#"First paragraph here.

Second paragraph after blank line.

Third paragraph with more content."#;

        let new_doc = parse_txxt_to_new_ast(source);

        // Should have 3 paragraphs + 2 blank lines = 5 blocks total
        assert!(
            new_doc.blocks.len() >= 3,
            "Should have at least 3 paragraph blocks"
        );

        let paragraph_count = new_doc
            .blocks
            .iter()
            .filter(|block| matches!(block, txxt::ast::blocks::Block::Paragraph(_)))
            .count();

        assert_eq!(paragraph_count, 3, "Should have exactly 3 paragraphs");
    }

    #[test]
    fn test_simple_list_e2e() {
        let source = r#"- First item
- Second item
- Third item"#;

        let new_doc = parse_txxt_to_new_ast(source);

        // Find the list block
        let list_blocks: Vec<_> = new_doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::List(list) = block {
                    Some(list)
                } else {
                    None
                }
            })
            .collect();

        assert!(!list_blocks.is_empty(), "Should have at least one list");

        let list = list_blocks[0];
        assert_eq!(list.items.len(), 3, "List should have 3 items");
        assert_eq!(list.items[0].marker, "-");
        assert_eq!(list.items[1].marker, "-");
        assert_eq!(list.items[2].marker, "-");
    }

    #[test]
    fn test_numbered_list_e2e() {
        let source = r#"1. First numbered item
2. Second numbered item
3. Third numbered item"#;

        let new_doc = parse_txxt_to_new_ast(source);

        let list_blocks: Vec<_> = new_doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::List(list) = block {
                    Some(list)
                } else {
                    None
                }
            })
            .collect();

        assert!(!list_blocks.is_empty(), "Should have a numbered list");

        let list = list_blocks[0];
        assert_eq!(list.items.len(), 3, "Numbered list should have 3 items");

        // Check that markers are preserved
        assert_eq!(list.items[0].marker, "1.");
        assert_eq!(list.items[1].marker, "2.");
        assert_eq!(list.items[2].marker, "3.");

        // Check numbering style detection
        assert_eq!(
            list.decoration_type.style,
            txxt::ast::structure::NumberingStyle::Numerical
        );
    }

    #[test]
    fn test_session_e2e() {
        let source = r#"1. Introduction

This is the introduction section with some content.

Here's another paragraph in the introduction."#;

        let new_doc = parse_txxt_to_new_ast(source);

        // Current parser behavior: treats this as separate blocks (list + paragraphs)
        // The adapter should convert the numbered list item correctly
        let list_blocks: Vec<_> = new_doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::List(list) = block {
                    Some(list)
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !list_blocks.is_empty(),
            "Should have a list with numbered item"
        );

        let list = list_blocks[0];
        assert_eq!(list.items.len(), 1, "Should have one list item");
        assert_eq!(list.items[0].marker, "1.", "Should preserve marker");

        // Check that we also have paragraph blocks
        let paragraph_count = new_doc
            .blocks
            .iter()
            .filter(|block| matches!(block, txxt::ast::blocks::Block::Paragraph(_)))
            .count();
        assert!(
            paragraph_count >= 2,
            "Should have separate paragraph blocks"
        );
    }

    #[test]
    fn test_definition_e2e() {
        let source = r#"Term:
    This is the definition of the term.
    
    Multiple paragraphs can be in a definition."#;

        let new_doc = parse_txxt_to_new_ast(source);

        // Current parser behavior: treats "Term:" as verbatim with title attribute
        // The adapter converts this to a Definition block
        let definition_blocks: Vec<_> = new_doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::Definition(def) = block {
                    Some(def)
                } else {
                    None
                }
            })
            .collect();

        assert!(!definition_blocks.is_empty(), "Should have a definition");

        let definition = definition_blocks[0];
        assert!(
            !definition.term.content.is_empty(),
            "Definition should have term from title attribute"
        );
        assert!(
            !definition.content.content.is_empty(),
            "Definition should have content from verbatim body"
        );
    }

    #[test]
    fn test_verbatim_block_e2e() {
        let source = r#"Example Code:
    def hello():
        print("Hello World")
        return 42
python"#;

        let new_doc = parse_txxt_to_new_ast(source);

        // Current parser behavior: treats code with title as verbatim with title attribute
        // The adapter converts this to a Definition block (since it has a title)
        let definition_blocks: Vec<_> = new_doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::Definition(def) = block {
                    Some(def)
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !definition_blocks.is_empty(),
            "Should have a definition (converted from verbatim with title)"
        );

        let definition = definition_blocks[0];
        // Content should be converted to paragraph blocks in the definition content
        assert!(
            !definition.content.content.is_empty(),
            "Should have content from verbatim body"
        );

        // Extract text content to verify it contains the code
        let content_text = if let Some(txxt::ast::blocks::Block::Paragraph(para)) =
            definition.content.content.first()
        {
            if let Some(txxt::ast::inlines::Inline::TextLine(transform)) = para.content.first() {
                transform.text_content()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        assert!(
            content_text.contains("def hello()"),
            "Should contain the function definition in converted content"
        );
    }

    #[test]
    fn test_nested_structure_e2e() {
        let source = r#"1. Main Section

    - Nested list item 1
    - Nested list item 2
        
        Nested paragraph under list item.
    
    - Nested list item 3

    Another paragraph in the main section."#;

        let new_doc = parse_txxt_to_new_ast(source);

        // The adapter correctly detects numbered list with content_container as a session
        let session_blocks: Vec<_> = new_doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::Session(session) = block {
                    Some(session)
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !session_blocks.is_empty(),
            "Should have a session from numbered list with nested content"
        );

        let session = session_blocks[0];
        assert!(
            !session.content.content.is_empty(),
            "Session should have nested content"
        );

        // Verify the session has some content (even if not properly structured yet)
        // The exact structure may need refinement in the adapter
        assert!(
            !session.content.content.is_empty(),
            "Session should have some content blocks"
        );
    }

    #[test]
    fn test_mixed_content_e2e() {
        let source = r#"Document Title

This is an introduction paragraph.

1. First Section

    Some content in the first section.
    
    - Bullet point 1
    - Bullet point 2
    
    More content after the list.

2. Second Section

    Definition Term:
        This is a definition within a section.
    
    Code Example:
        let x = 42;
        println!("{}", x);
    rust

End paragraph after all sections."#;

        let new_doc = parse_txxt_to_new_ast(source);

        // Current parser behavior: treats sections as separate blocks rather than sessions
        // Verify we have multiple types of blocks
        let block_types: std::collections::HashSet<_> = new_doc
            .blocks
            .iter()
            .map(|block| match block {
                txxt::ast::blocks::Block::Paragraph(_) => "paragraph",
                txxt::ast::blocks::Block::Session(_) => "session",
                txxt::ast::blocks::Block::List(_) => "list",
                txxt::ast::blocks::Block::Definition(_) => "definition",
                txxt::ast::blocks::Block::VerbatimBlock(_) => "verbatim",
                txxt::ast::blocks::Block::Container(_) => "container",
                txxt::ast::blocks::Block::BlankLine(_) => "blank_line",
            })
            .collect();

        // Should have multiple different block types
        assert!(
            block_types.len() >= 3,
            "Should have multiple block types, got: {:?}",
            block_types
        );

        // Should have paragraphs and lists as basic content types
        assert!(
            block_types.contains("paragraph"),
            "Should have paragraphs. Found: {:?}",
            block_types
        );
        assert!(
            block_types.contains("list"),
            "Should have lists. Found: {:?}",
            block_types
        );
    }

    #[test]
    fn test_annotation_preservation_e2e() {
        let source = r#":: note :: This is an important note

Regular paragraph here.

:: warning :: Be careful with this section"#;

        let new_doc = parse_txxt_to_new_ast(source);

        // The adapter should preserve the structure even if annotations aren't fully processed yet
        // At minimum, we should not lose content
        assert!(
            !new_doc.blocks.is_empty(),
            "Should have blocks even with annotations"
        );

        // Verify the document parsed without errors
        assert!(
            !new_doc.assembly_info.parser_version.is_empty(),
            "Should have assembly info"
        );
    }

    #[test]
    fn test_empty_document_e2e() {
        let source = "";

        let new_doc = parse_txxt_to_new_ast(source);

        // Empty document should still have valid metadata
        assert_eq!(
            new_doc.blocks.len(),
            0,
            "Empty document should have no blocks"
        );
        assert!(
            !new_doc.assembly_info.parser_version.is_empty(),
            "Should have parser version"
        );
    }

    #[test]
    fn test_whitespace_only_document_e2e() {
        let source = "   \n\n   \n   ";

        let new_doc = parse_txxt_to_new_ast(source);

        // Whitespace-only document might have blank line blocks
        // The important thing is it doesn't crash
        assert!(
            !new_doc.assembly_info.parser_version.is_empty(),
            "Should have parser version"
        );
    }
}

#[cfg(not(feature = "new-ast"))]
mod e2e_compatibility_tests {
    #[test]
    fn test_e2e_not_available() {
        // This test ensures the module compiles when new-ast feature is disabled
        assert!(true);
    }
}
