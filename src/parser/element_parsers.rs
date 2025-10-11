use super::text_extraction::TextExtractor;
use super::token_analysis::TokenAnalyzer;
use crate::ast::AstNode;
use crate::tokenizer::{Token, TokenType};

/// Individual element parsers for different TXXT elements
pub struct ElementParsers;

impl ElementParsers {
    /// Parse a list of token references into a specific element type
    pub fn parse_element_refs(tokens: &[&Token]) -> Option<AstNode> {
        Self::parse_element_internal(tokens)
    }

    /// Parse a list of tokens into a specific element type
    #[allow(dead_code)]
    pub fn parse_element(tokens: &[Token]) -> Option<AstNode> {
        let token_refs: Vec<&Token> = tokens.iter().collect();
        Self::parse_element_internal(&token_refs)
    }

    /// Internal implementation for parsing elements
    fn parse_element_internal(tokens: &[&Token]) -> Option<AstNode> {
        if tokens.is_empty() {
            return None;
        }

        // Skip EOF tokens for analysis
        let tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.token_type != TokenType::Eof)
            .cloned()
            .collect();
        if tokens.is_empty() {
            return None;
        }

        // Check for annotation pattern: :: label :: content
        if let Some(annotation) = Self::parse_annotation(&tokens) {
            return Some(annotation);
        }

        // Check for definition pattern: term ::
        if let Some(definition) = Self::parse_definition(&tokens) {
            return Some(definition);
        }

        // Check for verbatim block pattern
        if let Some(verbatim) = Self::parse_verbatim(&tokens) {
            return Some(verbatim);
        }

        // Check for list item pattern: - item or 1. item
        if let Some(list_item) = Self::parse_list_item(&tokens) {
            return Some(list_item);
        }

        // Check for blank line
        if tokens.len() == 1 && tokens[0].token_type == TokenType::BlankLine {
            let mut node = AstNode::new("blank_line".to_string());
            if let Some(start) = tokens.first().map(|t| t.line) {
                node.set_location(start, start);
            }
            return Some(node);
        }

