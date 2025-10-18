//! Phase 2b: AST Construction
//!
//! Converts semantic tokens into complete AST tree structure. This is the final step
//! of Phase 2 parsing, where we apply grammar rules to build the hierarchical
//! element tree from the flat stream of semantic tokens.
//!
//! # Overview
//!
//! AST construction transforms the flat stream of semantic tokens into a
//! complete Abstract Syntax Tree by applying the grammar rules from
//! docs/specs/core/grammar.txxt. This phase handles container nesting,
//! element recognition, and tree structure construction.
//!
//! # Key Responsibilities
//!
//! - **Element Recognition**: Apply grammar rules to identify block elements
//! - **Container Nesting**: Handle Indent/Dedent tokens for proper hierarchy
//! - **Tree Construction**: Build parent-child relationships correctly
//! - **Error Recovery**: Graceful handling of malformed input
//!
//! # Grammar Rule Application
//!
//! Block Element Patterns:
//! - Paragraph: PlainTextLine+
//! - List: SequenceTextLine{2,}  
//! - Session: (PlainTextLine | SequenceTextLine) + BlankLine+ + SessionContainer
//! - Definition: PlainTextLine + TxxtMarker + ContentContainer
//! - VerbatimBlock: PlainTextLine + Colon + IgnoreContainer + TxxtMarker + Label
//! - AnnotationBlock: TxxtMarker + Label + TxxtMarker + (ContentContainer | SessionContainer)
//!
//! # Input/Output
//!
//! - **Input**: `SemanticTokenList` from semantic analysis (Phase 2a)
//! - **Output**: Complete AST `ElementNode` tree structure

use std::collections::VecDeque;
use std::fmt;

use crate::ast::elements::components::parameters::Parameters;
use crate::ast::elements::containers::ContentContainer;
use crate::ast::elements::core::ElementNode;
use crate::ast::elements::definition::{DefinitionBlock, DefinitionTerm};
use crate::ast::elements::inlines::{TextSpan, TextTransform};
use crate::ast::elements::list::{
    ListBlock, ListDecorationType, ListItem, NumberingForm, NumberingStyle,
};
use crate::ast::elements::paragraph::ParagraphBlock;
use crate::ast::elements::session::{SessionBlock, SessionContainer, SessionTitle};
use crate::ast::scanner_tokens::ScannerTokenSequence;
use crate::ast::semantic_tokens::{SemanticToken, SemanticTokenList};

/// AST construction parser for converting semantic tokens to AST elements
///
/// This parser takes the flat stream of semantic tokens and applies grammar
/// rules to build a complete AST tree structure with proper nesting.
#[derive(Debug)]
pub struct AstConstructor {
    /// Stack of container elements for tracking nesting levels
    container_stack: Vec<ContainerContext>,
}

/// Context information for tracking container nesting
#[derive(Debug, Clone)]
struct ContainerContext {
    /// The container type (Session, Content, or Ignore)
    #[allow(dead_code)]
    container_type: ContainerType,
    /// Elements accumulated in this container
    elements: Vec<ElementNode>,
    /// Indentation level for this container
    #[allow(dead_code)]
    indent_level: usize,
    /// Current paragraph being built from text spans
    current_paragraph_text: Vec<String>,
    /// Session title if this is a session container
    current_session_title: Option<String>,
    /// Whether we're currently processing session content
    in_session_content: bool,
    /// List items being accumulated
    current_list_items: Vec<ListItem>,
    /// Current list item being built
    current_list_item_text: Vec<String>,
    /// Current list item marker
    current_list_item_marker: Option<String>,
    /// Whether we're currently processing definition content
    in_definition_content: bool,
    /// Index of the definition being populated (if in_definition_content is true)
    current_definition_index: Option<usize>,
}

/// Container type for parsing context
#[derive(Debug, Clone, PartialEq)]
enum ContainerType {
    /// Root document container
    Document,
    /// Session container (can contain sessions)
    #[allow(dead_code)]
    Session,
    /// Content container (cannot contain sessions)
    #[allow(dead_code)]
    Content,
    /// Ignore container (verbatim content only)
    #[allow(dead_code)]
    Ignore,
}

