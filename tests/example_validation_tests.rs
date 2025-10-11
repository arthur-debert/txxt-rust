//! Validation tests using example TXXT documents
//!
//! These tests validate the parsing pipeline using real-world example
//! documents to ensure the AST conversion works correctly with complex
//! content structures.

#[cfg(feature = "new-ast")]
mod example_validation_tests {
    use std::fs;
    use txxt::adapters::convert_document;
    use txxt::ast_debug::{AstStatistics, AstTreeVisualizer, TreeConfig};
    use txxt::block_grouping::build_block_tree;
    use txxt::parser::parse_document;
    use txxt::tokenizer::tokenize;

    /// Helper to parse example files
    fn parse_example_file(filename: &str) -> txxt::ast::base::Document {
        let path = format!("examples/{}", filename);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Could not read example file: {}", path));

        // Parse through the full pipeline
        let tokens = tokenize(&source);
        let block_tree = build_block_tree(tokens);

        let old_doc = parse_document(path.clone(), &block_tree);

        convert_document(&old_doc)
    }

    #[test]
    fn test_simple_document_parsing() {
        let doc = parse_example_file("simple_document.txxt");

        // Basic validation - just ensure it parses successfully
        assert!(
            !doc.blocks.is_empty(),
            "Simple document should have content blocks"
        );

        // Verify assembly info is populated
        assert!(
            !doc.assembly_info.parser_version.is_empty(),
            "Should have parser version"
        );

        // Verify we have some content types
        let stats = AstStatistics::from_document(&doc);
        assert!(
            stats.paragraph_count > 0 || stats.list_count > 0,
            "Should have some content (paragraphs or lists)"
        );
    }

    #[test]
    fn test_complex_document_parsing() {
        let doc = parse_example_file("complex_document.txxt");

        // Complex document should have rich structure
        assert!(
            !doc.blocks.is_empty(),
            "Complex document should have content blocks"
        );

        // Collect statistics
        let stats = AstStatistics::from_document(&doc);

        // Complex document should have substantial content
        assert!(
            stats.total_characters > 100,
            "Complex document should have substantial content"
        );
        assert!(
            doc.blocks.len() > 3,
            "Complex document should have multiple blocks"
        );

        println!("Complex document statistics:\n{}", stats);
    }

    #[test]
    fn test_minimal_examples_parsing() {
        let doc = parse_example_file("minimal_examples.txxt");

        // Even minimal examples should parse successfully
        assert!(
            !doc.blocks.is_empty(),
            "Minimal examples should have some content"
        );

        // Should handle edge cases without crashing
        let stats = AstStatistics::from_document(&doc);
        println!("Minimal examples statistics:\n{}", stats);

        // Validate it has expected minimal content
        assert!(
            stats.paragraph_count >= 1,
            "Should have at least one paragraph"
        );
    }

    #[test]
    fn test_tree_visualization() {
        let doc = parse_example_file("simple_document.txxt");

        // Test basic visualization
        let visualizer = AstTreeVisualizer::new();
        let tree_output = visualizer.visualize_document(&doc);

        // Basic validation of tree output
        assert!(
            tree_output.contains("📄 Document"),
            "Should show document root"
        );
        assert!(
            tree_output.contains("📦 Blocks"),
            "Should show blocks section"
        );

        // Should contain various block type indicators
        assert!(
            tree_output.contains("📚 Session") || tree_output.contains("📝 Paragraph"),
            "Should show content blocks"
        );

        println!(
            "Tree visualization preview:\n{}",
            &tree_output[..500.min(tree_output.len())]
        );
    }

    #[test]
    fn test_compact_visualization() {
        let doc = parse_example_file("minimal_examples.txxt");

        // Test compact mode
        let config = TreeConfig {
            compact: true,
            show_tokens: false,
            show_annotations: false,
            show_parameters: false,
            max_depth: Some(3),
        };

        let visualizer = AstTreeVisualizer::with_config(config);
        let compact_output = visualizer.visualize_document(&doc);

        // Compact output should be shorter
        assert!(
            compact_output.len() < 1000,
            "Compact output should be concise"
        );
        assert!(
            compact_output.contains("📄 Document"),
            "Should still show document root"
        );

        println!("Compact visualization:\n{}", compact_output);
    }

