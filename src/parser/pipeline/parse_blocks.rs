//! Phase 2a: Block Parsing
//!
//! Converts token trees into typed AST nodes for block elements.
//! This is the first step of Phase 2 parsing, where we take the hierarchical
//! token structure from the lexer and create proper AST element nodes.
//!
//! They challenge is to correctly disambiguate between a few elements, particularly
//! when nesting is involved, which typically requires a look ahead.
//!
//! All structured (nested) content in txxt is inside a container. Syntactically,
//! this is represented by indentation. The conceptual form for these:
//!
//! <element main node>
//!     <element content>
//! </element main node>
//!
//! IN syntactical terms, the closing of the element main node is explicit in some cases (Verbatim Block) while otherwise implictly by indendation changes (which are the containers)
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
//! - **Input**: `ScannerTokenTree` from lexer (Phase 1c)
//! - **Output**: AST tree of `ElementNode` variants

use crate::ast::ElementNode;
use crate::lexer::pipeline::ScannerTokenTree;
use crate::parser::elements::{
    paragraph::paragraph::parse_paragraph, session::session::parse_session,
};

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
    ///
    /// # Key Insight: Container-Based Indentation
    ///
    /// Every indent is caused by a container element:
    /// - Definition:: → Definition container
    /// - ::Annotation:: → Annotation container  
    /// - - list item → List container
    /// - Session Title → Session container
    /// - Verbatim Title: + label → Verbatim container
    ///
    /// The tricky part is distinguishing sessions from lists:
    /// - Sessions MUST be whitespace enclosed (blank lines before/after)
    /// - Lists CANNOT have blank lines between items
    ///
    /// Example of session vs list disambiguation:
    /// ```text
    /// 1. Foo
    ///
    ///     1. Bar    // This is a SESSION (blank line before)
    ///
    /// 2. Baz       // This is a LIST (no blank line before)
    /// ```
    pub fn parse_blocks(
        &self,
        token_tree: ScannerTokenTree,
    ) -> Result<Vec<ElementNode>, BlockParseError> {
        let mut elements = Vec::new();
        let root_tokens = token_tree.tokens.as_slice();
        let mut children = token_tree.children.as_slice();

        // Process all root-level tokens and their immediate children
        // Filter out Eof and BlankLine tokens as they should not be parsed as content
        let filtered_tokens: Vec<_> = root_tokens
            .iter()
            .filter(|token| {
                !matches!(
                    token,
                    crate::ast::scanner_tokens::ScannerToken::Eof { .. }
                        | crate::ast::scanner_tokens::ScannerToken::BlankLine { .. }
                )
            })
            .cloned()
            .collect();
        let mut root_tokens = filtered_tokens.as_slice();

        while !root_tokens.is_empty() {
            // Extract the first line of tokens
            let line_end = root_tokens
                .iter()
                .position(|t| matches!(t, crate::ast::scanner_tokens::ScannerToken::Newline { .. }))
                .map(|p| p + 1)
                .unwrap_or(root_tokens.len());

            let (line_tokens, rest_tokens) = root_tokens.split_at(line_end);
            root_tokens = rest_tokens;

            // Check if this line has indented content (children)
            if !children.is_empty() {
                // This line has indented content - it's a container element
                // The key insight: blank lines + indentation determine the type, not markers

                // Check for explicit container markers first (easy cases)
                if self.is_definition_marker(line_tokens) {
                    // Definition container - parse as definition
                    // TODO: Implement definition parsing
                    elements.push(ElementNode::ParagraphBlock(parse_paragraph(line_tokens)?));
                } else if self.is_annotation_marker(line_tokens) {
                    // Annotation container - parse as annotation
                    // TODO: Implement annotation parsing
                    elements.push(ElementNode::ParagraphBlock(parse_paragraph(line_tokens)?));
                } else {
                    // No explicit markers - this is either a session or list
                    // The key distinction: sessions require whitespace enclosure (blank lines)
                    // Lists cannot have blank lines between items

                    // Check if this looks like a session by examining whitespace patterns
                    if self.is_potential_session(line_tokens, &children[0]) {
                        // This is a SESSION - it has proper whitespace enclosure
                        let mut session_tokens = line_tokens.to_vec();
                        session_tokens.push(crate::ast::scanner_tokens::ScannerToken::BlankLine {
                            whitespace: String::new(),
                            span: children[0].tokens[0].span().clone(),
                        });
                        session_tokens.extend(children[0].tokens.clone());

                        match parse_session(&session_tokens) {
                            Ok(session) => {
                                elements.push(ElementNode::SessionBlock(session));
                                children = &children[1..]; // Consume the child
                                continue;
                            }
                            Err(_) => {
                                // Session parsing failed - fall back to paragraph
                                // This handles cases like "1. This is a paragraph" (no indented content)
                                elements.push(ElementNode::ParagraphBlock(parse_paragraph(
                                    line_tokens,
                                )?));
                            }
                        }
                    } else {
                        // This is likely a LIST - no whitespace enclosure
                        // Lists cannot have blank lines between items
                        // TODO: Implement list parsing
                        // For now, fall back to paragraph
                        elements.push(ElementNode::ParagraphBlock(parse_paragraph(line_tokens)?));
                    }
                }
            } else {
                // No indented content - this is a paragraph
                // Markers don't define the type - blank lines + indentation do
                // So "1. This is a paragraph" (followed by blank line, not indented content) is a paragraph
                elements.push(ElementNode::ParagraphBlock(parse_paragraph(line_tokens)?));
            }
        }

        // Recursively parse any remaining children
        // These are container contents that need to be parsed as blocks
        for child_tree in children {
            // Check if this child tree contains multiple potential sessions
            // Sessions are identified by blank lines + indented content patterns
            let session_boundaries = self.find_session_boundaries(&child_tree.tokens);

            if session_boundaries.len() > 1 {
                // Multiple sessions - split and parse each one
                let mut start_pos = 0;
                for &boundary_pos in &session_boundaries {
                    if boundary_pos > start_pos {
                        // Parse tokens before this session as paragraphs
                        let before_tokens = &child_tree.tokens[start_pos..boundary_pos];
                        let paragraph_groups = self.split_into_paragraphs(before_tokens);
                        for paragraph_tokens in paragraph_groups {
                            if !paragraph_tokens.is_empty() {
                                match parse_paragraph(&paragraph_tokens) {
                                    Ok(paragraph) => {
                                        elements.push(ElementNode::ParagraphBlock(paragraph));
                                    }
                                    Err(_) => {
                                        // Skip unrecognized tokens for now
                                    }
                                }
                            }
                        }
                    }

                    // Find the end of this session (next boundary or end of tokens)
                    let next_boundary_pos = session_boundaries
                        .iter()
                        .find(|&&pos| pos > boundary_pos)
                        .copied()
                        .unwrap_or(child_tree.tokens.len());

                    // Extract tokens for this session
                    let session_tokens = &child_tree.tokens[boundary_pos..next_boundary_pos];

                    // Try to parse as session
                    match self.try_parse_as_session(session_tokens) {
                        Ok(session) => {
                            elements.push(ElementNode::SessionBlock(session));
                        }
                        Err(_) => {
                            // Not a session, parse as paragraph
                            let paragraph_groups = self.split_into_paragraphs(session_tokens);
                            for paragraph_tokens in paragraph_groups {
                                if !paragraph_tokens.is_empty() {
                                    match parse_paragraph(&paragraph_tokens) {
                                        Ok(paragraph) => {
                                            elements.push(ElementNode::ParagraphBlock(paragraph));
                                        }
                                        Err(_) => {
                                            // Skip unrecognized tokens for now
                                        }
                                    }
                                }
                            }
                        }
                    }

                    start_pos = next_boundary_pos;
                }
            } else {
                // Single or no sessions - parse normally (recursively)
                elements.extend(self.parse_blocks(child_tree.clone())?);
            }
        }

        Ok(elements)
    }

    /// Find potential session boundaries in a token sequence
    ///
    /// Sessions are identified by patterns like:
    /// - Text title followed by blank line followed by indented content
    /// - Sequence marker followed by text followed by blank line followed by indented content
    fn find_session_boundaries(
        &self,
        tokens: &[crate::ast::scanner_tokens::ScannerToken],
    ) -> Vec<usize> {
        let mut boundaries = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            // Look for text tokens that could be session titles
            if matches!(token, crate::ast::scanner_tokens::ScannerToken::Text { .. }) {
                // Check if this text is at the start of a line (after whitespace or at beginning)
                let is_at_line_start = i == 0
                    || matches!(
                        tokens[i - 1],
                        crate::ast::scanner_tokens::ScannerToken::Whitespace { .. }
                    );

                if is_at_line_start {
                    // Look ahead to see if this is followed by a blank line and then indented content
                    if let Some(blank_line_pos) = self.find_blank_line_after(tokens, i) {
                        if let Some(_indented_content_pos) =
                            self.find_indented_content_after(tokens, blank_line_pos)
                        {
                            // This looks like a session: text title -> blank line -> indented content
                            boundaries.push(i);
                        }
                    }
                }
            }
        }

        boundaries
    }

    /// Find a blank line after a given position
    fn find_blank_line_after(
        &self,
        tokens: &[crate::ast::scanner_tokens::ScannerToken],
        start_pos: usize,
    ) -> Option<usize> {
        for (i, token) in tokens.iter().enumerate().skip(start_pos + 1) {
            if matches!(
                token,
                crate::ast::scanner_tokens::ScannerToken::BlankLine { .. }
            ) {
                return Some(i);
            }
        }
        None
    }

    /// Find indented content after a blank line
    fn find_indented_content_after(
        &self,
        tokens: &[crate::ast::scanner_tokens::ScannerToken],
        blank_line_pos: usize,
    ) -> Option<usize> {
        for (i, token) in tokens.iter().enumerate().skip(blank_line_pos + 1) {
            if matches!(
                token,
                crate::ast::scanner_tokens::ScannerToken::Whitespace { .. }
            ) {
                // Check if this whitespace represents indentation (multiple spaces)
                if let crate::ast::scanner_tokens::ScannerToken::Whitespace { content, .. } = token
                {
                    if content.len() >= 4 {
                        // At least 4 spaces for indentation
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Try to parse tokens as a session
    fn try_parse_as_session(
        &self,
        tokens: &[crate::ast::scanner_tokens::ScannerToken],
    ) -> Result<crate::ast::elements::session::SessionBlock, BlockParseError> {
        // Find the first line (up to newline)
        let line_end = tokens
            .iter()
            .position(|t| matches!(t, crate::ast::scanner_tokens::ScannerToken::Newline { .. }))
            .map(|p| p + 1)
            .unwrap_or(tokens.len());

        let title_tokens = &tokens[0..line_end];

        // Add a blank line after the title for session parsing
        let mut session_tokens = title_tokens.to_vec();
        session_tokens.push(crate::ast::scanner_tokens::ScannerToken::BlankLine {
            whitespace: String::new(),
            span: crate::ast::scanner_tokens::SourceSpan {
                start: crate::ast::scanner_tokens::Position { row: 0, column: 0 },
                end: crate::ast::scanner_tokens::Position { row: 0, column: 0 },
            },
        });

        // Add the rest of the tokens as content
        session_tokens.extend(tokens[line_end..].iter().cloned());

        parse_session(&session_tokens)
    }

    /// Check if tokens represent a definition marker (ends with ::)
    fn is_definition_marker(&self, tokens: &[crate::ast::scanner_tokens::ScannerToken]) -> bool {
        // Look for :: at the end of the line
        // This is a simplified check - in practice we'd need to look for two consecutive colons
        tokens.iter().any(|token| {
            matches!(
                token,
                crate::ast::scanner_tokens::ScannerToken::Colon { .. }
            )
        })
    }

    /// Check if tokens represent an annotation marker (starts with ::)
    fn is_annotation_marker(&self, tokens: &[crate::ast::scanner_tokens::ScannerToken]) -> bool {
        // Look for :: at the start of the line
        // This is a simplified check - in practice we'd need to look for two consecutive colons
        tokens.first().is_some_and(|token| {
            matches!(
                token,
                crate::ast::scanner_tokens::ScannerToken::Colon { .. }
            )
        })
    }

    /// Check if a line with indented content is a potential session
    ///
    /// Sessions are distinguished from lists by whitespace enclosure:
    /// - Sessions MUST have blank lines before/after (whitespace enclosed)
    /// - Lists CANNOT have blank lines between items
    ///
    /// This function examines the whitespace patterns to determine if
    /// the indented content represents a session container.
    fn is_potential_session(
        &self,
        line_tokens: &[crate::ast::scanner_tokens::ScannerToken],
        child_tree: &ScannerTokenTree,
    ) -> bool {
        // A session is identified by:
        // 1. Having indented content (we know this is true)
        // 2. Being whitespace enclosed (blank lines before/after)
        // 3. The indented content should be parseable as session content

        // Check if the child tree starts with a blank line (whitespace enclosure)
        // This indicates the content is properly separated and likely a session
        if let Some(first_token) = child_tree.tokens.first() {
            if matches!(
                first_token,
                crate::ast::scanner_tokens::ScannerToken::BlankLine { .. }
            ) {
                return true;
            }
        }

        // Also check if the line itself looks like a session title
        // Sessions can have sequence markers or be plain text
        let has_sequence_marker = line_tokens.iter().any(|token| {
            matches!(
                token,
                crate::ast::scanner_tokens::ScannerToken::SequenceMarker { .. }
            )
        });

        let has_text_content = line_tokens
            .iter()
            .any(|token| matches!(token, crate::ast::scanner_tokens::ScannerToken::Text { .. }));

        // If it has either a sequence marker or text content, and has indented children,
        // it's likely a session (as opposed to a list which would have different patterns)
        has_sequence_marker || has_text_content
    }

    /// Split tokens into paragraph groups based on BlankLine boundaries
    fn split_into_paragraphs(
        &self,
        tokens: &[crate::ast::scanner_tokens::ScannerToken],
    ) -> Vec<Vec<crate::ast::scanner_tokens::ScannerToken>> {
        tokens
            .split(|token| {
                matches!(
                    token,
                    crate::ast::scanner_tokens::ScannerToken::BlankLine { .. }
                )
            })
            .map(|s| s.to_vec())
            .filter(|s| !s.is_empty())
            .collect()
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
        position: crate::ast::scanner_tokens::Position,
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
    use crate::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};

    #[test]
    fn test_block_parser_creation() {
        let _parser = BlockParser::new();
        // Basic test to ensure parser can be created
        // The test passes if we reach this point without panicking
    }

    #[test]
    fn test_parse_blocks_placeholder() {
        let parser = BlockParser::new();
        let token_tree = ScannerTokenTree {
            tokens: vec![],
            children: vec![],
        };

        // This should return empty result until Phase 2 is implemented
        let result = parser.parse_blocks(token_tree);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_paragraph_detection() {
        use crate::lexer::elements::paragraph::detect_paragraph;

        // Create tokens for "This is a simple paragraph."
        let tokens = vec![
            ScannerToken::Text {
                content: "This".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 4 },
                },
            },
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 4 },
                    end: Position { row: 0, column: 5 },
                },
            },
            ScannerToken::Text {
                content: "is".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 5 },
                    end: Position { row: 0, column: 7 },
                },
            },
            ScannerToken::Period {
                span: SourceSpan {
                    start: Position { row: 0, column: 7 },
                    end: Position { row: 0, column: 8 },
                },
            },
        ];

        let result = detect_paragraph(&tokens);
        match result {
            crate::lexer::elements::paragraph::ParagraphParseResult::ValidParagraph(_paragraph) => {
                // Success
            }
            crate::lexer::elements::paragraph::ParagraphParseResult::NotParagraph => {
                panic!("Expected valid paragraph");
            }
            crate::lexer::elements::paragraph::ParagraphParseResult::Invalid(error) => {
                panic!("Invalid paragraph: {}", error);
            }
        }
    }

    #[test]
    fn test_parse_simple_paragraph() {
        let parser = BlockParser::new();

        // Create tokens for "This is a simple paragraph."
        let tokens = vec![
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 4 },
                    end: Position { row: 0, column: 5 },
                },
            },
            ScannerToken::Text {
                content: "is".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 5 },
                    end: Position { row: 0, column: 7 },
                },
            },
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 7 },
                    end: Position { row: 0, column: 8 },
                },
            },
            ScannerToken::Text {
                content: "a".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 8 },
                    end: Position { row: 0, column: 9 },
                },
            },
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 9 },
                    end: Position { row: 0, column: 10 },
                },
            },
            ScannerToken::Text {
                content: "simple".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 10 },
                    end: Position { row: 0, column: 16 },
                },
            },
            ScannerToken::Whitespace {
                content: " ".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 16 },
                    end: Position { row: 0, column: 17 },
                },
            },
            ScannerToken::Text {
                content: "paragraph".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 17 },
                    end: Position { row: 0, column: 26 },
                },
            },
            ScannerToken::Period {
                span: SourceSpan {
                    start: Position { row: 0, column: 26 },
                    end: Position { row: 0, column: 27 },
                },
            },
            ScannerToken::Newline {
                span: SourceSpan {
                    start: Position { row: 0, column: 27 },
                    end: Position { row: 1, column: 0 },
                },
            },
            ScannerToken::Eof {
                span: SourceSpan {
                    start: Position { row: 1, column: 0 },
                    end: Position { row: 1, column: 0 },
                },
            },
        ];

        let token_tree = ScannerTokenTree {
            tokens,
            children: vec![],
        };

        let result = parser.parse_blocks(token_tree);
        assert!(
            result.is_ok(),
            "Block parser should succeed on simple paragraph"
        );

        let elements = result.unwrap();
        assert_eq!(elements.len(), 1, "Should parse one paragraph element");

        match &elements[0] {
            ElementNode::ParagraphBlock(_) => {
                // Success - it's a paragraph
            }
            _ => panic!("Expected ParagraphBlock, got {:?}", elements[0]),
        }
    }
}