/// Error types for AST construction
#[derive(Debug, Clone, PartialEq)]
pub enum AstConstructionError {
    /// Empty token stream
    EmptyTokenStream,
    /// Unexpected token in current context
    UnexpectedToken(String),
    /// Malformed element structure
    MalformedElement(String),
    /// Container nesting error
    ContainerNestingError(String),
    /// Grammar rule violation
    GrammarViolation(String),
    /// JSON parsing error for semantic tokens
    JsonParseError(String),
}

impl fmt::Display for AstConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstConstructionError::EmptyTokenStream => write!(f, "Empty semantic token stream"),
            AstConstructionError::UnexpectedToken(msg) => write!(f, "Unexpected token: {}", msg),
            AstConstructionError::MalformedElement(msg) => write!(f, "Malformed element: {}", msg),
            AstConstructionError::ContainerNestingError(msg) => {
                write!(f, "Container nesting error: {}", msg)
            }
            AstConstructionError::GrammarViolation(msg) => write!(f, "Grammar violation: {}", msg),
            AstConstructionError::JsonParseError(msg) => write!(f, "JSON parse error: {}", msg),
        }
    }
}

impl std::error::Error for AstConstructionError {}

impl Default for AstConstructor {
    fn default() -> Self {
        Self::new()
    }
}

impl AstConstructor {
    /// Create a new AST constructor
    pub fn new() -> Self {
        Self {
            container_stack: Vec::new(),
        }
    }

    /// Construct AST from semantic token JSON string
    ///
    /// This is the main entry point that parses the JSON semantic tokens
    /// and builds the complete AST tree structure.
    ///
    /// # Arguments
    /// * `semantic_tokens_json` - JSON serialized semantic tokens from semantic analysis
    ///
    /// # Returns
    /// * `Result<ElementNode, AstConstructionError>` - Complete AST tree or error
    pub fn construct(
        &self,
        semantic_tokens_json: &str,
    ) -> Result<ElementNode, AstConstructionError> {
        // Parse JSON semantic tokens
        let semantic_tokens: SemanticTokenList = serde_json::from_str(semantic_tokens_json)
            .map_err(|e| AstConstructionError::JsonParseError(e.to_string()))?;

        if semantic_tokens.tokens.is_empty() {
            return Err(AstConstructionError::EmptyTokenStream);
        }

        // Apply grammar rules to build AST
        self.construct_from_tokens(semantic_tokens)
    }

    /// Construct AST from semantic token list
    ///
    /// Internal method that processes the semantic tokens and builds
    /// the AST tree using grammar rules and container management.
    fn construct_from_tokens(
        &self,
        semantic_tokens: SemanticTokenList,
    ) -> Result<ElementNode, AstConstructionError> {
        let mut constructor = AstConstructor::new();

        // Initialize with document root container
        constructor.container_stack.push(ContainerContext {
            container_type: ContainerType::Document,
            elements: Vec::new(),
            indent_level: 0,
            current_paragraph_text: Vec::new(),
            current_session_title: None,
            in_session_content: false,
            current_list_items: Vec::new(),
            current_list_item_text: Vec::new(),
            current_list_item_marker: None,
            in_definition_content: false,
            current_definition_index: None,
        });

        // Process tokens sequentially
        let mut tokens = VecDeque::from(semantic_tokens.tokens);

        while let Some(token) = tokens.pop_front() {
            constructor.process_token(token, &mut tokens)?;
        }

        // Build final result
        constructor.finalize_construction()
    }

    /// Process a single semantic token in the current context
    fn process_token(
        &mut self,
        token: SemanticToken,
        remaining_tokens: &mut VecDeque<SemanticToken>,
    ) -> Result<(), AstConstructionError> {
        match token {
            SemanticToken::Indent { .. } => {
                self.handle_indent()?;
            }
            SemanticToken::Dedent { .. } => {
                self.handle_dedent()?;
            }
            SemanticToken::BlankLine { .. } => {
                self.handle_blank_line()?;
            }
            SemanticToken::PlainTextLine { .. } => {
                self.handle_plain_text_line(token)?;
            }
            SemanticToken::SequenceTextLine { .. } => {
                self.handle_sequence_text_line(token)?;
            }
            SemanticToken::Annotation { .. } => {
                self.handle_annotation(token)?;
            }
            SemanticToken::Definition { .. } => {
                self.handle_definition(token)?;
            }
            SemanticToken::VerbatimBlock { .. } => {
                self.handle_verbatim_block(token)?;
            }
            SemanticToken::TextSpan { .. } => {
                // Check if this might be a session title pattern
                if self.might_be_session_title(&token, remaining_tokens) {
                    self.handle_potential_session_title(token, remaining_tokens)?;
                } else {
                    // Regular text span processing
                    self.handle_text_span(token)?;
                }
            }
            SemanticToken::SequenceMarker { .. } => {
                // Handle numbered session titles or list markers
                self.handle_sequence_marker(token, remaining_tokens)?;
            }
            SemanticToken::TxxtMarker { .. } => {
                // Handle definition markers (::)
                self.handle_txxt_marker(token, remaining_tokens)?;
            }
            _ => {
                // Other tokens are handled as part of composite elements
                return Err(AstConstructionError::UnexpectedToken(format!(
                    "{:?}",
                    token
                )));
            }
        }
        Ok(())
    }