    #[test]
    fn test_statistics_collection() {
        let doc = parse_example_file("complex_document.txxt");
        let stats = AstStatistics::from_document(&doc);

        // Validate statistics make sense
        assert!(
            stats.total_characters > 100,
            "Complex document should have substantial content"
        );
        assert!(stats.max_nesting_depth > 0, "Should have some nesting");

        // Test statistics display
        let stats_display = format!("{}", stats);
        assert!(
            stats_display.contains("📊 AST Statistics"),
            "Should format statistics nicely"
        );
        assert!(
            stats_display.contains("📝 Paragraphs"),
            "Should show paragraph count"
        );

        println!("Full statistics:\n{}", stats_display);
    }

    #[test]
    fn test_all_examples_parse_successfully() {
        let example_files = [
            "simple_document.txxt",
            "complex_document.txxt",
            "minimal_examples.txxt",
        ];

        for filename in &example_files {
            println!("Testing example file: {}", filename);

            // Should parse without panicking
            let doc = parse_example_file(filename);

            // Should have valid assembly info
            assert!(
                !doc.assembly_info.parser_version.is_empty(),
                "Document should have parser version in {}",
                filename
            );

            // Should be able to collect statistics
            let stats = AstStatistics::from_document(&doc);
            println!("  - Total characters: {}", stats.total_characters);
            println!("  - Max depth: {}", stats.max_nesting_depth);

            // Should be able to visualize
            let visualizer = AstTreeVisualizer::new();
            let tree = visualizer.visualize_document(&doc);
            assert!(
                !tree.is_empty(),
                "Should generate tree visualization for {}",
                filename
            );
        }
    }

    #[test]
    fn test_nested_structure_preservation() {
        let doc = parse_example_file("complex_document.txxt");

        // Find a session with nested content
        let sessions: Vec<_> = doc
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

        // Check if we have sessions or other structured content
        let has_structured_content = !sessions.is_empty() || doc.blocks.len() > 5;
        assert!(
            has_structured_content,
            "Should have sessions or substantial structured content"
        );

        // Check that sessions have nested content OR we have substantial block structure
        if !sessions.is_empty() {
            let has_nested_content = sessions
                .iter()
                .any(|session| !session.content.content.is_empty());
            assert!(
                has_nested_content,
                "Sessions should have nested content when present"
            );
        }
        // Otherwise just verify we have some structured content

        // Test that we have some reasonable structure
        let stats = AstStatistics::from_document(&doc);
        assert!(
            stats.max_nesting_depth >= 1,
            "Should have some nesting structure"
        );
    }

    #[test]
    fn test_content_preservation() {
        let doc = parse_example_file("simple_document.txxt");

        // Verify that content is preserved through the pipeline
        let visualizer = AstTreeVisualizer::new();

        // Check that we can extract meaningful text content
        let paragraphs: Vec<_> = doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::Paragraph(para) = block {
                    Some(para)
                } else {
                    None
                }
            })
            .collect();

        if !paragraphs.is_empty() {
            let first_para_text = visualizer.extract_text_from_inlines(&paragraphs[0].content);
            assert!(
                !first_para_text.trim().is_empty(),
                "Should preserve paragraph text content"
            );
            println!("Sample paragraph content: {:?}", first_para_text);
        }

        // Check verbatim content preservation
        let verbatim_blocks: Vec<_> = doc
            .blocks
            .iter()
            .filter_map(|block| {
                if let txxt::ast::blocks::Block::VerbatimBlock(verbatim) = block {
                    Some(verbatim)
                } else {
                    None
                }
            })
            .collect();

        if !verbatim_blocks.is_empty() {
            assert!(
                !verbatim_blocks[0].raw.trim().is_empty(),
                "Should preserve verbatim content exactly"
            );
            println!(
                "Sample verbatim content: {:?}",
                &verbatim_blocks[0].raw[..50.min(verbatim_blocks[0].raw.len())]
            );
        }
    }
}

#[cfg(not(feature = "new-ast"))]
mod example_compatibility_tests {
    #[test]
    fn test_examples_not_available() {
        // Ensure the module compiles when new-ast feature is disabled
        assert!(true);
    }
}
