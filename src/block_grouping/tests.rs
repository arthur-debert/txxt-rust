#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::block_grouping::build_block_tree;
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

        // Should have the text and newline tokens
        assert_eq!(tree.tokens.len(), 2); // TEXT + NEWLINE (EOF is filtered)
        assert_eq!(tree.tokens[0].token_type, TokenType::Text);
        assert!(tree.children.is_empty()); // No children for simple paragraph
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
        assert!(!tree.children.is_empty());
        // Root should have at least 2 children: first para + blank line + second para
        assert!(tree.children.len() >= 2);
    }

    #[test]
    fn test_indented_content() {
        let tokens = vec![
            create_token(TokenType::Text, Some("Main line"), 1, 1),
            create_token(TokenType::Newline, Some(""), 1, 10),
            create_token(TokenType::Indent, Some(""), 2, 1),
            create_token(TokenType::Text, Some("Indented line"), 2, 5),
            create_token(TokenType::Newline, Some(""), 2, 18),
            create_token(TokenType::Dedent, Some(""), 3, 1),
            create_token(TokenType::Eof, Some(""), 3, 1),
        ];

        let tree = build_block_tree(tokens);

        // Should have children representing the indented content
        assert!(!tree.children.is_empty());

        // The main line should be in the parent
        assert!(tree.tokens.iter().any(|t| t.token_type == TokenType::Text));

        // There should be child blocks for the indented content
        assert!(tree.children.iter().any(|child| child
            .tokens
            .iter()
            .any(|t| t.token_type == TokenType::Text
                && t.value.as_ref().is_some_and(|v| v.contains("Indented")))));
    }

    #[test]
    fn test_integration_with_tokenizer() {
        let text = "Main paragraph\n\n    Indented content";
        let tokens = crate::tokenizer::tokenize(text);
        let tree = build_block_tree(tokens);

        // Should create a tree structure
        assert!(!tree.tokens.is_empty() || !tree.children.is_empty());
    }
}
