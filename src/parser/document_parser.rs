use crate::ast::{AstNode, Document};
use crate::block_grouping::TokenBlock;
use crate::tokenizer::Token;

use super::container_association::ContainerAssociator;
use super::element_parsers::ElementParsers;
use super::list_processing::ListProcessor;
use super::token_analysis::TokenAnalyzer;

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

        // Parse the root block as a document (which is a SessionContainer)
        let mut doc_root = AstNode::new("document".to_string());

        // Use bottom-up parsing for proper session detection
        let root_elements = self.parse_block_bottom_up(block, true);

        for element in root_elements {
            doc_root.add_child(element);
        }

        document.root = doc_root;
        document
    }

    /// Parse the contents of a block into individual elements
    #[allow(dead_code)]
    fn parse_block_contents(&self, block: &TokenBlock) -> Vec<AstNode> {
        let mut elements = Vec::new();

        // First try to parse the block's own tokens as elements
        if !block.tokens.is_empty() {
            let element_groups = TokenAnalyzer::split_tokens_into_elements(&block.tokens);
            for element_tokens in element_groups {
                let token_refs: Vec<&Token> = element_tokens.to_vec();
                if let Some(element) = ElementParsers::parse_element_refs(&token_refs) {
                    elements.push(element);
                }
            }
        }

        // Then add child blocks
        for child_block in &block.children {
            let child_ast = self.parse_block(child_block);
            elements.push(child_ast);
        }

        // Group consecutive list items into lists
        ListProcessor::group_list_items(elements)
    }

    /// Parse a single TokenBlock into an AstNode
    #[allow(clippy::only_used_in_recursion)]
    #[allow(dead_code)]
    fn parse_block(&self, block: &TokenBlock) -> AstNode {
        // First try to parse the tokens as a specific element
        if let Some(element) = ElementParsers::parse_element(&block.tokens) {
            // If this block has children, we need to handle them based on the element type
            if !block.children.is_empty() {
                let mut elements = vec![element];
                // Use SessionContainer context for recursive block parsing (allows sessions)
                ContainerAssociator::connect_child_blocks_to_elements(
                    &mut elements,
                    &block.children,
                    true,
                    |child_block| self.parse_block_contents(child_block),
                    |child_block| self.parse_block_contents_as_session(child_block),
                );
                elements.into_iter().next().unwrap()
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

    /// Parse block contents as a session container (can contain sessions)
    #[allow(dead_code)]
    fn parse_block_contents_as_session(&self, block: &TokenBlock) -> Vec<AstNode> {
        let mut elements = Vec::new();

        // First try to parse the block's own tokens as elements
        if !block.tokens.is_empty() {
            let element_groups = TokenAnalyzer::split_tokens_into_elements(&block.tokens);
            for element_tokens in element_groups {
                let token_refs: Vec<&Token> = element_tokens.to_vec();
                if let Some(element) = ElementParsers::parse_element_refs(&token_refs) {
                    elements.push(element);
                }
            }
        }

        // Then process child blocks and check for sessions
        ContainerAssociator::connect_child_blocks_to_elements(
            &mut elements,
            &block.children,
            true,
            |child_block| self.parse_block_contents(child_block),
            |child_block| self.parse_block_contents_as_session(child_block),
        );

        // Group consecutive list items into lists
        ListProcessor::group_list_items(elements)
    }

    /// Parse a block using bottom-up approach for proper session detection
    /// is_session_context: true if this context can contain sessions
    #[allow(clippy::only_used_in_recursion)]
    fn parse_block_bottom_up(&self, block: &TokenBlock, is_session_context: bool) -> Vec<AstNode> {
        let mut elements = Vec::new();

        // First, parse any tokens in this block as individual elements
        if !block.tokens.is_empty() {
            let element_groups = TokenAnalyzer::split_tokens_into_elements(&block.tokens);
            for element_tokens in element_groups {
                let token_refs: Vec<&Token> = element_tokens.to_vec();
                if let Some(element) = ElementParsers::parse_element_refs(&token_refs) {
                    elements.push(element);
                }
            }
        }

        // Parse all child blocks recursively first (bottom-up)
        let mut parsed_child_blocks = Vec::new();
        for child_block in &block.children {
            // We need to determine the session context for each child block
            // This will be refined when we associate with parent elements
            let child_elements = self.parse_block_bottom_up(child_block, is_session_context);
            parsed_child_blocks.push((child_block, child_elements));
        }

        // If this block has no tokens but has children, just return the flattened children
        if block.tokens.is_empty() && !block.children.is_empty() {
            for (_, child_elements) in parsed_child_blocks {
                elements.extend(child_elements);
            }
            return ListProcessor::group_list_items(elements);
        }

        // Now try to associate child blocks with parent elements
        // and determine if any should be sessions
        if is_session_context {
            ContainerAssociator::associate_children_and_detect_sessions(
                &mut elements,
                &parsed_child_blocks,
                |child_block, is_session_context| {
                    self.parse_block_bottom_up(child_block, is_session_context)
                },
            );
        } else {
            ContainerAssociator::associate_children_as_content_containers(
                &mut elements,
                &parsed_child_blocks,
            );
        }

        // Group consecutive list items into lists
        ListProcessor::group_list_items(elements)
    }
}

/// Main entry point for parsing
pub fn parse_document(source: String, block: &TokenBlock) -> Document {
    let parser = DocumentParser::new(source);
    parser.parse(block)
}
