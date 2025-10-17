//! Phase 2a: Block Parsing
//!
//! Converts token trees into typed AST nodes for block elements.
//! This is the first step of Phase 2 parsing, where we take the hierarchical
//! token structure from the lexer and create proper AST element nodes.
//!
//! # Block Types
//!
//! - Paragraphs
//! - Lists (numbered, bulleted, alphabetical)
//! - Definitions
//! - Annotations
//! - Verbatim blocks
//! - Sessions
//! - Containers
//!
//! # Input/Output
//!
//! - **Input**: `TokenTree` from lexer (Phase 1c)
//! - **Output**: AST tree of `ElementNode` variants

use crate::ast::ElementNode;
use crate::lexer::pipeline::TokenTree;
use crate::parser::elements::paragraph::paragraph::parse_paragraph;

/// Block parser for converting token trees to AST nodes
///
/// This parser takes the hierarchical token structure and creates
/// typed AST nodes for each block element type.
pub struct BlockParser;

impl Default for BlockParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockParser {
    /// Create a new block parser instance
    pub fn new() -> Self {
        Self
    }

    /// Parse token tree into AST block elements
    ///
    /// Takes a hierarchical token tree and converts it into a tree
    /// of typed AST element nodes. Each block type is handled by
    /// its specific parsing logic.
    #[allow(clippy::only_used_in_recursion)]
    pub fn parse_blocks(&self, token_tree: TokenTree) -> Result<Vec<ElementNode>, BlockParseError> {
        let mut elements = Vec::new();

        // For Phase 1 Simple Elements, we focus on parsing paragraphs
        // Other elements will be added in subsequent phases

        // Process tokens at the root level
        if !token_tree.tokens.is_empty() {
            // First, try to detect and parse sessions
            // Sessions require indented content (child token trees)
            if !token_tree.children.is_empty() {
                // We have indented content, so try to parse as sessions
                // Split the root tokens into session groups
                let session_groups = self.split_into_sessions(&token_tree.tokens);

                // Each session group should correspond to a child tree
                for (i, session_tokens) in session_groups.iter().enumerate() {
                    if i < token_tree.children.len() {
                        let child_tree = &token_tree.children[i];
                        let session =
                            self.parse_session_with_content(session_tokens, child_tree.clone())?;
                        elements.push(ElementNode::SessionBlock(session));
                    }
                }
            } else {
                // No indented content, parse as paragraphs
                let paragraph_groups = self.split_into_paragraphs(&token_tree.tokens);

                for paragraph_tokens in paragraph_groups {
                    if !paragraph_tokens.is_empty() {
                        // Try to parse each paragraph group
                        match parse_paragraph(&paragraph_tokens) {
                            Ok(paragraph) => {
                                elements.push(ElementNode::ParagraphBlock(paragraph));
                            }
                            Err(_) => {
                                // If it's not a paragraph, we'll handle other element types later
                                // For now, skip unrecognized tokens
                            }
                        }
                    }
                }
            }
        }

        // Note: Child token trees are now processed as part of session parsing above
        // Only process child trees recursively if we didn't handle them as sessions
        if token_tree.tokens.is_empty() || token_tree.children.is_empty() {
            for child_tree in &token_tree.children {
                let child_elements = self.parse_blocks(child_tree.clone())?;
                elements.extend(child_elements);
            }
        }

        Ok(elements)
    }

    /// Split tokens into paragraph groups based on BlankLine boundaries
    ///
    /// This function takes a sequence of tokens and splits them into groups
    /// where each group represents one paragraph. BlankLine tokens serve as
    /// paragraph separators.
    fn split_into_paragraphs(
        &self,
        tokens: &[crate::ast::tokens::Token],
    ) -> Vec<Vec<crate::ast::tokens::Token>> {
        let mut paragraph_groups = Vec::new();
        let mut current_paragraph = Vec::new();

        for token in tokens {
            match token {
                crate::ast::tokens::Token::BlankLine { .. } => {
                    // BlankLine marks the end of a paragraph
                    if !current_paragraph.is_empty() {
                        paragraph_groups.push(current_paragraph);
                        current_paragraph = Vec::new();
                    }
                    // Skip the BlankLine token itself
                }
                _ => {
                    // Add token to current paragraph
                    current_paragraph.push(token.clone());
                }
            }
        }

        // Add the final paragraph if it has content
        if !current_paragraph.is_empty() {
            paragraph_groups.push(current_paragraph);
        }

        paragraph_groups
    }

    /// Split tokens into session groups based on session boundaries
    ///
    /// A session consists of:
    /// 1. A title line (first non-blank token)
    /// 2. A blank line
    /// 3. Content tokens (everything after the blank line until next session title)
    ///
    /// Sessions are separated by blank lines followed by new session titles.
    fn split_into_sessions(
        &self,
        tokens: &[crate::ast::tokens::Token],
    ) -> Vec<Vec<crate::ast::tokens::Token>> {
        let mut session_groups = Vec::new();
        let mut current_session = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];