    /// Check if this TextSpan might be a session title
    /// Pattern: TextSpan + "\n" + BlankLine + Indent
    fn might_be_session_title(
        &self,
        token: &SemanticToken,
        remaining_tokens: &VecDeque<SemanticToken>,
    ) -> bool {
        // Current token should be a non-newline TextSpan
        if let SemanticToken::TextSpan { content, .. } = token {
            if content == "\n" || content.trim().is_empty() {
                return false;
            }
        } else {
            return false;
        }

        // Check upcoming tokens for session pattern
        let mut iter = remaining_tokens.iter();

        // Next should be "\n" TextSpan
        if let Some(SemanticToken::TextSpan { content, .. }) = iter.next() {
            if content != "\n" {
                return false;
            }
        } else {
            return false;
        }

        // Then BlankLine
        if let Some(SemanticToken::BlankLine { .. }) = iter.next() {
            // Then Indent (indicates session content)
            if let Some(SemanticToken::Indent { .. }) = iter.next() {
                return true;
            }
        }

        false
    }

    /// Handle a potential session title and its following content
    fn handle_potential_session_title(
        &mut self,
        title_token: SemanticToken,
        remaining_tokens: &mut VecDeque<SemanticToken>,
    ) -> Result<(), AstConstructionError> {
        // Extract the session title
        let title = match title_token {
            SemanticToken::TextSpan { content, .. } => content,
            _ => {
                return Err(AstConstructionError::MalformedElement(
                    "Expected TextSpan for session title".to_string(),
                ))
            }
        };

        // Consume the pattern tokens: "\n" + BlankLine + Indent
        // "\n" TextSpan
        if let Some(SemanticToken::TextSpan { content, .. }) = remaining_tokens.pop_front() {
            if content != "\n" {
                return Err(AstConstructionError::MalformedElement(
                    "Expected newline after session title".to_string(),
                ));
            }
        }

        // BlankLine
        if let Some(SemanticToken::BlankLine { .. }) = remaining_tokens.pop_front() {
            // Good, blank line separator
        } else {
            return Err(AstConstructionError::MalformedElement(
                "Expected blank line after session title".to_string(),
            ));
        }

        // Indent
        if let Some(SemanticToken::Indent { .. }) = remaining_tokens.pop_front() {
            // Start of session content
        } else {
            return Err(AstConstructionError::MalformedElement(
                "Expected indent after session title".to_string(),
            ));
        }

        // Mark that we're in a session container
        if let Some(current_container) = self.container_stack.last_mut() {
            current_container.current_session_title = Some(title);
            current_container.in_session_content = true;
        }

        Ok(())
    }

    /// Handle indentation token (start of new container)
    fn handle_indent(&mut self) -> Result<(), AstConstructionError> {
        // Indent tokens are consumed as part of session detection
        // This method handles standalone indent tokens
        Ok(())
    }

    /// Handle dedentation token (end of container)
    fn handle_dedent(&mut self) -> Result<(), AstConstructionError> {
        // Finalize current paragraph and list if any
        self.finalize_current_paragraph()?;
        self.finalize_current_list()?;

        // If we're in definition content, stop processing definition content
        if let Some(current_container) = self.container_stack.last_mut() {
            if current_container.in_definition_content {
                current_container.in_definition_content = false;
                current_container.current_definition_index = None;
            }
        }

        // If we're in session content, finalize the session
        if let Some(current_container) = self.container_stack.last_mut() {
            if current_container.in_session_content
                && current_container.current_session_title.is_some()
            {
                self.finalize_current_session()?;
            }
        }

        Ok(())
    }

