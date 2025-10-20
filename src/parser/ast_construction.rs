//! Phase 2b: AST Construction
//!
//! Converts semantic tokens into AST tree nodes using a homegrown grammar engine
//! with carefully planned precedence rules.
//!
//! This phase focuses on the core block elements: Session, Paragraph, List,
//! Annotation, Definition, and Verbatim.

use crate::ast::elements::{
    annotation::annotation_block::AnnotationBlock, definition::block::DefinitionBlock,
    list::block::ListBlock, paragraph::block::ParagraphBlock, session::block::SessionBlock,
    verbatim::block::VerbatimBlock,
};
use crate::ast::{
    scanner_tokens::{Position, ScannerTokenSequence, SourceSpan},
    tokens::semantic::{SemanticToken, SemanticTokenList},
};
use crate::parser::BlockParseError;

/// AST Construction parser for converting semantic tokens to AST nodes
///
/// This parser takes semantic tokens and transforms them into structured AST nodes
/// using precedence-based pattern matching.
pub struct AstConstructor<'a> {
    /// The semantic token stream being parsed
    tokens: &'a [SemanticToken],
    /// Current parsing position in the token stream
    position: usize,
    /// Current indentation level for nested parsing
    indentation_level: usize,
}

impl<'a> AstConstructor<'a> {
    /// Create a new AST constructor instance
    pub fn new() -> Self {
        Self {
            tokens: &[],
            position: 0,
            indentation_level: 0,
        }
    }

    /// Create a new AST constructor instance with token stream
    pub fn with_tokens(tokens: &'a [SemanticToken]) -> Self {
        Self {
            tokens,
            position: 0,
            indentation_level: 0,
        }
    }

    /// Parse semantic tokens into AST nodes
    ///
    /// This is the main entry point for AST construction. It iterates through
    /// the semantic tokens and constructs AST nodes using precedence rules.
    ///
    /// # Arguments
    /// * `semantic_tokens` - The semantic token list to parse
    ///
    /// # Returns
    /// * `Result<Vec<AstNode>, BlockParseError>` - Parsed AST nodes
    pub fn parse(
        &mut self,
        semantic_tokens: &'a SemanticTokenList,
    ) -> Result<Vec<AstNode>, BlockParseError> {
        // Update the token stream
        self.tokens = &semantic_tokens.tokens;
        let mut ast_nodes = Vec::new();

        // Reset parser state
        self.position = 0;
        self.indentation_level = 0;

        eprintln!("AST: Total tokens to process: {}", self.tokens.len());
        for (i, token) in self.tokens.iter().enumerate() {
            eprintln!("AST: Token {}: {:?}", i, token);
        }

        // Main parsing loop
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            eprintln!(
                "AST: Processing token at position {}: {:?}",
                self.position, token
            );

            // Handle structural tokens that don't contribute to content
            match token {
                SemanticToken::Indent { .. } => {
                    self.indentation_level += 1;
                    self.position += 1;
                    continue;
                }
                SemanticToken::Dedent { .. } => {
                    if self.indentation_level > 0 {
                        self.indentation_level -= 1;
                    }
                    self.position += 1;
                    continue;
                }
                _ => {
                    // Process all content tokens through the dispatcher
                    eprintln!(
                        "AST: Calling dispatch_parsing at position {}",
                        self.position
                    );
                    if let Some((node, _tokens_consumed)) = self.dispatch_parsing()? {
                        eprintln!("AST: Got node, adding to ast_nodes");
                        ast_nodes.push(node);
                    } else {
                        eprintln!("AST: dispatch_parsing returned None");
                    }
                    eprintln!(
                        "AST: After dispatch_parsing, position is now {}",
                        self.position
                    );
                }
            }
        }