            match token {
                crate::ast::tokens::Token::BlankLine { .. } => {
                    // Skip blank lines before any title
                    i += 1;
                    continue;
                }
                _ => {
                    // This could be the start of a new session
                    // Collect tokens until we find a blank line
                    let mut session_tokens = Vec::new();
                    let mut found_blank = false;

                    // Collect the title line
                    while i < tokens.len() {
                        let current_token = &tokens[i];
                        match current_token {
                            crate::ast::tokens::Token::Newline { .. } => {
                                session_tokens.push(current_token.clone());
                                i += 1;
                                break;
                            }
                            _ => {
                                session_tokens.push(current_token.clone());
                                i += 1;
                            }
                        }
                    }

                    // Look for the blank line after the title
                    if i < tokens.len()
                        && matches!(tokens[i], crate::ast::tokens::Token::BlankLine { .. })
                    {
                        session_tokens.push(tokens[i].clone());
                        i += 1;
                        found_blank = true;
                    }

                    // If we found a blank line, this is a complete session
                    if found_blank && !session_tokens.is_empty() {
                        session_groups.push(session_tokens);
                    } else {
                        // If no blank line, this might not be a session
                        // Add remaining tokens to current session and break
                        current_session.extend(session_tokens);
                        break;
                    }
                }
            }
        }

        // Add any remaining tokens as a final session
        if !current_session.is_empty() {
            session_groups.push(current_session);
        }

        session_groups
    }

    /// Parse a session with its content from title tokens and child token tree
    fn parse_session_with_content(
        &self,
        title_tokens: &[crate::ast::tokens::Token],
        content_tree: TokenTree,
    ) -> Result<crate::ast::elements::session::block::SessionBlock, BlockParseError> {
        // For now, we'll treat all title tokens as one session
        // TODO: Implement proper session splitting for multiple sessions
        let title = self.parse_session_title(title_tokens)?;

        // Parse the session content from the child token tree
        let content = self.parse_session_content_from_tree(content_tree)?;

        // Create the session block
        let session = crate::ast::elements::session::block::SessionBlock {
            title,
            content,
            annotations: Vec::new(), // TODO: Parse annotations when implemented
            parameters: crate::ast::elements::components::parameters::Parameters::new(),
            tokens: crate::ast::elements::tokens::TokenSequence::new(),
        };

        Ok(session)
    }

    /// Parse a session title from tokens
    fn parse_session_title(
        &self,
        tokens: &[crate::ast::tokens::Token],
    ) -> Result<crate::ast::elements::session::block::SessionTitle, BlockParseError> {
        if tokens.is_empty() {
            return Err(BlockParseError::InvalidStructure(
                "Empty title tokens".to_string(),
            ));
        }

        // Extract title text from tokens (simple implementation for now)
        let title_text = tokens
            .iter()
            .filter_map(|token| match token {
                crate::ast::tokens::Token::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Create a simple text transform for the title
        let title_content = if title_text.is_empty() {
            Vec::new()
        } else {
            vec![crate::ast::elements::inlines::TextTransform::Identity(
                crate::ast::elements::inlines::TextSpan::simple(&title_text),
            )]
        };

        // For now, create a simple title without numbering detection
        // TODO: Implement proper numbering detection and parsing
        let title = crate::ast::elements::session::block::SessionTitle {
            content: title_content,
            numbering: None, // TODO: Detect and parse numbering
            tokens: crate::ast::elements::tokens::TokenSequence::new(),
        };

        Ok(title)
    }

    /// Parse session content from a child token tree
    fn parse_session_content_from_tree(
        &self,
        content_tree: TokenTree,
    ) -> Result<crate::ast::elements::session::session_container::SessionContainer, BlockParseError>
    {
        // Use the block parser to parse the content
        let elements = self.parse_blocks(content_tree)?;

        // Convert ElementNode to SessionContainerElement
        let content: Vec<crate::ast::elements::session::session_container::SessionContainerElement> = elements
            .into_iter()
            .map(|element| match element {
                ElementNode::ParagraphBlock(paragraph) => {
                    Ok(crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(paragraph))
                }
                ElementNode::SessionBlock(session) => {
                    Ok(crate::ast::elements::session::session_container::SessionContainerElement::Session(session))
                }
                // TODO: Handle other element types as they're implemented
                _ => {
                    // For now, skip unsupported elements
                    Err(BlockParseError::InvalidStructure(format!("Unsupported element in session content: {:?}", element)))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Create the session container
        let container = crate::ast::elements::session::session_container::SessionContainer {
            content,
            annotations: Vec::new(), // TODO: Parse annotations when implemented
            parameters: crate::ast::elements::components::parameters::Parameters::new(),
            tokens: crate::ast::elements::tokens::TokenSequence::new(),
        };

        Ok(container)
    }
}

/// Errors that can occur during block parsing
#[derive(Debug)]
pub enum BlockParseError {
    /// Invalid block structure detected
    InvalidStructure(String),
    /// Unsupported block type encountered
    UnsupportedBlockType(String),
    /// Parse error at specific position
    ParseError {
        position: crate::ast::tokens::Position,
        message: String,
    },
}

impl std::fmt::Display for BlockParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockParseError::InvalidStructure(msg) => write!(f, "Invalid block structure: {}", msg),
            BlockParseError::UnsupportedBlockType(block_type) => {
                write!(f, "Unsupported block type: {}", block_type)
            }
            BlockParseError::ParseError { position, message } => {
                write!(f, "Parse error at position {:?}: {}", position, message)
            }
        }
    }
}

impl std::error::Error for BlockParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_parser_creation() {
        let _parser = BlockParser::new();
        // Basic test to ensure parser can be created
        // The test passes if we reach this point without panicking
    }

    #[test]
    fn test_parse_blocks_placeholder() {
        let parser = BlockParser::new();
        let token_tree = TokenTree {
            tokens: vec![],
            children: vec![],
        };

        // This should return empty result until Phase 2 is implemented
        let result = parser.parse_blocks(token_tree);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