        // Default to paragraph
        Some(Self::parse_paragraph(&tokens))
    }

    /// Parse annotation: :: label :: content
    fn parse_annotation(tokens: &[&Token]) -> Option<AstNode> {
        if tokens.len() < 2 {
            return None;
        }

        // Check for pattern: :: label ::
        if tokens[0].token_type == TokenType::PragmaMarker {
            let mut label_tokens = Vec::new();
            let mut content_tokens = Vec::new();
            let mut in_label = true;
            let mut pragma_count = 0;

            for token in tokens {
                if token.token_type == TokenType::PragmaMarker {
                    pragma_count += 1;
                    if pragma_count == 2 {
                        in_label = false;
                        continue;
                    }
                    if pragma_count > 2 {
                        break;
                    }
                } else if in_label {
                    label_tokens.push(*token);
                } else {
                    content_tokens.push(*token);
                }
            }

            if pragma_count >= 2 {
                let mut node = AstNode::new("annotation".to_string());

                // Extract label
                let label = TextExtractor::extract_text(&label_tokens)
                    .trim()
                    .to_string();
                node.set_attribute("label".to_string(), label);

                // Extract content
                let content = TextExtractor::extract_text(&content_tokens)
                    .trim()
                    .to_string();
                if !content.is_empty() {
                    node.content = Some(content);
                }

                // Set location
                if let (Some(first), Some(last)) = (tokens.first(), tokens.last()) {
                    node.set_location(first.line, last.line);
                }

                return Some(node);
            }
        }

        None
    }

    /// Parse definition: term ::
    fn parse_definition(tokens: &[&Token]) -> Option<AstNode> {
        // Look for :: at the end
        if let Some(def_pos) = tokens
            .iter()
            .position(|t| t.token_type == TokenType::DefinitionMarker)
        {
            let term_tokens = &tokens[..def_pos];

            let mut node = AstNode::new("definition".to_string());

            // Extract term with inline formatting
            let term = TextExtractor::extract_text_with_inlines(term_tokens);
            node.set_attribute("term".to_string(), term);

            // Set location
            if let (Some(first), Some(last)) = (tokens.first(), tokens.last()) {
                node.set_location(first.line, last.line);
            }

            return Some(node);
        }

        None
    }

    /// Parse verbatim block
    fn parse_verbatim(tokens: &[&Token]) -> Option<AstNode> {
        // Look for verbatim patterns
        let has_verbatim_start = tokens
            .iter()
            .any(|t| t.token_type == TokenType::VerbatimStart);
        let has_verbatim_content = tokens
            .iter()
            .any(|t| t.token_type == TokenType::VerbatimContent);
        let has_verbatim_end = tokens
            .iter()
            .any(|t| t.token_type == TokenType::VerbatimEnd);

        if has_verbatim_start || has_verbatim_content || has_verbatim_end {
            let mut node = AstNode::new("verbatim".to_string());

            // Extract title (text before VerbatimStart)
            if let Some(title) = TextExtractor::extract_verbatim_title(tokens) {
                node.set_attribute("title".to_string(), title);
            }

            // Extract content (preserve exactly as tokenized - do NOT parse as TXXT)
            let content = TextExtractor::extract_verbatim_content(tokens);
            if !content.is_empty() {
                node.content = Some(content);
            }

            // Extract label if present
            if let Some(label) = TextExtractor::extract_verbatim_label(tokens) {
                node.set_attribute("label".to_string(), label);
            }

            // Set location
            if let (Some(first), Some(last)) = (tokens.first(), tokens.last()) {
                node.set_location(first.line, last.line);
            }

            return Some(node);
        }

        None
    }

    /// Parse list item: - item or 1. item
    fn parse_list_item(tokens: &[&Token]) -> Option<AstNode> {
        if tokens.is_empty() {
            return None;
        }

        let first_token = tokens[0];
        if matches!(
            first_token.token_type,
            TokenType::Dash | TokenType::SequenceMarker
        ) {
            let mut node = AstNode::new("list_item".to_string());

            // Extract marker
            if let Some(marker_value) = &first_token.value {
                node.set_attribute("marker".to_string(), marker_value.clone());
            }

            // Extract content (skip the marker, but only until next list item or element boundary)
            let mut content_tokens = Vec::new();
            let mut i = 1; // Skip the marker
            while i < tokens.len() {
                let token = tokens[i];

                // Stop if we encounter another list marker or element starter
                if matches!(
                    token.token_type,
                    TokenType::Dash | TokenType::SequenceMarker
                ) || token.token_type == TokenType::PragmaMarker
                    || token.token_type == TokenType::VerbatimStart
                    || token.token_type == TokenType::BlankLine
                    || (token.token_type == TokenType::Text && {
                        let remaining_tokens: Vec<Token> =
                            tokens[i..].iter().map(|&t| t.clone()).collect();
                        TokenAnalyzer::looks_like_definition_start(&remaining_tokens)
                    })
                {
                    break;
                }

                content_tokens.push(token);
                i += 1;
            }

            let content = TextExtractor::extract_text_with_inlines(&content_tokens)
                .trim()
                .to_string();
            if !content.is_empty() {
                node.content = Some(content);
            }

            // Set location (from first token to last consumed token)
            let last_token_index = if content_tokens.is_empty() { 0 } else { i - 1 };
            if let (Some(first), Some(last)) = (tokens.first(), tokens.get(last_token_index)) {
                node.set_location(first.line, last.line);
            }

            return Some(node);
        }

        None
    }

    /// Parse paragraph (default case)
    fn parse_paragraph(tokens: &[&Token]) -> AstNode {
        let mut node = AstNode::new("paragraph".to_string());

        // Extract content with inline formatting
        let content = TextExtractor::extract_text_with_inlines(tokens);
        if !content.trim().is_empty() {
            node.content = Some(content.trim().to_string());
        }

        // Set location
        if let (Some(first), Some(last)) = (tokens.first(), tokens.last()) {
            node.set_location(first.line, last.line);
        }

        node
    }
}
