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
use crate::cst::{HighLevelToken, HighLevelTokenList};
use crate::semantic::BlockParseError;

/// AST Construction parser for converting semantic tokens to AST nodes
///
/// This parser takes semantic tokens and transforms them into structured AST nodes
/// using precedence-based pattern matching.
pub struct AstConstructor<'a> {
    /// The semantic token stream being parsed
    tokens: &'a [HighLevelToken],
    /// Current parsing position in the token stream
    pub position: usize,
    /// Current indentation level for nested parsing
    pub indentation_level: usize,
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
    pub fn with_tokens(tokens: &'a [HighLevelToken]) -> Self {
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
        semantic_tokens: &'a HighLevelTokenList,
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
                HighLevelToken::Indent { .. } => {
                    self.indentation_level += 1;
                    self.position += 1;
                    continue;
                }
                HighLevelToken::Dedent { .. } => {
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
        if matches!(current_token, HighLevelToken::BlankLine { .. }) {
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
    fn peek(&self) -> Option<&HighLevelToken> {
        self.tokens.get(self.position)
    }

    /// Get current token and advance position
    #[allow(dead_code)] // Will be used in next steps
    fn consume(&mut self) -> Option<&HighLevelToken> {
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
            HighLevelToken::Annotation { .. } => {
                // Consume the token
                self.position += 1;

                // Delegate to annotation element constructor
                let annotation_block =
                    crate::semantic::elements::annotation::create_annotation_element(token)?;

                Ok(Some(AstNode::Annotation(annotation_block)))
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
            HighLevelToken::VerbatimBlock { .. } => {
                // Consume the token
                self.position += 1;

                // Delegate to verbatim element constructor
                let verbatim_block =
                    crate::semantic::elements::verbatim::create_verbatim_element(token)?;

                Ok(Some(AstNode::VerbatimBlock(verbatim_block)))
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
            HighLevelToken::Definition { .. } => {
                // Consume the token
                self.position += 1;

                // Delegate to definition element constructor
                let definition_block =
                    crate::semantic::elements::definition::create_definition_element(token)?;

                Ok(Some(AstNode::Definition(definition_block)))
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
            HighLevelToken::BlankLine { .. } => {
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
            HighLevelToken::PlainTextLine { content, .. } => match content.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            HighLevelToken::SequenceTextLine { content, .. } => match content.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
                _ => "unknown".to_string(),
            },
            HighLevelToken::Definition { term, .. } => match term.as_ref() {
                HighLevelToken::TextSpan { content, .. } => content.clone(),
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
            HighLevelToken::BlankLine { .. } => {
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
            HighLevelToken::Indent { .. } => {
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
                HighLevelToken::Dedent { .. } => {
                    self.position += 1; // Consume the dedent token
                    self.indentation_level -= 1; // Track indentation level
                    break;
                }
                HighLevelToken::BlankLine { .. } => {
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
        // Calculate how many tokens we consumed
        let tokens_consumed = self.position - start_position;

        // Get the title token (we need to go back to find it)
        let title_token_index = start_position + 1; // Skip blank line, get title
        let title_token = &self.tokens[title_token_index];

        // Delegate to session element constructor
        let session_block =
            crate::semantic::elements::session::create_session_element(title_token, &child_nodes)?;

        Ok(Some((AstNode::Session(session_block), tokens_consumed)))
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
                HighLevelToken::SequenceTextLine { .. } => {
                    item_count += 1;
                    self.position += 1; // Consume the sequence text line

                    // Parse any indented content that follows this list item
                    self.parse_list_item_content()?;
                }
                HighLevelToken::BlankLine { .. } => {
                    // Check if this blank line is between list items
                    if item_count > 0 {
                        // Look ahead to see if there's another sequence marker
                        let mut next_pos = self.position + 1;
                        while next_pos < self.tokens.len() {
                            let next_token = &self.tokens[next_pos];
                            match next_token {
                                HighLevelToken::BlankLine { .. } => {
                                    next_pos += 1;
                                    continue;
                                }
                                HighLevelToken::SequenceTextLine { .. } => {
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
        // Calculate how many tokens we consumed
        let tokens_consumed = self.position - start_position;

        // Delegate to list element constructor
        let list_block = crate::semantic::elements::list::create_list_element(item_count)?;

        Ok(Some((AstNode::List(list_block), tokens_consumed)))
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
            HighLevelToken::Indent { .. } => {
                self.position += 1; // Consume the indent token
                self.indentation_level += 1; // Track indentation level

                // Parse all indented content until we hit a dedent
                while self.position < self.tokens.len() {
                    let token = &self.tokens[self.position];
                    match token {
                        HighLevelToken::Dedent { .. } => {
                            self.position += 1; // Consume the dedent token
                            self.indentation_level -= 1; // Track indentation level
                            break;
                        }
                        HighLevelToken::BlankLine { .. } => {
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
            HighLevelToken::PlainTextLine { .. } => {
                // Consume the token
                self.position += 1;

                // Delegate to paragraph element constructor
                let paragraph_block =
                    crate::semantic::elements::paragraph::create_paragraph_element(token)?;

                Ok(Some(AstNode::Paragraph(paragraph_block)))
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
        semantic_tokens: &HighLevelTokenList,
    ) -> Result<Vec<crate::ast::elements::core::ElementNode>, BlockParseError> {
        let mut constructor = AstConstructor::new();
        let ast_nodes = constructor.parse(semantic_tokens)?;
        Ok(ast_nodes
            .into_iter()
            .map(|node| node.to_element_node())
            .collect())
    }
}