        Ok(ast_nodes)
    }

    /// Dispatch parsing based on current token(s) and precedence rules
    ///
    /// This function looks at the current token(s) and decides which specific
    /// parsing function to call based on precedence rules.
    ///
    /// # Returns
    /// * `Result<Option<(TempAstNode, usize)>, BlockParseError>` - Parsed node and tokens consumed
    fn dispatch_parsing(&mut self) -> Result<Option<(AstNode, usize)>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let current_token = &self.tokens[self.position];

        // Apply precedence rules (first match wins):
        // 1. VerbatimBlock pattern (highest precedence)
        if let Some(node) = self.try_parse_verbatim_block()? {
            return Ok(Some((node, 1)));
        }

        // 2. Annotation pattern
        if let Some(node) = self.try_parse_annotation()? {
            return Ok(Some((node, 1)));
        }

        // 3. Definition pattern
        if let Some(node) = self.try_parse_definition()? {
            return Ok(Some((node, 1)));
        }

        // 4. Session pattern (check for BlankLine and trigger lookahead)
        if matches!(current_token, SemanticToken::BlankLine { .. }) {
            if let Some((node, tokens_consumed)) = self.try_parse_session()? {
                return Ok(Some((node, tokens_consumed)));
            }
            // If not a session, skip the blank line and continue
            self.consume(); // Consume the blank line
            return Ok(None); // Return None to continue main parsing loop
        }

        // 5. List pattern
        if let Some((node, tokens_consumed)) = self.try_parse_list()? {
            return Ok(Some((node, tokens_consumed)));
        }

        // 6. Paragraph pattern (catch-all, lowest precedence)
        if let Some(node) = self.try_parse_paragraph()? {
            return Ok(Some((node, 1)));
        }

        // No pattern matched
        Ok(None)
    }

    /// Check if we're at the end of the token stream
    #[allow(dead_code)] // Will be used in next steps
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Get current token without advancing position
    #[allow(dead_code)] // Will be used in next steps
    fn peek(&self) -> Option<&SemanticToken> {
        self.tokens.get(self.position)
    }

    /// Get current token and advance position
    #[allow(dead_code)] // Will be used in next steps
    fn consume(&mut self) -> Option<&SemanticToken> {
        if self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Try to parse an annotation semantic token
    ///
    /// Annotations are already constructed semantic tokens, so we just need to
    /// extract the information and create a temporary AST node.
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Annotation node if matched
    fn try_parse_annotation(&mut self) -> Result<Option<AstNode>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let token = &self.tokens[self.position];
        match token {
            SemanticToken::Annotation { label, content, .. } => {
                // Consume the token
                self.position += 1;

                // Extract label text
                let label_text = match label.as_ref() {
                    SemanticToken::Label { text, .. } => text.clone(),
                    _ => "unknown".to_string(),
                };

                // Extract content text if present
                let _content_text = match content {
                    Some(content_token) => match content_token.as_ref() {
                        SemanticToken::TextSpan { content, .. } => Some(content.clone()),
                        SemanticToken::PlainTextLine { content, .. } => match content.as_ref() {
                            SemanticToken::TextSpan { content, .. } => Some(content.clone()),
                            _ => None,
                        },
                        _ => None,
                    },
                    None => None,
                };

                Ok(Some(AstNode::Annotation(AnnotationBlock {
                    label: label_text,
                    content: crate::ast::elements::annotation::annotation_block::AnnotationContent::Inline(vec![]),
                    parameters: crate::ast::elements::components::parameters::Parameters::new(),
                    annotations: Vec::new(),
                    tokens: ScannerTokenSequence::new(),
                    namespace: None,
                })))
            }
            _ => Ok(None),
        }
    }

    /// Try to parse a verbatim block semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Verbatim block node if matched
    fn try_parse_verbatim_block(&mut self) -> Result<Option<AstNode>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let token = &self.tokens[self.position];
        match token {
            SemanticToken::VerbatimBlock { title, label, .. } => {
                // Consume the token
                self.position += 1;

                // Extract title text
                let _title_text = match title.as_ref() {
                    SemanticToken::TextSpan { content, .. } => content.clone(),
                    _ => "unknown".to_string(),
                };

                // Extract label text
                let label_text = match label.as_ref() {
                    SemanticToken::Label { text, .. } => text.clone(),
                    _ => "unknown".to_string(),
                };

                Ok(Some(AstNode::VerbatimBlock(VerbatimBlock {
                    title: vec![], // TODO: Convert title_text to TextTransform
                    content: crate::ast::elements::verbatim::ignore_container::IgnoreContainer::new(
                        vec![],
                        vec![],
                        vec![],
                        crate::ast::elements::components::parameters::Parameters::new(),
                        ScannerTokenSequence::new(),
                    ),
                    label: label_text,
                    verbatim_type: crate::ast::elements::verbatim::block::VerbatimType::InFlow,
                    parameters: crate::ast::elements::components::parameters::Parameters::new(),
                    annotations: Vec::new(),
                    tokens: ScannerTokenSequence::new(),
                })))
            }
            _ => Ok(None),
        }
    }

    /// Try to parse a definition semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Definition node if matched
    fn try_parse_definition(&mut self) -> Result<Option<AstNode>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let token = &self.tokens[self.position];
        match token {
            SemanticToken::Definition {
                term, parameters, ..
            } => {
                // Consume the token
                self.position += 1;

                // Extract term text
                let _term_text = match term.as_ref() {
                    SemanticToken::TextSpan { content, .. } => content.clone(),
                    _ => "unknown".to_string(),
                };

                // Extract parameters text if present
                let _params_text = match parameters {
                    Some(params_token) => {
                        match params_token.as_ref() {
                            SemanticToken::Parameters { params, .. } => {
                                // Convert parameters to string representation
                                let param_strings: Vec<String> =
                                    params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                                Some(param_strings.join(","))
                            }
                            _ => None,
                        }
                    }
                    None => None,
                };

                Ok(Some(AstNode::Definition(DefinitionBlock {
                    term: crate::ast::elements::definition::block::DefinitionTerm {
                        content: vec![], // TODO: Convert term_text to TextTransform
                        tokens: ScannerTokenSequence::new(),
                    },
                    content: crate::ast::elements::containers::content::ContentContainer::new(
                        vec![],
                        vec![],
                        crate::ast::elements::components::parameters::Parameters::new(),
                        ScannerTokenSequence::new(),
                    ),
                    parameters: crate::ast::elements::components::parameters::Parameters::new(),
                    annotations: Vec::new(),
                    tokens: ScannerTokenSequence::new(),
                })))
            }
            _ => Ok(None),
        }
    }

    /// Try to parse a session pattern
    ///
    /// Sessions follow the pattern:
    /// 1. Blank line (mandatory)
    /// 2. Title line (can be plain text, sequence marker, or definition)
    /// 3. Blank line (mandatory)
    /// 4. Indent token
    /// 5. Content block (must have at least one indented child)
    ///
    /// The content block is recursively parsed to handle nested sessions, lists, etc.
    ///
    /// # Returns
    /// * `Result<Option<(TempAstNode, usize)>, BlockParseError>` - Session node and tokens consumed if matched
    fn try_parse_session(&mut self) -> Result<Option<(AstNode, usize)>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        // Save the starting position
        let start_position = self.position;

        // Step 1: Must start with blank line
        if self.position >= self.tokens.len() {
            return Ok(None);
        }
        let first_token = &self.tokens[self.position];
        match first_token {
            SemanticToken::BlankLine { .. } => {
                self.position += 1; // Consume the blank line
            }
            _ => return Ok(None),
        }

        // Step 2: Must have title line (any content except blank line)
        if self.position >= self.tokens.len() {
            return Ok(None);
        }
        let title_token = &self.tokens[self.position];
        let _title_text = match title_token {
            SemanticToken::PlainTextLine { content, .. } => match content.as_ref() {
                SemanticToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            SemanticToken::SequenceTextLine { content, .. } => match content.as_ref() {
                SemanticToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            SemanticToken::Definition { term, .. } => match term.as_ref() {
                SemanticToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            _ => return Ok(None), // Not a valid title
        };
        self.position += 1; // Consume the title token

        // Step 3: Must have blank line after title
        if self.position >= self.tokens.len() {
            return Ok(None);
        }
        let second_blank_token = &self.tokens[self.position];
        match second_blank_token {
            SemanticToken::BlankLine { .. } => {
                self.position += 1; // Consume the blank line
            }
            _ => return Ok(None),
        }

        // Step 4: Must have indent token
        if self.position >= self.tokens.len() {
            return Ok(None);
        }
        let indent_token = &self.tokens[self.position];
        match indent_token {
            SemanticToken::Indent { .. } => {
                self.position += 1; // Consume the indent token
                self.indentation_level += 1; // Track indentation level
            }
            _ => return Ok(None),
        }

        // Step 5: Parse indented content recursively
        let mut child_nodes = Vec::new();
        let mut child_count = 0;

        // Parse all indented content until we hit a dedent
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            match token {
                SemanticToken::Dedent { .. } => {
                    self.position += 1; // Consume the dedent token
                    self.indentation_level -= 1; // Track indentation level
                    break;
                }
                SemanticToken::BlankLine { .. } => {
                    self.position += 1; // Consume blank lines
                    continue;
                }
                _ => {
                    // Parse the content token using the dispatcher
                    if let Some((node, _tokens_consumed)) = self.dispatch_parsing()? {
                        child_nodes.push(node);
                        child_count += 1;
                    } else {
                        // If no pattern matched, advance to avoid infinite loop
                        self.position += 1;
                    }
                }
            }
        }

        if child_count == 0 {
            return Ok(None); // No children, not a session
        }

        // We have a valid session pattern!
        // Use a fallback span since we consumed the first token
        let _span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 0 },
        };

        // Calculate how many tokens we consumed
        let tokens_consumed = self.position - start_position;

        Ok(Some((
            AstNode::Session(SessionBlock {
                title: crate::ast::elements::session::block::SessionTitle {
                    content: vec![], // TODO: Convert title_text to TextTransform
                    numbering: None,
                    tokens: ScannerTokenSequence::new(),
                },
                content: crate::ast::elements::session::session_container::SessionContainer {
                    content: vec![], // TODO: Add parsed child nodes
                    annotations: Vec::new(),
                    parameters: crate::ast::elements::components::parameters::Parameters::new(),
                    tokens: ScannerTokenSequence::new(),
                },
                annotations: Vec::new(),
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                tokens: ScannerTokenSequence::new(),
            }),
            tokens_consumed,
        )))
    }

    /// Try to parse a list pattern
    ///
    /// Lists follow the pattern:
    /// 1. Sequence of at least 2 items with sequence markers
    /// 2. Items cannot have blank lines between them
    /// 3. Items can be nested (but we'll handle simple lists first)
    ///
    /// Each list item is parsed recursively to handle nested content.
    ///
    /// # Returns
    /// * `Result<Option<(TempAstNode, usize)>, BlockParseError>` - List node and tokens consumed if matched
    fn try_parse_list(&mut self) -> Result<Option<(AstNode, usize)>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        // Save the starting position
        let start_position = self.position;
        let mut item_count = 0;
        let mut has_blank_lines = false;

        // Count consecutive sequence text lines and parse their content
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            match token {
                SemanticToken::SequenceTextLine { .. } => {
                    item_count += 1;
                    self.position += 1; // Consume the sequence text line

                    // Parse any indented content that follows this list item
                    self.parse_list_item_content()?;
                }
                SemanticToken::BlankLine { .. } => {
                    // Check if this blank line is between list items
                    if item_count > 0 {
                        // Look ahead to see if there's another sequence marker
                        let mut next_pos = self.position + 1;
                        while next_pos < self.tokens.len() {
                            let next_token = &self.tokens[next_pos];
                            match next_token {
                                SemanticToken::BlankLine { .. } => {
                                    next_pos += 1;
                                    continue;
                                }
                                SemanticToken::SequenceTextLine { .. } => {
                                    has_blank_lines = true;
                                    break;
                                }
                                _ => break,
                            }
                        }
                    }
                    self.position += 1; // Consume the blank line
                }
                _ => break,
            }
        }

        // Lists must have at least 2 items
        if item_count < 2 {
            return Ok(None);
        }

        // Lists cannot have blank lines between items
        if has_blank_lines {
            return Ok(None);
        }

        // We have a valid list pattern!
        // Use a fallback span since we consumed the tokens
        let _span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 0 },
        };

        // Calculate how many tokens we consumed
        let tokens_consumed = self.position - start_position;

        Ok(Some((
            AstNode::List(ListBlock {
                decoration_type: crate::ast::elements::list::block::ListDecorationType {
                    style: crate::ast::elements::list::block::NumberingStyle::Plain,
                    form: crate::ast::elements::list::block::NumberingForm::Short,
                },
                items: vec![], // TODO: Add parsed list items
                annotations: Vec::new(),
                parameters: crate::ast::elements::components::parameters::Parameters::new(),
                tokens: ScannerTokenSequence::new(),
            }),
            tokens_consumed,
        )))
    }

    /// Parse the content of a list item (indented content following a sequence text line)
    ///
    /// This handles nested content within list items, including nested lists and sessions.
    fn parse_list_item_content(&mut self) -> Result<(), BlockParseError> {
        // Check if there's indented content following the list item
        if self.position >= self.tokens.len() {
            return Ok(());
        }

        let token = &self.tokens[self.position];
        match token {
            SemanticToken::Indent { .. } => {
                self.position += 1; // Consume the indent token
                self.indentation_level += 1; // Track indentation level

                // Parse all indented content until we hit a dedent
                while self.position < self.tokens.len() {
                    let token = &self.tokens[self.position];
                    match token {
                        SemanticToken::Dedent { .. } => {
                            self.position += 1; // Consume the dedent token
                            self.indentation_level -= 1; // Track indentation level
                            break;
                        }
                        SemanticToken::BlankLine { .. } => {
                            self.position += 1; // Consume blank lines
                            continue;
                        }
                        _ => {
                            // Parse the content token using the dispatcher
                            if let Some((_node, _tokens_consumed)) = self.dispatch_parsing()? {
                                // Content parsed successfully
                            } else {
                                // If no pattern matched, advance to avoid infinite loop
                                self.position += 1;
                            }
                        }
                    }
                }
            }
            _ => {
                // No indented content, this is a simple list item
            }
        }

        Ok(())
    }

    /// Try to parse a paragraph semantic token
    ///
    /// # Arguments
    /// * `token` - The current semantic token
    ///
    /// # Returns
    /// * `Result<Option<TempAstNode>, BlockParseError>` - Paragraph node if matched
    fn try_parse_paragraph(&mut self) -> Result<Option<AstNode>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let token = &self.tokens[self.position];
        match token {
            SemanticToken::PlainTextLine { content, .. } => {
                // Consume the token
                self.position += 1;

                // Extract content text
                let content_text = match content.as_ref() {
                    SemanticToken::TextSpan { content, .. } => content.clone(),
                    _ => "unknown".to_string(),
                };

                // Create a simple TextTransform::Identity for the plain text content
                let text_span = crate::ast::elements::inlines::TextSpan::simple(&content_text);
                let text_transform =
                    crate::ast::elements::inlines::TextTransform::Identity(text_span);

                Ok(Some(AstNode::Paragraph(ParagraphBlock {
                    content: vec![text_transform],
                    annotations: Vec::new(),
                    parameters: crate::ast::elements::components::parameters::Parameters::new(),
                    tokens: ScannerTokenSequence::new(),
                })))
            }
            _ => Ok(None),
        }
    }
}

