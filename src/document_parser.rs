use crate::ast::{AstNode, Document};
use crate::block_grouping::TokenBlock;
use crate::tokenizer::{Token, TokenType};

/// Main parser that converts TokenBlocks to AST
pub struct DocumentParser {
    source: String,
}

impl DocumentParser {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    /// Parse a TokenBlock tree into an AST Document
    pub fn parse(&self, block: &TokenBlock) -> Document {
        let mut document = Document::new(self.source.clone());

        // Parse the root block as a document
        let mut doc_root = AstNode::new("document".to_string());

        // If the root block has content, parse it as elements
        if !block.tokens.is_empty() {
            if let Some(element) = self.parse_element(&block.tokens) {
                doc_root.add_child(element);
            }
        }

        // Add child blocks as elements
        for child_block in &block.children {
            let child_ast = self.parse_block(child_block);
            doc_root.add_child(child_ast);
        }

        document.root = doc_root;
        document
    }

    /// Parse a single TokenBlock into an AstNode
    #[allow(clippy::only_used_in_recursion)]
    fn parse_block(&self, block: &TokenBlock) -> AstNode {
        // First try to parse the tokens as a specific element
        if let Some(element) = self.parse_element(&block.tokens) {
            // If this block has children, we need to handle them based on the element type
            if !block.children.is_empty() {
                match element.node_type.as_str() {
                    "definition" | "list_item" => {
                        // These elements can have content containers
                        let mut content_node = AstNode::new("content_container".to_string());
                        for child_block in &block.children {
                            let child_ast = self.parse_block(child_block);
                            content_node.add_child(child_ast);
                        }
                        let mut result = element;
                        result.add_child(content_node);
                        result
                    }
                    _ => {
                        // For other elements, just add children directly
                        let mut result = element;
                        for child_block in &block.children {
                            let child_ast = self.parse_block(child_block);
                            result.add_child(child_ast);
                        }
                        result
                    }
                }
            } else {
                element
            }
        } else {
            // If we can't parse as a specific element, create a generic block
            let mut node = AstNode::new("block".to_string());

            if let (Some(start), Some(end)) = (block.start_line, block.end_line) {
                node.set_location(start, end);
            }

            // Add children recursively
            for child_block in &block.children {
                let child_ast = self.parse_block(child_block);
                node.add_child(child_ast);
            }

            node
        }
    }

    /// Parse a list of tokens into a specific element type
    fn parse_element(&self, tokens: &[Token]) -> Option<AstNode> {
        if tokens.is_empty() {
            return None;
        }

        // Skip EOF tokens for analysis
        let tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.token_type != TokenType::Eof)
            .collect();
        if tokens.is_empty() {
            return None;
        }

        // Check for annotation pattern: :: label :: content
        if let Some(annotation) = self.parse_annotation(&tokens) {
            return Some(annotation);
        }

        // Check for definition pattern: term ::
        if let Some(definition) = self.parse_definition(&tokens) {
            return Some(definition);
        }

        // Check for verbatim block pattern
        if let Some(verbatim) = self.parse_verbatim(&tokens) {
            return Some(verbatim);
        }

        // Check for list item pattern: - item or 1. item
        if let Some(list_item) = self.parse_list_item(&tokens) {
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
        Some(self.parse_paragraph(&tokens))
    }

    /// Parse annotation: :: label :: content
    fn parse_annotation(&self, tokens: &[&Token]) -> Option<AstNode> {
        if tokens.len() < 3 {
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
                let label = self.extract_text(&label_tokens).trim().to_string();
                node.set_attribute("label".to_string(), label);

                // Extract content
                let content = self.extract_text(&content_tokens).trim().to_string();
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
    fn parse_definition(&self, tokens: &[&Token]) -> Option<AstNode> {
        // Look for :: at the end
        if let Some(def_pos) = tokens
            .iter()
            .position(|t| t.token_type == TokenType::DefinitionMarker)
        {
            let term_tokens = &tokens[..def_pos];

            let mut node = AstNode::new("definition".to_string());

            // Extract term with inline formatting
            let term = self.extract_text_with_inlines(term_tokens);
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
    fn parse_verbatim(&self, tokens: &[&Token]) -> Option<AstNode> {
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

            // Extract content
            let content = self.extract_verbatim_content(tokens);
            if !content.is_empty() {
                node.content = Some(content);
            }

            // Extract label if present
            if let Some(label) = self.extract_verbatim_label(tokens) {
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
    fn parse_list_item(&self, tokens: &[&Token]) -> Option<AstNode> {
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

            // Extract content (skip the marker)
            let content_tokens = &tokens[1..];
            let content = self
                .extract_text_with_inlines(content_tokens)
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

        None
    }

    /// Parse paragraph (default case)
    fn parse_paragraph(&self, tokens: &[&Token]) -> AstNode {
        let mut node = AstNode::new("paragraph".to_string());

        // Extract content with inline formatting
        let content = self.extract_text_with_inlines(tokens);
        if !content.trim().is_empty() {
            node.content = Some(content.trim().to_string());
        }

        // Set location
        if let (Some(first), Some(last)) = (tokens.first(), tokens.last()) {
            node.set_location(first.line, last.line);
        }

        node
    }

    /// Extract plain text from tokens
    fn extract_text(&self, tokens: &[&Token]) -> String {
        tokens
            .iter()
            .filter_map(|t| match t.token_type {
                TokenType::Text | TokenType::Identifier => t.value.as_ref(),
                _ => None,
            })
            .cloned()
            .collect::<Vec<String>>()
            .join("")
    }

    /// Extract text with inline formatting
    fn extract_text_with_inlines(&self, tokens: &[&Token]) -> String {
        let mut result = String::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = tokens[i];

            match token.token_type {
                TokenType::Text | TokenType::Identifier => {
                    if let Some(value) = &token.value {
                        result.push_str(value);
                    }
                }
                TokenType::EmphasisMarker
                | TokenType::StrongMarker
                | TokenType::CodeMarker
                | TokenType::MathMarker => {
                    // For now, just include the marker content
                    if let Some(value) = &token.value {
                        result.push_str(value);
                    }
                }
                TokenType::Newline => {
                    result.push(' ');
                }
                _ => {}
            }
            i += 1;
        }

        result
    }

    /// Extract verbatim content
    fn extract_verbatim_content(&self, tokens: &[&Token]) -> String {
        tokens
            .iter()
            .filter_map(|t| {
                if t.token_type == TokenType::VerbatimContent {
                    t.value.as_ref()
                } else {
                    None
                }
            })
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Extract verbatim label
    fn extract_verbatim_label(&self, tokens: &[&Token]) -> Option<String> {
        // Look for identifier between VerbatimEnd tokens
        let mut in_label = false;
        for token in tokens {
            if token.token_type == TokenType::VerbatimEnd {
                if let Some(value) = &token.value {
                    if value == "(" {
                        in_label = true;
                        continue;
                    } else if value == ")" {
                        break;
                    }
                }
            }
            if in_label && token.token_type == TokenType::Identifier {
                if let Some(label) = &token.value {
                    return Some(label.clone());
                }
            }
        }
        None
    }
}

/// Main entry point for parsing
pub fn parse_document(source: String, block: &TokenBlock) -> Document {
    let parser = DocumentParser::new(source);
    parser.parse(block)
}
