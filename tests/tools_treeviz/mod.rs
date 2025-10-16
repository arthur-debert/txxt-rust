//! Tests for the TXXT Tree Visualization System
//!
//! These tests validate the core functionality of the treeviz module,
//! including data structure creation, icon mapping, content extraction,
//! and tree rendering.

use txxt::ast::elements::components::parameters::Parameters;
use txxt::ast::elements::core::ElementNode;
use txxt::ast::elements::session::session_container::SessionContainer;
use txxt::ast::tokens::TokenSequence;
use txxt::tools::treeviz::*;

#[cfg(test)]
mod tests {
    use super::*;
    use txxt::tools::treeviz::{
        converter::create_demo_notation_data,
        icons::{extract_content_from_node, get_node_type_name, ContentExtractor},
        renderer::{notation_data_to_json, render_with_options, RenderOptions, TreeChars},
    };

    #[test]
    fn test_tree_node_creation() {
        let mut node = TreeNode::new(
            "§".to_string(),
            "Test Session".to_string(),
            "SessionBlock".to_string(),
        );

        assert_eq!(node.icon, "§");
        assert_eq!(node.content, "Test Session");
        assert_eq!(node.node_type, "SessionBlock");
        assert!(node.children.is_empty());
        assert!(node.metadata.is_empty());

        // Test adding children
        let child = TreeNode::new(
            "¶".to_string(),
            "Test Paragraph".to_string(),
            "ParagraphBlock".to_string(),
        );
        node.add_child(child);

        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].icon, "¶");

        // Test metadata
        node.set_metadata("test_key".to_string(), "test_value".to_string());
        assert_eq!(
            node.metadata.get("test_key"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_notation_data_creation() {
        let root = TreeNode::new(
            "⧉".to_string(),
            "Document".to_string(),
            "Document".to_string(),
        );
        let config = DEFAULT_ICON_CONFIG.clone();
        let data = NotationData::new(root, config.clone());

        assert_eq!(data.root.icon, "⧉");
        assert_eq!(data.config.type_icons, config.type_icons);
    }

    #[test]
    fn test_default_icon_config() {
        let config = &*DEFAULT_ICON_CONFIG;

        // Test document structure icons
        assert_eq!(config.get_icon("Document"), "⧉");
        assert_eq!(config.get_icon("SessionBlock"), "§");
        assert_eq!(config.get_icon("SessionContainer"), "Ψ");
        assert_eq!(config.get_icon("SessionTitle"), "⊤");

        // Test block element icons
        assert_eq!(config.get_icon("ParagraphBlock"), "¶");
        assert_eq!(config.get_icon("ListBlock"), "☰");
        assert_eq!(config.get_icon("ListItem"), "•");
        assert_eq!(config.get_icon("VerbatimBlock"), "𝒱");
        assert_eq!(config.get_icon("DefinitionBlock"), "≔");
        assert_eq!(config.get_icon("ContentContainer"), "➔");

        // Test inline element icons
        assert_eq!(config.get_icon("TextSpan"), "◦");
        assert_eq!(config.get_icon("TextLine"), "↵");
        assert_eq!(config.get_icon("ItalicSpan"), "𝐼");
        assert_eq!(config.get_icon("BoldSpan"), "𝐁");
        assert_eq!(config.get_icon("CodeSpan"), "ƒ");
        assert_eq!(config.get_icon("MathSpan"), "√");

        // Test reference icons
        assert_eq!(config.get_icon("ReferenceSpan"), "⊕");
        assert_eq!(config.get_icon("CitationSpan"), "†");
        assert_eq!(config.get_icon("PageReferenceSpan"), "◫");
        assert_eq!(config.get_icon("FootnoteReferenceSpan"), "³");
        assert_eq!(config.get_icon("SessionReferenceSpan"), "#");

        // Test metadata icons
        assert_eq!(config.get_icon("Label"), "◔");
        assert_eq!(config.get_icon("ParameterKey"), "✗");
        assert_eq!(config.get_icon("ParameterValue"), "$");
        assert_eq!(config.get_icon("AnnotationBlock"), "\"");

        // Test fallback
        assert_eq!(config.get_icon("UnknownType"), "◦");
    }

    #[test]
    fn test_content_extractor() {
        let extractor = ContentExtractor::simple("content", "children");
        assert_eq!(extractor.content_property, "content");
        assert_eq!(extractor.children_property, "children");
        assert!(extractor.format_template.is_none());

        let extractor_with_format = ContentExtractor::with_format("title", "items", "Section: {}");
        assert_eq!(extractor_with_format.content_property, "title");
        assert_eq!(extractor_with_format.children_property, "items");
        assert_eq!(
            extractor_with_format.format_template,
            Some("Section: {}".to_string())
        );
    }

    #[test]
    fn test_get_node_type_name() {
        // Create a simple ElementNode for testing
        let session_container = ElementNode::SessionContainer(SessionContainer::new(
            vec![],
            vec![],
            Parameters::default(),
            TokenSequence::new(),
        ));

        assert_eq!(get_node_type_name(&session_container), "SessionContainer");
    }

    #[test]
    fn test_extract_content_from_node() {
        let config = &*DEFAULT_ICON_CONFIG;
        let session_container = ElementNode::SessionContainer(SessionContainer::new(
            vec![],
            vec![],
            Parameters::default(),
            TokenSequence::new(),
        ));

        let content = extract_content_from_node(&session_container, config);
        // The function returns the type name when no specific extractor is found
        assert_eq!(content, "SessionContainer");
    }

    #[test]
    fn test_demo_notation_data() {
        let demo = create_demo_notation_data();

        // Verify root structure
        assert_eq!(demo.root.icon, "⧉");
        assert_eq!(demo.root.content, "Sample Document");
        assert_eq!(demo.root.node_type, "Document");

        // Verify it has children
        assert!(!demo.root.children.is_empty());

        // Check first child (session)
        let session = &demo.root.children[0];
        assert_eq!(session.icon, "§");
        assert_eq!(session.content, "1. Introduction");
        assert_eq!(session.node_type, "SessionBlock");
    }

    #[test]
    fn test_basic_tree_rendering() {
        let demo = create_demo_notation_data();
        let config = &demo.config;

        let result = notation_data_to_string(&demo, config);
        assert!(result.is_ok());

        let output = result.unwrap();

        // Check for proper tree structure
        assert!(output.contains("├─"));
        assert!(output.contains("└─"));
        assert!(output.contains("│"));

        // Check for correct icons
        assert!(output.contains("⧉ Sample Document"));
        assert!(output.contains("§ 1. Introduction"));
        assert!(output.contains("¶ This is a sample paragraph"));
        assert!(output.contains("☰ list (3 items)"));
        assert!(output.contains("• First item"));
        assert!(output.contains("𝒱 verbatim: code.example"));
    }

    #[test]
    fn test_json_serialization() {
        let demo = create_demo_notation_data();

        let json_result = notation_data_to_json(&demo);
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("\"root\""));
        assert!(json.contains("\"config\""));
        assert!(json.contains("\"icon\""));
        assert!(json.contains("\"content\""));
        assert!(json.contains("\"node_type\""));
        assert!(json.contains("\"children\""));

        // Test that we can deserialize it back
        let deserialized: NotationData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.root.icon, "⧉");
        assert_eq!(deserialized.root.content, "Sample Document");
    }

    #[test]
    fn test_render_options() {
        let demo = create_demo_notation_data();

        // Test ASCII rendering
        let ascii_options = RenderOptions {
            tree_chars: TreeChars::ascii(),
            ..Default::default()
        };

        let ascii_result = render_with_options(&demo, &ascii_options);
        assert!(ascii_result.is_ok());

        let ascii_output = ascii_result.unwrap();
        assert!(ascii_output.contains("|-"));
        assert!(ascii_output.contains("`-"));
        assert!(ascii_output.contains("|"));

        // Test with debug info
        let debug_options = RenderOptions {
            include_debug: true,
            ..Default::default()
        };

        let debug_result = render_with_options(&demo, &debug_options);
        assert!(debug_result.is_ok());

        let debug_output = debug_result.unwrap();
        assert!(debug_output.contains("[Document]"));
        assert!(debug_output.contains("[SessionBlock]"));
    }

    #[test]
    fn test_convenience_function() {
        // Test the one-step conversion function
        let session_container = ElementNode::SessionContainer(SessionContainer::new(
            vec![],
            vec![],
            Parameters::default(),
            TokenSequence::new(),
        ));

        let result = ast_to_tree_notation(&session_container);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("├─") || output.contains("└─"));
        assert!(output.contains("Ψ"));
    }

    #[test]
    fn test_synthetic_basic_ast_nodes() {
        // Test a few basic ElementNode variants that are simpler to construct
        let test_cases = vec![
            // Basic container elements that work
            (
                ElementNode::SessionContainer(SessionContainer::new(
                    vec![],
                    vec![],
                    Parameters::default(),
                    TokenSequence::new(),
                )),
                "SessionContainer",
                "Ψ",
            ),
            // Simple elements
            (
                ElementNode::BlankLine(txxt::ast::elements::core::BlankLine {
                    tokens: TokenSequence::new(),
                }),
                "BlankLine",
                "◦",
            ), // Uses fallback icon
        ];

        for (node, expected_type, expected_icon) in test_cases {
            // Test node type detection
            assert_eq!(get_node_type_name(&node), expected_type);

            // Test icon mapping
            let config = &*DEFAULT_ICON_CONFIG;
            assert_eq!(config.get_icon(expected_type), expected_icon);

            // Test conversion works without errors
            let result = ast_to_tree_notation(&node);
            assert!(
                result.is_ok(),
                "Failed to convert {} to tree notation",
                expected_type
            );

            let output = result.unwrap();
            assert!(
                output.contains(expected_icon),
                "Output for {} should contain icon {}",
                expected_type,
                expected_icon
            );
        }
    }

    #[test]
    fn test_basic_inline_elements() {
        // Test simple inline elements that can be easily created
        use txxt::ast::elements::inlines::TextSpan;

        let simple_test_cases = vec![
            (
                ElementNode::TextSpan(TextSpan::simple("sample text")),
                "TextSpan",
                "◦",
            ),
            (
                ElementNode::BlankLine(txxt::ast::elements::core::BlankLine {
                    tokens: TokenSequence::new(),
                }),
                "BlankLine",
                "◦",
            ), // No specific icon, uses fallback
        ];

        for (node, expected_type, _expected_icon) in simple_test_cases {
            // Test node type detection
            assert_eq!(get_node_type_name(&node), expected_type);

            // Test conversion works
            let result = ast_to_tree_notation(&node);
            assert!(
                result.is_ok(),
                "Failed to convert {} to tree notation",
                expected_type
            );
        }
    }

    #[test]
    fn test_all_node_type_names() {
        // Test that all ElementNode variants have correct type names
        // This uses the existing functions without complex constructors

        // Test a simple case we know works
        let session_container = ElementNode::SessionContainer(SessionContainer::new(
            vec![],
            vec![],
            Parameters::default(),
            TokenSequence::new(),
        ));
        assert_eq!(get_node_type_name(&session_container), "SessionContainer");

        let blank_line = ElementNode::BlankLine(txxt::ast::elements::core::BlankLine {
            tokens: TokenSequence::new(),
        });
        assert_eq!(get_node_type_name(&blank_line), "BlankLine");

        let text_span =
            ElementNode::TextSpan(txxt::ast::elements::inlines::TextSpan::simple("test"));
        assert_eq!(get_node_type_name(&text_span), "TextSpan");
    }

    #[test]
    fn test_complex_nested_synthetic_tree() {
        // Create a complex synthetic tree with multiple levels of nesting using TreeNode directly
        // since ElementNode constructors are complex

        // Build a complex nested structure manually using TreeNode
        let mut root = TreeNode::new(
            "⧉".to_string(),
            "Complex Document".to_string(),
            "Document".to_string(),
        );

        // Add multiple sessions
        for i in 1..=3 {
            let mut session = TreeNode::new(
                "§".to_string(),
                format!("{}. Section {}", i, i),
                "SessionBlock".to_string(),
            );

            // Add session title
            session.add_child(TreeNode::new(
                "⊤".to_string(),
                format!("Section {}", i),
                "SessionTitle".to_string(),
            ));

            // Add content container
            let mut container = TreeNode::new(
                "➔".to_string(),
                format!("{} items", 2 + i),
                "ContentContainer".to_string(),
            );

            // Add paragraph
            let mut paragraph = TreeNode::new(
                "¶".to_string(),
                format!("This is paragraph {} with multiple elements.", i),
                "ParagraphBlock".to_string(),
            );

            // Add text line with multiple spans
            let mut text_line = TreeNode::new(
                "↵".to_string(),
                format!("Line {} content", i),
                "TextLine".to_string(),
            );

            text_line.add_child(TreeNode::new(
                "◦".to_string(),
                "Regular text ".to_string(),
                "TextSpan".to_string(),
            ));

            text_line.add_child(TreeNode::new(
                "𝐁".to_string(),
                "bold".to_string(),
                "BoldSpan".to_string(),
            ));

            text_line.add_child(TreeNode::new(
                "◦".to_string(),
                " and ".to_string(),
                "TextSpan".to_string(),
            ));

            text_line.add_child(TreeNode::new(
                "𝐼".to_string(),
                "italic".to_string(),
                "ItalicSpan".to_string(),
            ));

            text_line.add_child(TreeNode::new(
                "◦".to_string(),
                " text.".to_string(),
                "TextSpan".to_string(),
            ));

            paragraph.add_child(text_line);
            container.add_child(paragraph);

            // Add a list with items
            let mut list = TreeNode::new(
                "☰".to_string(),
                format!("list ({} items)", i + 1),
                "ListBlock".to_string(),
            );

            for j in 1..=(i + 1) {
                list.add_child(TreeNode::new(
                    "•".to_string(),
                    format!("List item {}.{}", i, j),
                    "ListItem".to_string(),
                ));
            }

            container.add_child(list);

            // Add verbatim block for some sections
            if i % 2 == 0 {
                container.add_child(TreeNode::new(
                    "𝒱".to_string(),
                    format!("verbatim: example{}", i),
                    "VerbatimBlock".to_string(),
                ));
            }

            session.add_child(container);
            root.add_child(session);
        }

        // Test the complex tree
        let config = DEFAULT_ICON_CONFIG.clone();
        let notation_data = NotationData::new(root, config.clone());

        // Test rendering
        let result = notation_data_to_string(&notation_data, &config);
        assert!(result.is_ok());

        let output = result.unwrap();

        // Verify structure elements are present
        assert!(output.contains("⧉ Complex Document"));
        assert!(output.contains("§ 1. Section 1"));
        assert!(output.contains("§ 2. Section 2"));
        assert!(output.contains("§ 3. Section 3"));

        // Verify tree structure characters
        assert!(output.contains("├─"));
        assert!(output.contains("└─"));
        assert!(output.contains("│"));

        // Verify various element types
        assert!(output.contains("¶ This is paragraph"));
        assert!(output.contains("☰ list"));
        assert!(output.contains("• List item"));
        assert!(output.contains("◦ Regular text"));
        assert!(output.contains("𝐁 bold"));
        assert!(output.contains("𝐼 italic"));

        // Test JSON serialization of complex tree
        let json_result = notation_data_to_json(&notation_data);
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("Complex Document"));
        assert!(json.contains("Section 1"));
        assert!(json.contains("Section 2"));
        assert!(json.contains("Section 3"));

        // Verify we can round-trip through JSON
        let deserialized: NotationData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.root.content, "Complex Document");
        assert_eq!(deserialized.root.children.len(), 3); // 3 sections
    }

    #[test]
    fn test_error_handling() {
        // Test various error conditions and edge cases

        // Test empty tree
        let empty_root = TreeNode::new(
            "⧉".to_string(),
            "Empty Document".to_string(),
            "Document".to_string(),
        );
        let config = DEFAULT_ICON_CONFIG.clone();
        let empty_data = NotationData::new(empty_root, config.clone());

        let result = notation_data_to_string(&empty_data, &config);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("⧉ Empty Document"));
        assert!(!output.contains("├─")); // No children, so no branch characters

        // Test single node tree
        let single_node = ElementNode::BlankLine(txxt::ast::elements::core::BlankLine {
            tokens: TokenSequence::new(),
        });

        let result = ast_to_tree_notation(&single_node);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("└─")); // Should have at least one tree character
    }

    #[test]
    fn test_custom_icon_config() {
        // Test using a custom icon configuration
        let mut custom_config = IconConfig::new();

        // Add custom icons different from defaults
        custom_config.add_icon("TestNode".to_string(), "★".to_string());
        custom_config.add_icon("ParagraphBlock".to_string(), "●".to_string()); // Override default

        // Test custom icon retrieval
        assert_eq!(custom_config.get_icon("TestNode"), "★");
        assert_eq!(custom_config.get_icon("ParagraphBlock"), "●");
        assert_eq!(custom_config.get_icon("UnknownNode"), "◦"); // Fallback

        // Test with synthetic data using custom config
        let mut root = TreeNode::new(
            "★".to_string(),
            "Custom Document".to_string(),
            "TestNode".to_string(),
        );

        root.add_child(TreeNode::new(
            "●".to_string(),
            "Custom paragraph".to_string(),
            "ParagraphBlock".to_string(),
        ));

        let notation_data = NotationData::new(root, custom_config.clone());
        let result = notation_data_to_string(&notation_data, &custom_config);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("★ Custom Document"));
        assert!(output.contains("● Custom paragraph"));
    }

    #[test]
    fn test_content_extraction_with_format_templates() {
        // Test content extractors with format templates
        let mut config = IconConfig::new();

        config.add_extractor(
            "TestFormat".to_string(),
            ContentExtractor::with_format("title", "children", "Section: {}"),
        );

        let extractor = config.get_content_extractor("TestFormat").unwrap();
        assert_eq!(extractor.format_template, Some("Section: {}".to_string()));

        // Test formatting in actual extraction (this tests the fallback behavior)
        let node = ElementNode::BlankLine(txxt::ast::elements::core::BlankLine {
            tokens: TokenSequence::new(),
        });

        let content = extract_content_from_node(&node, &config);
        // Since BlankLine doesn't have a custom extractor, it should return type name
        assert_eq!(content, "BlankLine");
    }

    #[test]
    fn test_metadata_inclusion() {
        // Test metadata inclusion in rendering
        let mut root = TreeNode::new(
            "⧉".to_string(),
            "Document with metadata".to_string(),
            "Document".to_string(),
        );

        root.set_metadata("version".to_string(), "1.0".to_string());
        root.set_metadata("author".to_string(), "test".to_string());

        let config = IconConfig {
            include_metadata: true,
            ..Default::default()
        };

        let notation_data = NotationData::new(root, config.clone());

        let options = RenderOptions {
            include_metadata: true,
            ..Default::default()
        };

        let result = render_with_options(&notation_data, &options);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("version=1.0"));
        assert!(output.contains("author=test"));
    }

    #[test]
    fn test_full_pipeline_with_synthetic_ast() {
        // Test the complete pipeline from synthetic AST to visualization

        // Create a synthetic AST using simple constructors
        let text_span = ElementNode::TextSpan(txxt::ast::elements::inlines::TextSpan::simple(
            "Hello World",
        ));

        // Test the three-function API exactly as specified in the GitHub issue

        // 1. ast_to_notation_data
        let config = &*DEFAULT_ICON_CONFIG;
        let notation_data_result = ast_to_notation_data(&text_span, config);
        assert!(notation_data_result.is_ok());
        let notation_data = notation_data_result.unwrap();

        assert_eq!(notation_data.root.icon, "◦");
        assert_eq!(notation_data.root.node_type, "TextSpan");

        // 2. notation_data_to_string
        let tree_string_result = notation_data_to_string(&notation_data, config);
        assert!(tree_string_result.is_ok());
        let tree_string = tree_string_result.unwrap();

        assert!(tree_string.contains("◦"));
        assert!(tree_string.contains("└─")); // Should have tree structure

        // 3. ast_to_tree_notation (convenience function)
        let direct_result = ast_to_tree_notation(&text_span);
        assert!(direct_result.is_ok());
        let direct_output = direct_result.unwrap();

        // Should produce same result as the two-step process
        assert_eq!(tree_string, direct_output);

        // Test JSON serialization
        let json_result = notation_data_to_json(&notation_data);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();

        assert!(json.contains("\"icon\": \"◦\""));
        assert!(json.contains("\"node_type\": \"TextSpan\""));

        // Test round-trip through JSON
        let deserialized: NotationData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.root.icon, "◦");
        assert_eq!(deserialized.root.node_type, "TextSpan");
    }

    #[test]
    fn test_comprehensive_icon_coverage() {
        // Test that all icons from the GitHub issue specification are present
        let config = &*DEFAULT_ICON_CONFIG;

        // Document Structure icons
        assert_eq!(config.get_icon("Document"), "⧉");
        assert_eq!(config.get_icon("SessionBlock"), "§");
        assert_eq!(config.get_icon("SessionContainer"), "Ψ");
        assert_eq!(config.get_icon("SessionTitle"), "⊤");

        // Block Elements icons
        assert_eq!(config.get_icon("ParagraphBlock"), "¶");
        assert_eq!(config.get_icon("ListBlock"), "☰");
        assert_eq!(config.get_icon("ListItem"), "•");
        assert_eq!(config.get_icon("VerbatimBlock"), "𝒱");
        assert_eq!(config.get_icon("VerbatimLine"), "℣");
        assert_eq!(config.get_icon("DefinitionBlock"), "≔");
        assert_eq!(config.get_icon("ContentContainer"), "➔");

        // Inline Elements icons
        assert_eq!(config.get_icon("TextSpan"), "◦");
        assert_eq!(config.get_icon("TextLine"), "↵");
        assert_eq!(config.get_icon("ItalicSpan"), "𝐼");
        assert_eq!(config.get_icon("BoldSpan"), "𝐁");
        assert_eq!(config.get_icon("CodeSpan"), "ƒ");
        assert_eq!(config.get_icon("MathSpan"), "√");

        // References icons
        assert_eq!(config.get_icon("ReferenceSpan"), "⊕");
        assert_eq!(config.get_icon("FileReference"), "/");
        assert_eq!(config.get_icon("CitationSpan"), "†");
        assert_eq!(config.get_icon("AuthorReference"), "@");
        assert_eq!(config.get_icon("PageReferenceSpan"), "◫");
        assert_eq!(config.get_icon("ReferenceToCome"), "⋯");
        assert_eq!(config.get_icon("ReferenceUnknown"), "∅");
        assert_eq!(config.get_icon("FootnoteReferenceSpan"), "³");
        assert_eq!(config.get_icon("SessionReferenceSpan"), "#");

        // Metadata & Parameters icons
        assert_eq!(config.get_icon("Label"), "◔");
        assert_eq!(config.get_icon("ParameterKey"), "✗");
        assert_eq!(config.get_icon("ParameterValue"), "$");
        assert_eq!(config.get_icon("AnnotationBlock"), "\"");

        println!(
            "✅ All {} icons from GitHub issue #46 are correctly configured",
            config.type_icons.len()
        );
    }
}
