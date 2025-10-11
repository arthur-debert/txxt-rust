use crate::ast::AstNode;

/// Logic for detecting and validating sessions using bottom-up parsing
pub struct SessionDetector;

impl SessionDetector {
    /// Determine if an element with given child content should be a session
    pub fn should_be_session(element: &AstNode, child_elements: &[AstNode]) -> bool {
        // Only paragraphs can become sessions
        if element.node_type != "paragraph" {
            return false;
        }

        // Sessions cannot be empty - child elements must contain non-empty content
        if child_elements.is_empty() {
            return false;
        }

        // Check if child elements contain any actual content (not just blank lines)
        let has_content = child_elements.iter().any(|child| {
            match child.node_type.as_str() {
                "blank_line" => false,
                _ => true, // Any non-blank element counts as content
            }
        });

        has_content
    }

    /// Check if an element with the given child block should be considered a session
    /// Sessions require: preceded by blank line, non-empty indented content
    #[allow(dead_code)]
    pub fn is_session_title(
        element: &AstNode,
        child_block: &crate::block_grouping::TokenBlock,
    ) -> bool {
        // Sessions can only be paragraphs (not annotations, definitions, etc.)
        if element.node_type != "paragraph" {
            return false;
        }

        // Check if child block is non-empty (sessions cannot be empty)
        if child_block.tokens.is_empty() && child_block.children.is_empty() {
            return false;
        }

        // For now, assume any paragraph with indented content could be a session
        // TODO: Add blank line checking once we have better context tracking
        true
    }
}