    /// Handle blank line (potential paragraph separator)
    fn handle_blank_line(&mut self) -> Result<(), AstConstructionError> {
        // Blank lines end the current paragraph and list
        self.finalize_current_paragraph()?;
        self.finalize_current_list()?;
        Ok(())
    }

    /// Handle individual text span tokens  
    fn handle_text_span(&mut self, token: SemanticToken) -> Result<(), AstConstructionError> {
        match token {
            SemanticToken::TextSpan { content, .. } => {
                if let Some(current_container) = self.container_stack.last_mut() {
                    // Skip empty content
                    if content.is_empty() {
                        return Ok(());
                    }

                    if content == "\n" {
                        // Newline within paragraph - convert to space according to spec
                        // Only add space if we have existing content and it doesn't already end with space
                        if !current_container.current_paragraph_text.is_empty() {
                            let empty_string = String::new();
                            let last_text = current_container
                                .current_paragraph_text
                                .last()
                                .unwrap_or(&empty_string);
                            if !last_text.ends_with(' ') {
                                current_container
                                    .current_paragraph_text
                                    .push(" ".to_string());
                            }
                        }
                    } else {
                        // Regular text content
                        current_container.current_paragraph_text.push(content);
                    }
                }
                Ok(())
            }
            _ => Err(AstConstructionError::MalformedElement(
                "Expected TextSpan token".to_string(),
            )),
        }
    }

    /// Finalize the current paragraph if there's accumulated text
    fn finalize_current_paragraph(&mut self) -> Result<(), AstConstructionError> {
        if let Some(current_container) = self.container_stack.last_mut() {
            if !current_container.current_paragraph_text.is_empty() {
                // Create paragraph from accumulated text
                let paragraph_text = current_container.current_paragraph_text.join("");

                // Convert text to inline elements - create a simple TextSpan
                let text_span = TextSpan::simple(&paragraph_text);
                let content = vec![TextTransform::Identity(text_span)];

                // Check if we're processing definition content
                if current_container.in_definition_content {
                    if let Some(definition_index) = current_container.current_definition_index {
                        // Add paragraph to the definition's content container
                        let paragraph = ParagraphBlock {
                            content,
                            annotations: Vec::new(),
                            parameters: Parameters::new(),
                            tokens: ScannerTokenSequence::new(),
                        };

                        // Get the definition and add the paragraph to its content
                        if let Some(ElementNode::DefinitionBlock(ref mut definition)) =
                            current_container.elements.get_mut(definition_index)
                        {
                            use crate::ast::elements::containers::content::ContentContainerElement;
                            definition
                                .content
                                .content
                                .push(ContentContainerElement::Paragraph(paragraph));
                        }
                    }
                } else {
                    // Regular paragraph creation
                    let paragraph = ParagraphBlock {
                        content,
                        annotations: Vec::new(),
                        parameters: Parameters::new(),
                        tokens: ScannerTokenSequence::new(),
                    };

                    current_container
                        .elements
                        .push(ElementNode::ParagraphBlock(paragraph));
                }

                current_container.current_paragraph_text.clear();
            }
        }
        Ok(())
    }

    /// Finalize the current session if we have a title and content
    fn finalize_current_session(&mut self) -> Result<(), AstConstructionError> {
        if let Some(current_container) = self.container_stack.last_mut() {
            if let Some(title) = current_container.current_session_title.take() {
                // Create session block from accumulated elements
                let session_content = current_container.elements.clone();

                // Convert elements to SessionContainerElement
                let session_elements: Vec<crate::ast::elements::session::session_container::SessionContainerElement> =
                    session_content.into_iter().filter_map(|elem| {
                        match elem {
                            ElementNode::ParagraphBlock(p) => Some(crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(p)),
                            ElementNode::SessionBlock(s) => Some(crate::ast::elements::session::session_container::SessionContainerElement::Session(s)),
                            ElementNode::ListBlock(l) => Some(crate::ast::elements::session::session_container::SessionContainerElement::List(l)),
                            // TODO: Add other element types as needed (Definition, VerbatimBlock, Annotation)
                            _ => None,
                        }
                    }).collect();

                // Create SessionTitle from string
                let session_title = SessionTitle {
                    content: vec![TextTransform::Identity(TextSpan::simple(&title))],
                    numbering: None,
                    tokens: ScannerTokenSequence::new(),
                };

                let session = SessionBlock {
                    title: session_title,
                    content: SessionContainer {
                        content: session_elements,
                        annotations: Vec::new(),
                        parameters: Parameters::new(),
                        tokens: ScannerTokenSequence::new(),
                    },
                    annotations: Vec::new(),
                    parameters: Parameters::new(),
                    tokens: ScannerTokenSequence::new(),
                };

                // Clear elements and add the session
                current_container.elements.clear();
                current_container
                    .elements
                    .push(ElementNode::SessionBlock(session));
                current_container.in_session_content = false;
            }
        }
        Ok(())
    }

