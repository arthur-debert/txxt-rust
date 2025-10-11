use crate::ast::{AstNode, Document};
use crate::block_grouping::TokenBlock;

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

        // For now, just create a dummy AST to verify the tooling works
        let mut root_node = AstNode::new("document".to_string());

        // Add a dummy child to show the structure works
        let mut dummy_child = AstNode::new("dummy_element".to_string());
        dummy_child.set_attribute("block_indent".to_string(), block.indent_level.to_string());

        if let (Some(start), Some(end)) = (block.start_line, block.end_line) {
            dummy_child.set_location(start, end);
        }

        // Add token count as content
        dummy_child.content = Some(format!(
            "tokens: {}, children: {}",
            block.tokens.len(),
            block.children.len()
        ));

        root_node.add_child(dummy_child);

        // Recursively add children
        for child_block in &block.children {
            let child_ast = self.parse_block(child_block);
            root_node.add_child(child_ast);
        }

        document.root = root_node;
        document
    }

    /// Parse a single TokenBlock into an AstNode (dummy implementation)
    #[allow(clippy::only_used_in_recursion)]
    fn parse_block(&self, block: &TokenBlock) -> AstNode {
        let mut node = AstNode::new("block_element".to_string());

        node.set_attribute("indent_level".to_string(), block.indent_level.to_string());
        node.set_attribute("token_count".to_string(), block.tokens.len().to_string());

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

/// Main entry point for parsing
pub fn parse_document(source: String, block: &TokenBlock) -> Document {
    let parser = DocumentParser::new(source);
    parser.parse(block)
}