impl<'a> Default for AstConstructor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// AST node types that can be constructed from semantic tokens
///
/// This enum represents the different types of AST nodes that can be created
/// during the AST construction phase. Each variant contains the actual
/// AST structure for that element type.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Annotation block node
    Annotation(AnnotationBlock),
    /// Definition block node
    Definition(DefinitionBlock),
    /// Verbatim block node
    VerbatimBlock(VerbatimBlock),
    /// Session block node
    Session(SessionBlock),
    /// List block node
    List(ListBlock),
    /// Paragraph block node
    Paragraph(ParagraphBlock),
}

impl AstNode {
    /// Convert an AstNode to an ElementNode for integration with the parsing pipeline
    ///
    /// This function converts the internal AST construction representation
    /// to the standard ElementNode format used throughout the pipeline.
    ///
    /// # Returns
    /// * `ElementNode` - The converted element node
    pub fn to_element_node(&self) -> crate::ast::elements::core::ElementNode {
        match self {
            AstNode::Annotation(block) => {
                crate::ast::elements::core::ElementNode::AnnotationBlock(block.clone())
            }
            AstNode::Definition(block) => {
                crate::ast::elements::core::ElementNode::DefinitionBlock(block.clone())
            }
            AstNode::VerbatimBlock(block) => {
                crate::ast::elements::core::ElementNode::VerbatimBlock(block.clone())
            }
            AstNode::Session(block) => {
                crate::ast::elements::core::ElementNode::SessionBlock(block.clone())
            }
            AstNode::List(block) => {
                crate::ast::elements::core::ElementNode::ListBlock(block.clone())
            }
            AstNode::Paragraph(block) => {
                crate::ast::elements::core::ElementNode::ParagraphBlock(block.clone())
            }
        }
    }
}