    /// Handle plain text line (potential paragraph content)
    fn handle_plain_text_line(&mut self, token: SemanticToken) -> Result<(), AstConstructionError> {
        // Create a basic paragraph element for Phase 1 implementation
        let paragraph = self.create_paragraph_from_token(token)?;

        // Add to current container
        if let Some(current_container) = self.container_stack.last_mut() {
            current_container
                .elements
                .push(ElementNode::ParagraphBlock(paragraph));
        }

        Ok(())
    }

    /// Handle sequence text line (potential list item)
    fn handle_sequence_text_line(
        &mut self,
        _token: SemanticToken,
    ) -> Result<(), AstConstructionError> {
        // TODO: Implement list detection and creation
        // For Phase 1, we'll stub this out
        Ok(())
    }

    /// Handle annotation (annotation block)
    fn handle_annotation(&mut self, _token: SemanticToken) -> Result<(), AstConstructionError> {
        // TODO: Implement annotation block creation
        // For Phase 1, we'll stub this out
        Ok(())
    }

    /// Handle definition (definition block)
    fn handle_definition(&mut self, token: SemanticToken) -> Result<(), AstConstructionError> {
        let definition = self.create_definition_from_token(token)?;

        // Add to current container
        if let Some(current_container) = self.container_stack.last_mut() {
            current_container
                .elements
                .push(ElementNode::DefinitionBlock(definition));
        }

        Ok(())
    }

    /// Handle verbatim block
    fn handle_verbatim_block(&mut self, _token: SemanticToken) -> Result<(), AstConstructionError> {
        // TODO: Implement verbatim block creation
        // For Phase 1, we'll stub this out
        Ok(())
    }

    /// Handle sequence marker (numbered session titles or list items)
    fn handle_sequence_marker(
        &mut self,
        marker_token: SemanticToken,
        remaining_tokens: &mut VecDeque<SemanticToken>,
    ) -> Result<(), AstConstructionError> {
        // For Phase 1: Check if this is a numbered session title pattern
        // Pattern: SequenceMarker + TextSpan (title) + "\n" + BlankLine + Indent

        // Look ahead to see if this looks like a session title
        let mut iter = remaining_tokens.iter();

        // Next should be a TextSpan (the session title text)
        if let Some(SemanticToken::TextSpan {
            content: _title_content,
            ..
        }) = iter.next()
        {
            // Then "\n"
            if let Some(SemanticToken::TextSpan {
                content: newline, ..
            }) = iter.next()
            {
                if newline == "\n" {
                    // Then BlankLine + Indent indicates a session
                    if let Some(SemanticToken::BlankLine { .. }) = iter.next() {
                        if let Some(SemanticToken::Indent { .. }) = iter.next() {
                            // This is a numbered session title - handle it
                            return self
                                .handle_numbered_session_title(marker_token, remaining_tokens);
                        }
                    }
                }
            }
        }

        // If not a session, this is likely a list item
        // Pattern: SequenceMarker + TextSpan content (no BlankLine + Indent after)
        self.handle_list_item_marker(marker_token, remaining_tokens)
    }

