#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::block_grouping::{build_block_tree, Block, BlockType};
    use crate::tokenizer::{Token, TokenType};

    fn create_token(
        token_type: TokenType,
        value: Option<&str>,
        line: usize,
        column: usize,
    ) -> Token {
        Token::new(token_type, value.map(|s| s.to_string()), line, column)
    }

    #[test]
    fn test_simple_paragraph() {
        let tokens = vec![
            create_token(TokenType::Text, Some("Hello world"), 1, 1),
            create_token(TokenType::Newline, Some(""), 1, 12),
            create_token(TokenType::Eof, Some(""), 1, 13),
        ];

        let tree = build_block_tree(tokens);

        if let Block::Node(node) = tree {
            match node.block_type {
                BlockType::Paragraph { tokens } => {
                    assert_eq!(tokens.len(), 2); // TEXT + NEWLINE
                    assert_eq!(tokens[0].token_type, TokenType::Text);
                }
                _ => panic!("Expected paragraph, got {:?}", node.block_type),
            }
            assert!(node.container.is_none());
        } else {
            panic!("Expected BlockNode");
        }
    }

    #[test]
    fn test_annotation() {
        let tokens = vec![
            create_token(TokenType::PragmaMarker, Some("::"), 1, 1),
            create_token(TokenType::Identifier, Some("title"), 1, 4),
            create_token(TokenType::PragmaMarker, Some("::"), 1, 10),
            create_token(TokenType::Text, Some("My Document"), 1, 13),
            create_token(TokenType::Newline, Some(""), 1, 24),
            create_token(TokenType::Eof, Some(""), 1, 25),
        ];

        let tree = build_block_tree(tokens);

        if let Block::Node(node) = tree {
            match node.block_type {
                BlockType::Annotation { label, .. } => {
                    assert_eq!(label, "title");
                }
                _ => panic!("Expected annotation, got {:?}", node.block_type),
            }
        } else {
            panic!("Expected BlockNode");
        }
    }

    #[test]
    fn test_definition() {
        let tokens = vec![
            create_token(TokenType::Text, Some("Parser"), 1, 1),
            create_token(TokenType::DefinitionMarker, Some("::"), 1, 8),
            create_token(TokenType::Newline, Some(""), 1, 11),
            create_token(TokenType::Eof, Some(""), 1, 12),
        ];

        let tree = build_block_tree(tokens);

        if let Block::Node(node) = tree {
            match node.block_type {
                BlockType::Definition { term_tokens, .. } => {
                    assert_eq!(term_tokens.len(), 1);
                    assert_eq!(term_tokens[0].token_type, TokenType::Text);
                }
                _ => panic!("Expected definition, got {:?}", node.block_type),
            }
        } else {
            panic!("Expected BlockNode");
        }
    }

    #[test]
    fn test_list_item() {
        let tokens = vec![
            create_token(TokenType::Dash, Some("- "), 1, 1),
            create_token(TokenType::Text, Some("List item"), 1, 3),
            create_token(TokenType::Newline, Some(""), 1, 12),
            create_token(TokenType::Eof, Some(""), 1, 13),
        ];

        let tree = build_block_tree(tokens);

        if let Block::Node(node) = tree {
            match node.block_type {
                BlockType::ListItem {
                    marker_token,
                    inline_tokens,
                } => {
                    assert_eq!(marker_token.token_type, TokenType::Dash);
                    assert_eq!(inline_tokens.len(), 2); // TEXT + NEWLINE
                }
                _ => panic!("Expected list item, got {:?}", node.block_type),
            }
        } else {
            panic!("Expected BlockNode");
        }
    }

    #[test]
    fn test_verbatim_block() {
        let tokens = vec![
            create_token(TokenType::Text, Some("Code"), 1, 1),
            create_token(TokenType::VerbatimStart, Some(":"), 1, 5),
            create_token(TokenType::Newline, Some(""), 1, 6),
            create_token(TokenType::Eof, Some(""), 1, 7),
        ];

        let tree = build_block_tree(tokens);

        if let Block::Node(node) = tree {
            match node.block_type {
                BlockType::Verbatim { tokens } => {
                    assert!(tokens
                        .iter()
                        .any(|t| t.token_type == TokenType::VerbatimStart));
                }
                _ => panic!("Expected verbatim block, got {:?}", node.block_type),
            }
        } else {
            panic!("Expected BlockNode");
        }
    }

    #[test]
    fn test_session_with_children() {
        let tokens = vec![
            create_token(TokenType::Text, Some("Section Title"), 1, 1),
            create_token(TokenType::Newline, Some(""), 1, 14),
            create_token(TokenType::BlankLine, Some("\n"), 2, 1),
            create_token(TokenType::Indent, Some(""), 3, 1),
            create_token(TokenType::Text, Some("Child paragraph"), 3, 5),
            create_token(TokenType::Newline, Some(""), 3, 20),
            create_token(TokenType::Dedent, Some(""), 4, 1),
            create_token(TokenType::Eof, Some(""), 4, 1),
        ];

        let tree = build_block_tree(tokens);

        // The tree will be a root block containing a session child
        if let Block::Node(node) = tree {
            match node.block_type {
                BlockType::Root => {
                    assert!(node.container.is_some());

                    if let Some(container) = &node.container {
                        match container.as_ref() {
                            crate::block_grouping::blocks::Container::Session(session) => {
                                assert!(!session.children.is_empty());
                                // Should contain session children and blank line
                                assert!(session.children.len() >= 2);
                            }
                            _ => panic!("Expected session container"),
                        }
                    }
                }
                _ => panic!("Expected root, got {:?}", node.block_type),
            }
        } else {
            panic!("Expected BlockNode");
        }
    }

    #[test]
    fn test_blank_line_splitting() {
        let tokens = vec![
            create_token(TokenType::Text, Some("First para"), 1, 1),
            create_token(TokenType::Newline, Some(""), 1, 11),
            create_token(TokenType::BlankLine, Some("\n"), 2, 1),
            create_token(TokenType::Text, Some("Second para"), 3, 1),
            create_token(TokenType::Newline, Some(""), 3, 12),
            create_token(TokenType::Eof, Some(""), 3, 13),
        ];

        let tree = build_block_tree(tokens);

        // Should create a root with multiple children due to blank line splitting
        if let Block::Node(node) = tree {
            if let Some(container) = &node.container {
                match container.as_ref() {
                    crate::block_grouping::blocks::Container::Session(session) => {
                        // Should have 3 children: paragraph, blank line, paragraph
                        assert_eq!(session.children.len(), 3);
                    }
                    _ => panic!("Expected session container"),
                }
            }
        }
    }

    #[test]
    fn test_content_container_restrictions() {
        use crate::block_grouping::blocks::{BlockNode, ContentContainer};

        let mut content = ContentContainer::new(1);

        // Should allow paragraph
        let paragraph = Block::Node(BlockNode::new(BlockType::Paragraph { tokens: vec![] }));
        assert!(content.add_child(paragraph).is_ok());

        // Should reject session
        let session = Block::Node(BlockNode::new(BlockType::Session {
            title_tokens: vec![],
        }));
        assert!(content.add_child(session).is_err());
    }

    #[test]
    fn test_session_container_allows_all() {
        use crate::block_grouping::blocks::{BlockNode, SessionContainer};

        let mut session_container = SessionContainer::new(1);

        // Should allow paragraph
        let paragraph = Block::Node(BlockNode::new(BlockType::Paragraph { tokens: vec![] }));
        session_container.add_child(paragraph);

        // Should allow session
        let session = Block::Node(BlockNode::new(BlockType::Session {
            title_tokens: vec![],
        }));
        session_container.add_child(session);

        assert_eq!(session_container.children.len(), 2);
    }

    #[test]
    fn test_integration_with_tokenizer() {
        let text = ":: title :: My Document\n\nThis is a paragraph.";
        let tokens = crate::tokenizer::tokenize(text);
        let tree = build_block_tree(tokens);

        // Should create a root with annotation and paragraph
        if let Block::Node(node) = tree {
            if let Some(container) = &node.container {
                match container.as_ref() {
                    crate::block_grouping::blocks::Container::Session(session) => {
                        assert!(session.children.len() >= 2); // At least annotation and paragraph
                    }
                    _ => panic!("Expected session container"),
                }
            }
        }
    }
}