impl AstConstructor<'_> {
    /// Parse semantic tokens and return ElementNodes for pipeline integration
    ///
    /// This is a convenience method that parses semantic tokens and converts
    /// the resulting AstNodes to ElementNodes for use in the parsing pipeline.
    ///
    /// # Arguments
    /// * `semantic_tokens` - The semantic token list to parse
    ///
    /// # Returns
    /// * `Result<Vec<ElementNode>, BlockParseError>` - Vector of ElementNodes
    pub fn parse_to_element_nodes(
        semantic_tokens: &SemanticTokenList,
    ) -> Result<Vec<crate::ast::elements::core::ElementNode>, BlockParseError> {
        let mut constructor = AstConstructor::new();
        let ast_nodes = constructor.parse(semantic_tokens)?;
        Ok(ast_nodes
            .into_iter()
            .map(|node| node.to_element_node())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::scanner_tokens::{Position, SourceSpan};
    use crate::ast::tokens::semantic::{
        SemanticNumberingForm, SemanticNumberingStyle, SemanticTokenBuilder, SemanticTokenList,
    };

    /// Test that the parser machinery initializes correctly
    #[test]
    fn test_parser_initialization() {
        let parser = AstConstructor::new();
        assert_eq!(parser.position, 0);
        assert_eq!(parser.indentation_level, 0);
    }

    /// Test that the parser handles empty semantic token list
    #[test]
    fn test_parse_empty_tokens() {
        let mut parser = AstConstructor::new();
        let empty_tokens = SemanticTokenList::with_tokens(vec![]);

        let result = parser.parse(&empty_tokens);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Test that the parser skips structural tokens correctly
    #[test]
    fn test_parse_structural_tokens_only() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 4 },
        };

        let tokens = vec![
            SemanticTokenBuilder::indent(span.clone()),
            SemanticTokenBuilder::blank_line(span.clone()),
            SemanticTokenBuilder::dedent(span.clone()),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Test that the parser handles indentation level tracking
    #[test]
    fn test_indentation_level_tracking() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 4 },
        };

        let tokens = vec![
            SemanticTokenBuilder::indent(span.clone()),
            SemanticTokenBuilder::indent(span.clone()),
            SemanticTokenBuilder::dedent(span.clone()),
            SemanticTokenBuilder::dedent(span.clone()),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        // Parser should end with indentation level back to 0
        assert_eq!(parser.indentation_level, 0);
    }

    /// Test that the parser can parse annotation semantic tokens
    #[test]
    fn test_parse_annotation() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 20 },
        };

        let label_span = SourceSpan {
            start: Position { row: 1, column: 3 },
            end: Position { row: 1, column: 7 },
        };

        let content_span = SourceSpan {
            start: Position { row: 1, column: 11 },
            end: Position { row: 1, column: 20 },
        };

        // Create an annotation semantic token
        let annotation_token = SemanticTokenBuilder::annotation(
            SemanticTokenBuilder::label("note".to_string(), label_span),
            None, // No parameters
            Some(SemanticTokenBuilder::text_span(
                "This is a note".to_string(),
                content_span,
            )),
            span,
        );

        let tokens = vec![annotation_token];
        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            AstNode::Annotation(annotation_block) => {
                assert_eq!(annotation_block.label, "note");
                // TODO: Check annotation content when properly implemented
            }
            _ => panic!("Expected Annotation node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse definition semantic tokens
    #[test]
    fn test_parse_definition() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 15 },
        };

        let term_span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 12 },
        };

        // Create a definition semantic token
        let definition_token = SemanticTokenBuilder::definition(
            SemanticTokenBuilder::text_span("Term".to_string(), term_span),
            None, // No parameters
            span,
        );

        let tokens = vec![definition_token];
        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            AstNode::Definition(_definition_block) => {
                // TODO: Check definition term when properly implemented
                // TODO: Check definition parameters when properly implemented
            }
            _ => panic!("Expected Definition node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse paragraph semantic tokens
    #[test]
    fn test_parse_paragraph() {
        let mut parser = AstConstructor::new();

        let span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 12 },
        };

        let content_span = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 12 },
        };

        // Create a paragraph semantic token
        let paragraph_token = SemanticTokenBuilder::plain_text_line(
            SemanticTokenBuilder::text_span("Hello world".to_string(), content_span),
            span,
        );

        let tokens = vec![paragraph_token];
        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            AstNode::Paragraph(_paragraph_block) => {
                // TODO: Check paragraph content when properly implemented
            }
            _ => panic!("Expected Paragraph node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse session patterns
    #[test]
    fn test_parse_session() {
        let mut parser = AstConstructor::new();

        let span1 = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 0 },
        };

        let span2 = SourceSpan {
            start: Position { row: 2, column: 0 },
            end: Position { row: 2, column: 10 },
        };

        let span3 = SourceSpan {
            start: Position { row: 3, column: 0 },
            end: Position { row: 3, column: 0 },
        };

        let span4 = SourceSpan {
            start: Position { row: 4, column: 0 },
            end: Position { row: 4, column: 4 },
        };

        let span5 = SourceSpan {
            start: Position { row: 5, column: 4 },
            end: Position { row: 5, column: 20 },
        };

        // Create a session pattern: blank line + title + blank line + indent + content
        let tokens = vec![
            SemanticTokenBuilder::blank_line(span1),
            SemanticTokenBuilder::plain_text_line(
                SemanticTokenBuilder::text_span("Session Title".to_string(), span2.clone()),
                span2,
            ),
            SemanticTokenBuilder::blank_line(span3),
            SemanticTokenBuilder::indent(span4),
            SemanticTokenBuilder::plain_text_line(
                SemanticTokenBuilder::text_span("Session content".to_string(), span5.clone()),
                span5,
            ),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            AstNode::Session(_session_block) => {
                // TODO: Check session title when properly implemented
                // TODO: Check session child count when properly implemented
            }
            _ => panic!("Expected Session node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse list patterns
    #[test]
    fn test_parse_list() {
        let mut parser = AstConstructor::new();

        let span1 = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 10 },
        };

        let span2 = SourceSpan {
            start: Position { row: 2, column: 0 },
            end: Position { row: 2, column: 12 },
        };

        // Create a list pattern: two sequence text lines
        let tokens = vec![
            SemanticTokenBuilder::sequence_text_line(
                SemanticTokenBuilder::sequence_marker(
                    SemanticNumberingStyle::Plain,
                    SemanticNumberingForm::Regular,
                    "-".to_string(),
                    span1.clone(),
                ),
                SemanticTokenBuilder::text_span("First item".to_string(), span1.clone()),
                span1,
            ),
            SemanticTokenBuilder::sequence_text_line(
                SemanticTokenBuilder::sequence_marker(
                    SemanticNumberingStyle::Plain,
                    SemanticNumberingForm::Regular,
                    "-".to_string(),
                    span2.clone(),
                ),
                SemanticTokenBuilder::text_span("Second item".to_string(), span2.clone()),
                span2,
            ),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            AstNode::List(_list_block) => {
                // TODO: Check list item count when properly implemented
            }
            _ => panic!("Expected List node, got {:?}", ast_nodes[0]),
        }
    }

    /// Test that the parser can parse nested session patterns
    #[test]
    fn test_parse_nested_sessions() {
        let mut parser = AstConstructor::new();

        let span1 = SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 0 },
        };

        let span2 = SourceSpan {
            start: Position { row: 2, column: 0 },
            end: Position { row: 2, column: 15 },
        };

        let span3 = SourceSpan {
            start: Position { row: 3, column: 0 },
            end: Position { row: 3, column: 0 },
        };

        let span4 = SourceSpan {
            start: Position { row: 4, column: 0 },
            end: Position { row: 4, column: 4 },
        };

        let span5 = SourceSpan {
            start: Position { row: 5, column: 4 },
            end: Position { row: 5, column: 20 },
        };

        let span6 = SourceSpan {
            start: Position { row: 6, column: 4 },
            end: Position { row: 6, column: 0 },
        };

        let span7 = SourceSpan {
            start: Position { row: 7, column: 4 },
            end: Position { row: 7, column: 4 },
        };

        let _span8 = SourceSpan {
            start: Position { row: 8, column: 4 },
            end: Position { row: 8, column: 8 },
        };

        let span9 = SourceSpan {
            start: Position { row: 9, column: 8 },
            end: Position { row: 9, column: 25 },
        };

        let span10 = SourceSpan {
            start: Position { row: 10, column: 4 },
            end: Position { row: 10, column: 4 },
        };

        // Create a nested session pattern:
        // Blank line + "Outer Session" + blank line + indent +
        //   blank line + "Inner Session" + blank line + indent + "Content" + dedent + dedent
        let tokens = vec![
            SemanticTokenBuilder::blank_line(span1),
            SemanticTokenBuilder::plain_text_line(
                SemanticTokenBuilder::text_span("Outer Session".to_string(), span2.clone()),
                span2,
            ),
            SemanticTokenBuilder::blank_line(span3),
            SemanticTokenBuilder::indent(span4),
            SemanticTokenBuilder::blank_line(span6.clone()), // Add blank line before inner session
            SemanticTokenBuilder::plain_text_line(
                SemanticTokenBuilder::text_span("Inner Session".to_string(), span5.clone()),
                span5,
            ),
            SemanticTokenBuilder::blank_line(span6),
            SemanticTokenBuilder::indent(span7),
            SemanticTokenBuilder::plain_text_line(
                SemanticTokenBuilder::text_span("Nested content".to_string(), span9.clone()),
                span9,
            ),
            SemanticTokenBuilder::dedent(span10.clone()),
            SemanticTokenBuilder::dedent(span10),
        ];

        let semantic_tokens = SemanticTokenList::with_tokens(tokens);
        let result = parser.parse(&semantic_tokens);

        assert!(result.is_ok());
        let ast_nodes = result.unwrap();
        assert_eq!(ast_nodes.len(), 1);

        match &ast_nodes[0] {
            AstNode::Session(_session_block) => {
                // TODO: Check session title when properly implemented
                // TODO: Check session child count when properly implemented
            }
            _ => panic!("Expected Session node, got {:?}", ast_nodes[0]),
        }
    }
}