    /// Handle a numbered session title pattern
    fn handle_numbered_session_title(
        &mut self,
        marker_token: SemanticToken,
        remaining_tokens: &mut VecDeque<SemanticToken>,
    ) -> Result<(), AstConstructionError> {
        // Extract the sequence marker
        let marker_text = match marker_token {
            SemanticToken::SequenceMarker { marker, .. } => marker,
            _ => {
                return Err(AstConstructionError::MalformedElement(
                    "Expected SequenceMarker".to_string(),
                ))
            }
        };

        // Collect all TextSpan tokens until we hit "\n"
        let mut title_parts = vec![marker_text];

        while let Some(token) = remaining_tokens.front() {
            match token {
                SemanticToken::TextSpan { content, .. } => {
                    if content == "\n" {
                        // End of title, consume the newline and break
                        remaining_tokens.pop_front();
                        break;
                    } else {
                        // Part of the title, consume and add to title
                        if let Some(SemanticToken::TextSpan { content, .. }) =
                            remaining_tokens.pop_front()
                        {
                            title_parts.push(content);
                        }
                    }
                }
                _ => {
                    // Hit a non-TextSpan, this shouldn't happen in a title
                    return Err(AstConstructionError::MalformedElement(
                        "Unexpected token in session title".to_string(),
                    ));
                }
            }
        }

        let title_text = title_parts.join("");

        // Consume the pattern tokens: BlankLine + Indent
        // BlankLine
        if let Some(SemanticToken::BlankLine { .. }) = remaining_tokens.pop_front() {
            // Good, blank line separator
        } else {
            return Err(AstConstructionError::MalformedElement(
                "Expected blank line after numbered session title".to_string(),
            ));
        }

        // Indent
        if let Some(SemanticToken::Indent { .. }) = remaining_tokens.pop_front() {
            // Start of session content
        } else {
            return Err(AstConstructionError::MalformedElement(
                "Expected indent after numbered session title".to_string(),
            ));
        }

        // Mark that we're in a session container with numbered title
        if let Some(current_container) = self.container_stack.last_mut() {
            current_container.current_session_title = Some(title_text);
            current_container.in_session_content = true;
        }

        Ok(())
    }

    /// Handle a list item marker (-, *, 1., etc.)
    fn handle_list_item_marker(
        &mut self,
        marker_token: SemanticToken,
        remaining_tokens: &mut VecDeque<SemanticToken>,
    ) -> Result<(), AstConstructionError> {
        // Extract the list marker
        let marker_text = match marker_token {
            SemanticToken::SequenceMarker { marker, .. } => marker,
            _ => {
                return Err(AstConstructionError::MalformedElement(
                    "Expected SequenceMarker for list item".to_string(),
                ))
            }
        };

        // Finalize any previous list item
        self.finalize_current_list_item()?;

        // Start a new list item
        if let Some(current_container) = self.container_stack.last_mut() {
            current_container.current_list_item_marker = Some(marker_text);
            current_container.current_list_item_text.clear();
        }

        // Collect the list item content until end of line
        while let Some(token) = remaining_tokens.front() {
            match token {
                SemanticToken::TextSpan { content, .. } => {
                    if content == "\n" {
                        // End of list item content
                        remaining_tokens.pop_front(); // consume the newline
                        break;
                    } else {
                        // Part of the list item content
                        if let Some(SemanticToken::TextSpan { content, .. }) =
                            remaining_tokens.pop_front()
                        {
                            if let Some(current_container) = self.container_stack.last_mut() {
                                current_container.current_list_item_text.push(content);
                            }
                        }
                    }
                }
                _ => {
                    // Hit a non-TextSpan, end of list item
                    break;
                }
            }
        }

        Ok(())
    }

    /// Finalize the current list item if there's one being built
    fn finalize_current_list_item(&mut self) -> Result<(), AstConstructionError> {
        if let Some(current_container) = self.container_stack.last_mut() {
            if let Some(marker) = current_container.current_list_item_marker.take() {
                if !current_container.current_list_item_text.is_empty() {
                    // Create list item content
                    let content_text = current_container.current_list_item_text.join("");
                    let content = vec![TextTransform::Identity(TextSpan::simple(&content_text))];

                    let list_item = ListItem {
                        marker,
                        content,
                        nested: None,
                        annotations: Vec::new(),
                        parameters: Parameters::new(),
                        tokens: ScannerTokenSequence::new(),
                    };

                    current_container.current_list_items.push(list_item);
                    current_container.current_list_item_text.clear();
                }
            }
        }
        Ok(())
    }

    /// Finalize the current list if there are accumulated list items
    fn finalize_current_list(&mut self) -> Result<(), AstConstructionError> {
        // First finalize any current list item
        self.finalize_current_list_item()?;

        // Then check if we have list items to finalize
        if let Some(current_container) = self.container_stack.last_mut() {
            if !current_container.current_list_items.is_empty() {
                // Determine list decoration type from first item
                let first_marker = &current_container.current_list_items[0].marker;
                let decoration_type = ListDecorationType {
                    style: if first_marker == "-" || first_marker == "*" {
                        NumberingStyle::Plain
                    } else if first_marker.chars().next().unwrap_or('0').is_ascii_digit() {
                        NumberingStyle::Numerical
                    } else {
                        NumberingStyle::Alphabetical
                    },
                    form: NumberingForm::Short, // For Phase 2, assume short form
                };

                let list_block = ListBlock {
                    decoration_type,
                    items: current_container.current_list_items.clone(),
                    annotations: Vec::new(),
                    parameters: Parameters::new(),
                    tokens: ScannerTokenSequence::new(),
                };

                current_container
                    .elements
                    .push(ElementNode::ListBlock(list_block));
                current_container.current_list_items.clear();
            }
        }
        Ok(())
    }

    /// Handle TxxtMarker token (:: for definitions)
    fn handle_txxt_marker(
        &mut self,
        _marker_token: SemanticToken,
        remaining_tokens: &mut VecDeque<SemanticToken>,
    ) -> Result<(), AstConstructionError> {
        // Check if this TxxtMarker represents a definition pattern
        // Pattern: Text + Whitespace + TxxtMarker + Newline + Indent
        // This is more specific than just Text + :: to avoid conflicts with annotations
        
        let term_text = if let Some(current_container) = self.container_stack.last() {
            if !current_container.current_paragraph_text.is_empty() {
                Some(current_container.current_paragraph_text.join(""))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(term_text) = term_text {
            if !term_text.trim().is_empty() {
                // Check if this looks like a definition by looking ahead for the pattern:
                // TxxtMarker + TextSpan("\n") + Indent
                let mut iter = remaining_tokens.iter();
                let is_definition_pattern = if let Some(SemanticToken::TextSpan { content, .. }) = iter.next() {
                    if content == "\n" {
                        // Next should be Indent for definition content
                        matches!(iter.next(), Some(SemanticToken::Indent { .. }))
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_definition_pattern {
                    // This looks like a definition pattern - create definition
                    let definition = self.create_definition_from_pattern(term_text.trim())?;

                    if let Some(current_container) = self.container_stack.last_mut() {
                        let definition_index = current_container.elements.len();
                        current_container
                            .elements
                            .push(ElementNode::DefinitionBlock(definition));
                        current_container.current_paragraph_text.clear();

                        // Mark that we're now processing definition content
                        current_container.in_definition_content = true;
                        current_container.current_definition_index = Some(definition_index);
                    }

                    return Ok(());
                }
            }
        }

        // If not a definition pattern, treat as regular content
        // Add the :: marker to current paragraph text
        if let Some(current_container) = self.container_stack.last_mut() {
            current_container
                .current_paragraph_text
                .push("::".to_string());
        }

        Ok(())
    }

    /// Create a paragraph block from a plain text line token
    fn create_paragraph_from_token(
        &self,
        token: SemanticToken,
    ) -> Result<ParagraphBlock, AstConstructionError> {
        match token {
            SemanticToken::PlainTextLine { content, span: _ } => {
                // Extract text content from the nested TextSpan
                let text_content = match *content {
                    SemanticToken::TextSpan { content, .. } => content,
                    _ => {
                        return Err(AstConstructionError::MalformedElement(
                            "PlainTextLine must contain TextSpan".to_string(),
                        ))
                    }
                };

                // Convert text to inline elements - create a simple TextSpan
                let text_span = TextSpan::simple(&text_content);
                let content = vec![TextTransform::Identity(text_span)];

                Ok(ParagraphBlock {
                    content,
                    annotations: Vec::new(),
                    parameters: Parameters::new(),
                    tokens: ScannerTokenSequence::new(),
                })
            }
            _ => Err(AstConstructionError::MalformedElement(
                "Expected PlainTextLine token".to_string(),
            )),
        }
    }

    /// Create a definition block from a definition semantic token
    fn create_definition_from_token(
        &self,
        token: SemanticToken,
    ) -> Result<DefinitionBlock, AstConstructionError> {
        match token {
            SemanticToken::Definition {
                term,
                parameters: _,
                span: _,
            } => {
                // Extract term content from the nested semantic token
                let term_content = match *term {
                    SemanticToken::TextSpan { content, .. } => content,
                    _ => {
                        return Err(AstConstructionError::MalformedElement(
                            "Definition term must contain TextSpan".to_string(),
                        ))
                    }
                };

                // Create definition term - convert text to inline elements
                let term_text_span = TextSpan::simple(&term_content);
                let term_inline = vec![TextTransform::Identity(term_text_span)];

                let definition_term = DefinitionTerm {
                    content: term_inline,
                    tokens: ScannerTokenSequence::new(),
                };

                // Create empty content container for now - will be populated by indented content parsing
                let content_container = ContentContainer {
                    content: vec![], // Will be populated when we process the indented definition content
                    annotations: Vec::new(),
                    parameters: Parameters::new(),
                    tokens: ScannerTokenSequence::new(),
                };

                Ok(DefinitionBlock {
                    term: definition_term,
                    content: content_container,
                    parameters: Parameters::new(),
                    annotations: Vec::new(),
                    tokens: ScannerTokenSequence::new(),
                })
            }
            _ => Err(AstConstructionError::MalformedElement(
                "Expected Definition token".to_string(),
            )),
        }
    }

    /// Create a definition block from a term pattern (Text + :: pattern recognition)
    fn create_definition_from_pattern(
        &self,
        term_text: &str,
    ) -> Result<DefinitionBlock, AstConstructionError> {
        // Create definition term from the recognized text pattern
        let term_text_span = TextSpan::simple(term_text);
        let term_inline = vec![TextTransform::Identity(term_text_span)];

        let definition_term = DefinitionTerm {
            content: term_inline,
            tokens: ScannerTokenSequence::new(),
        };

        // Create empty content container for now - will be populated by indented content parsing
        let content_container = ContentContainer {
            content: vec![], // Will be populated when we process the indented definition content
            annotations: Vec::new(),
            parameters: Parameters::new(),
            tokens: ScannerTokenSequence::new(),
        };

        Ok(DefinitionBlock {
            term: definition_term,
            content: content_container,
            parameters: Parameters::new(),
            annotations: Vec::new(),
            tokens: ScannerTokenSequence::new(),
        })
    }

    /// Finalize construction and return the root element
    fn finalize_construction(mut self) -> Result<ElementNode, AstConstructionError> {
        if self.container_stack.is_empty() {
            return Err(AstConstructionError::ContainerNestingError(
                "No container stack available".to_string(),
            ));
        }

        // Finalize any remaining paragraph and list
        self.finalize_current_paragraph()?;
        self.finalize_current_list()?;

        // Get the document root container
        let root_container = self.container_stack.pop().unwrap();

        if root_container.elements.is_empty() {
            return Err(AstConstructionError::EmptyTokenStream);
        }

        // For Phase 1: If we have a single element, return it directly
        // For multiple elements, wrap in a SessionContainer
        if root_container.elements.len() == 1 {
            Ok(root_container.elements.into_iter().next().unwrap())
        } else {
            // Create a session container to hold multiple elements
            let session_container = SessionContainer {
                content: root_container.elements.into_iter().filter_map(|elem| {
                    match elem {
                        ElementNode::ParagraphBlock(p) => Some(crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(p)),
                        ElementNode::SessionBlock(s) => Some(crate::ast::elements::session::session_container::SessionContainerElement::Session(s)),
                        ElementNode::ListBlock(l) => Some(crate::ast::elements::session::session_container::SessionContainerElement::List(l)),
                        // TODO: Add other element types as needed (Definition, VerbatimBlock, Annotation)
                        _ => None,
                    }
                }).collect(),
                annotations: Vec::new(),
                parameters: Parameters::new(),
                tokens: ScannerTokenSequence::new(),
            };

            Ok(ElementNode::SessionContainer(session_container))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_constructor_creation() {
        let constructor = AstConstructor::new();
        assert!(constructor.container_stack.is_empty());
    }

    #[test]
    fn test_empty_token_stream_error() {
        let constructor = AstConstructor::new();
        let empty_tokens = SemanticTokenList::new();
        let empty_json = serde_json::to_string(&empty_tokens).unwrap();

        let result = constructor.construct(&empty_json);
        assert!(matches!(
            result,
            Err(AstConstructionError::EmptyTokenStream)
        ));
    }

    #[test]
    fn test_invalid_json_error() {
        let constructor = AstConstructor::new();
        let result = constructor.construct("invalid json");

        assert!(matches!(
            result,
            Err(AstConstructionError::JsonParseError(_))
        ));
    }
}
