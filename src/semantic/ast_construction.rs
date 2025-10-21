//! Phase 2b: AST Construction
//!
//! Converts semantic tokens into AST tree nodes using a regex-based grammar engine
//! with carefully planned precedence rules.
//!
//! ## Implementation Focus
//!
//! This phase focuses ONLY on the core three elements without explicit syntax markers:
//! - **Paragraph**: Catch-all for text lines
//! - **Session**: Blank-line enclosed titles with indented content
//! - **List**: Consecutive sequence markers (at least 2 items)
//!
//! These three elements are the source of all parsing difficulty in txxt due to their
//! lack of explicit syntax markers. We must master their interaction before adding
//! other elements (Definition, Annotation, Verbatim).
//!
//! ## Architecture
//!
//! The parser uses a regex-based pattern matching approach:
//! 1. Serialize token stream to string representation (e.g., "<BlankLine> <TextSpan>")
//! 2. Try grammar rules in precedence order
//! 3. On match, count capture groups to determine tokens consumed
//! 4. Delegate to element constructors in src/semantic/elements/
//! 5. Continue with remaining tokens
//!
//! See docs/proposals/regex-grammar-engine.txxt for complete design.

use crate::cst::{HighLevelToken, HighLevelTokenList};
use crate::semantic::BlockParseError;

/// Maximum recursion depth for nested structures to prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 100;

/// AST Construction parser for converting semantic tokens to AST nodes
///
/// This parser implements a regex-based grammar engine that matches token patterns
/// and delegates to element constructors for AST node creation.
pub struct AstConstructor<'a> {
    /// The semantic token stream being parsed
    tokens: &'a [HighLevelToken],
    /// Current parsing position in the token stream
    position: usize,
    /// Current recursion depth (for nested structures)
    recursion_depth: usize,
}

impl<'a> AstConstructor<'a> {
    /// Create a new AST constructor instance
    pub fn new() -> Self {
        Self {
            tokens: &[],
            position: 0,
            recursion_depth: 0,
        }
    }

    /// Create a new AST constructor instance with token stream
    pub fn with_tokens(tokens: &'a [HighLevelToken]) -> Self {
        Self {
            tokens,
            position: 0,
            recursion_depth: 0,
        }
    }

    /// Parse semantic tokens into AST nodes
    ///
    /// This is the main entry point for AST construction.
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
        self.tokens = &semantic_tokens.tokens;
        self.position = 0;

        let mut ast_nodes = Vec::new();

        // Main parsing loop - process tokens until we reach the end
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];

            // Try to match patterns in precedence order

            // Annotation pattern (standalone token, no indented content for now)
            // Pattern: <Annotation>
            if let Some(node) = self.try_parse_annotation()? {
                ast_nodes.push(node);
                continue;
            }

            // Verbatim block pattern (standalone token, content already parsed)
            // Pattern: <VerbatimBlock>
            if let Some(node) = self.try_parse_verbatim()? {
                ast_nodes.push(node);
                continue;
            }

            // Definition pattern (check before sessions as both can have similar structure)
            // Pattern: <Definition> <Indent> <Content>* <Dedent>
            if let Some((node, _tokens_consumed)) = self.try_parse_definition()? {
                ast_nodes.push(node);
                continue;
            }

            // Session pattern (higher precedence than list and paragraph)
            // Two variants:
            // 1. Start of document: <TitleLine> <BlankLine> <Indent>
            // 2. Mid-document: <BlankLine> <TitleLine> <BlankLine> <Indent>
            if let Some((node, _tokens_consumed)) = self.try_parse_session()? {
                ast_nodes.push(node);
                continue;
            }

            // List pattern (2+ consecutive SequenceTextLine, no blank lines)
            if let Some((node, _tokens_consumed)) = self.try_parse_list()? {
                ast_nodes.push(node);
                continue;
            }

            // Skip standalone blank lines (not part of session/list pattern)
            if matches!(token, HighLevelToken::BlankLine { .. }) {
                self.position += 1;
                continue;
            }

            // Paragraph pattern (catch-all for PlainTextLine)
            if let Some(node) = self.try_parse_paragraph()? {
                ast_nodes.push(node);
            } else {
                // No pattern matched - skip this token to avoid infinite loop
                self.position += 1;
            }
        }

        Ok(ast_nodes)
    }

    /// Try to parse a session pattern
    ///
    /// Sessions have two patterns:
    /// 1. Start of document: <TitleLine> <BlankLine> <Indent> <Content>* <Dedent>
    /// 2. Mid-document: <BlankLine> <TitleLine> <BlankLine> <Indent> <Content>* <Dedent>
    ///
    /// Returns: (SessionBlock, tokens_consumed) if matched, None otherwise
    fn try_parse_session(&mut self) -> Result<Option<(AstNode, usize)>, BlockParseError> {
        let start_pos = self.position;

        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let current_token = &self.tokens[self.position];

        // Determine which pattern we're trying to match
        let (title_offset, has_leading_blank) =
            if matches!(current_token, HighLevelToken::BlankLine { .. }) {
                // Pattern 2: Mid-document with leading blank line
                // Position 0: BlankLine, Position 1: Title
                (1, true)
            } else if matches!(
                current_token,
                HighLevelToken::PlainTextLine { .. } | HighLevelToken::SequenceTextLine { .. }
            ) {
                // Pattern 1: Start of document, no leading blank line
                // Position 0: Title
                (0, false)
            } else {
                return Ok(None);
            };

        // Check if we have enough tokens for the pattern
        let min_tokens = if has_leading_blank { 4 } else { 3 }; // BlankLine? + Title + BlankLine + Indent
        if self.position + min_tokens > self.tokens.len() {
            return Ok(None);
        }

        // Validate title token
        let title_pos = self.position + title_offset;
        let title_token = &self.tokens[title_pos];
        if !matches!(
            title_token,
            HighLevelToken::PlainTextLine { .. } | HighLevelToken::SequenceTextLine { .. }
        ) {
            return Ok(None);
        }

        // Check for blank line(s) after title (1 or more)
        let blank_after_title_pos = title_pos + 1;
        if !matches!(
            self.tokens[blank_after_title_pos],
            HighLevelToken::BlankLine { .. }
        ) {
            return Ok(None);
        }

        // Skip any additional blank lines (txxt allows multiple blanks)
        let mut indent_pos = blank_after_title_pos + 1;
        while indent_pos < self.tokens.len()
            && matches!(self.tokens[indent_pos], HighLevelToken::BlankLine { .. })
        {
            indent_pos += 1;
        }

        // Check for Indent token after blank line(s)
        if indent_pos >= self.tokens.len()
            || !matches!(self.tokens[indent_pos], HighLevelToken::Indent { .. })
        {
            return Ok(None);
        }

        // Pattern matched! Now consume tokens and build session
        if has_leading_blank {
            self.position += 1; // Skip leading BlankLine
        }

        // Clone/capture the title token before advancing position
        let title_token_clone = self.tokens[self.position].clone();
        self.position += 1; // Consume title

        // Skip all blank lines after title (we already validated there's at least one)
        while self.position < self.tokens.len()
            && matches!(self.tokens[self.position], HighLevelToken::BlankLine { .. })
        {
            self.position += 1;
        }

        self.position += 1; // Skip Indent

        // Now recursively parse the content until we hit Dedent
        let content_nodes = self.parse_until_dedent()?;

        // Consume the Dedent token
        if self.position < self.tokens.len()
            && matches!(self.tokens[self.position], HighLevelToken::Dedent { .. })
        {
            self.position += 1;
        }

        // Delegate to session element constructor
        let session_block = crate::semantic::elements::session::create_session_element(
            &title_token_clone,
            &content_nodes,
        )?;

        let tokens_consumed = self.position - start_pos;
        Ok(Some((AstNode::Session(session_block), tokens_consumed)))
    }

    /// Parse content tokens until we hit a Dedent token at the current nesting level
    ///
    /// This is used for recursive parsing of container content (sessions, definitions, etc.)
    /// It tracks Indent/Dedent nesting to ensure we stop at the correct Dedent.
    fn parse_until_dedent(&mut self) -> Result<Vec<AstNode>, BlockParseError> {
        // Check recursion depth to prevent stack overflow
        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            return Err(BlockParseError::InvalidStructure(format!(
                "Maximum nesting depth exceeded ({}). Document has too many nested structures.",
                MAX_RECURSION_DEPTH
            )));
        }

        let mut content_nodes = Vec::new();
        let mut indent_depth = 0; // Track nested indentation levels

        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];

            // Track Indent tokens to handle nested indentation (e.g., verbatim blocks, nested lists)
            if matches!(token, HighLevelToken::Indent { .. }) {
                indent_depth += 1;
                self.position += 1;
                continue;
            }

            // Stop at Dedent only if we're at the base level (no nested indents)
            if matches!(token, HighLevelToken::Dedent { .. }) {
                if indent_depth == 0 {
                    // This is the Dedent that closes our container
                    break;
                } else {
                    // This closes a nested indent - decrement and continue
                    indent_depth -= 1;
                    self.position += 1;
                    continue;
                }
            }

            // Try to match patterns in precedence order

            // Try annotation pattern first (standalone token)
            if let Some(node) = self.try_parse_annotation()? {
                content_nodes.push(node);
                continue;
            }

            // Try verbatim block pattern (standalone token, content already parsed)
            if let Some(node) = self.try_parse_verbatim()? {
                content_nodes.push(node);
                continue;
            }

            // Try definition pattern (explicit marker)
            if let Some((node, _tokens_consumed)) = self.try_parse_definition()? {
                content_nodes.push(node);
                continue;
            }

            // Try session pattern (for nested sessions)
            if let Some((node, _tokens_consumed)) = self.try_parse_session()? {
                content_nodes.push(node);
                continue;
            }

            // Try list pattern (2+ consecutive SequenceTextLine)
            if let Some((node, _tokens_consumed)) = self.try_parse_list()? {
                content_nodes.push(node);
                continue;
            }

            // Skip blank lines within content
            if matches!(token, HighLevelToken::BlankLine { .. }) {
                self.position += 1;
                continue;
            }

            // Try paragraph pattern (catch-all)
            if let Some(node) = self.try_parse_paragraph()? {
                content_nodes.push(node);
            } else {
                // No pattern matched - skip to avoid infinite loop
                self.position += 1;
            }
        }

        // Decrement recursion depth before returning
        self.recursion_depth -= 1;
        Ok(content_nodes)
    }

    /// Try to parse a definition pattern
    ///
    /// Definitions have the pattern:
    /// <Definition> <Indent> <Content>* <Dedent>
    ///
    /// Returns: (DefinitionBlock, tokens_consumed) if matched, None otherwise
    fn try_parse_definition(&mut self) -> Result<Option<(AstNode, usize)>, BlockParseError> {
        let start_pos = self.position;

        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        // Check if current token is a Definition
        let current_token = &self.tokens[self.position];
        if !matches!(current_token, HighLevelToken::Definition { .. }) {
            return Ok(None);
        }

        // Check if next token is Indent (definition must have indented content)
        if self.position + 1 >= self.tokens.len()
            || !matches!(
                self.tokens[self.position + 1],
                HighLevelToken::Indent { .. }
            )
        {
            return Ok(None);
        }

        // Clone the definition token before advancing position
        let definition_token_clone = self.tokens[self.position].clone();
        self.position += 1; // Consume definition token
        self.position += 1; // Skip Indent

        // Recursively parse the content until we hit Dedent
        let content_nodes = self.parse_until_dedent()?;

        // Consume the Dedent token
        if self.position < self.tokens.len()
            && matches!(self.tokens[self.position], HighLevelToken::Dedent { .. })
        {
            self.position += 1;
        }

        // Delegate to definition element constructor
        let definition_block = crate::semantic::elements::definition::create_definition_element(
            &definition_token_clone,
            &content_nodes,
        )?;

        let tokens_consumed = self.position - start_pos;
        Ok(Some((
            AstNode::Definition(definition_block),
            tokens_consumed,
        )))
    }

    /// Try to parse an annotation pattern
    ///
    /// Annotations can be standalone tokens or have indented content for nesting.
    ///
    /// Patterns:
    /// 1. `<Annotation>` (inline)
    /// 2. `<Annotation> <Indent> <Content>* <Dedent>` (with nested content)
    ///
    /// Returns: AnnotationBlock if matched, None otherwise
    fn try_parse_annotation(&mut self) -> Result<Option<AstNode>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        // Check if current token is an Annotation
        let token = &self.tokens[self.position];
        if !matches!(token, HighLevelToken::Annotation { .. }) {
            return Ok(None);
        }

        // Clone the annotation token before advancing position
        let annotation_token_clone = token.clone();
        self.position += 1; // Consume annotation token

        // Check for optional indented content
        let mut content_nodes = Vec::new();
        if self.position < self.tokens.len()
            && matches!(self.tokens[self.position], HighLevelToken::Indent { .. })
        {
            self.position += 1; // Consume Indent token

            // Recursively parse the content until we hit Dedent
            content_nodes = self.parse_until_dedent()?;

            // Consume the Dedent token
            if self.position < self.tokens.len()
                && matches!(self.tokens[self.position], HighLevelToken::Dedent { .. })
            {
                self.position += 1;
            }
        }

        // Delegate to annotation element constructor
        let annotation_block = crate::semantic::elements::annotation::create_annotation_element(
            &annotation_token_clone,
            &content_nodes,
        )?;

        Ok(Some(AstNode::Annotation(annotation_block)))
    }

    /// Try to parse a verbatim block pattern
    ///
    /// Verbatim blocks are standalone tokens - the scanner/tokenizer has already
    /// extracted the title, content, and label.
    ///
    /// Pattern: <VerbatimBlock>
    ///
    /// Returns: VerbatimBlock if matched, None otherwise
    fn try_parse_verbatim(&mut self) -> Result<Option<AstNode>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        // Check if current token is a VerbatimBlock
        let token = &self.tokens[self.position];
        if !matches!(token, HighLevelToken::VerbatimBlock { .. }) {
            return Ok(None);
        }

        // Clone the verbatim token before advancing position
        let verbatim_token_clone = token.clone();
        self.position += 1; // Consume verbatim token

        // Delegate to verbatim element constructor
        let verbatim_block =
            crate::semantic::elements::verbatim::create_verbatim_element(&verbatim_token_clone)?;

        Ok(Some(AstNode::Verbatim(verbatim_block)))
    }

    /// Try to parse a list pattern, supporting nested lists
    ///
    /// Lists are one or more consecutive SequenceTextLine tokens. Nesting is handled
    /// by Indent/Dedent tokens.
    ///
    /// # Arguments
    /// * `level` - The current indentation level for the list
    ///
    /// Try to parse a list pattern, supporting nested content within list items.
    ///
    /// Lists are 2+ consecutive SequenceTextLine tokens. Nesting is handled by
    /// parsing indented content after a list item.
    fn try_parse_list(&mut self) -> Result<Option<(AstNode, usize)>, BlockParseError> {
        let start_pos = self.position;
        if self.position >= self.tokens.len()
            || !matches!(
                self.tokens[self.position],
                HighLevelToken::SequenceTextLine { .. }
            )
        {
            return Ok(None);
        }

        let mut list_items_data = Vec::new();
        while self.position < self.tokens.len() {
            if let HighLevelToken::SequenceTextLine { .. } = self.tokens[self.position] {
                let item_token = self.tokens[self.position].clone();
                self.position += 1;

                let nested_content = if self.position < self.tokens.len()
                    && matches!(self.tokens[self.position], HighLevelToken::Indent { .. })
                {
                    self.position += 1; // Consume Indent
                    let content = self.parse_until_dedent()?;
                    if self.position < self.tokens.len()
                        && matches!(self.tokens[self.position], HighLevelToken::Dedent { .. })
                    {
                        self.position += 1; // Consume Dedent
                    }
                    content
                } else {
                    Vec::new()
                };
                list_items_data.push((item_token, nested_content));
            } else {
                break;
            }
        }

        if list_items_data.len() < 2 {
            self.position = start_pos;
            return Ok(None);
        }

        let list_block =
            crate::semantic::elements::list::create_list_element_with_nesting(&list_items_data)?;
        let tokens_consumed = self.position - start_pos;
        Ok(Some((AstNode::List(list_block), tokens_consumed)))
    }

    /// Try to parse a paragraph pattern
    ///
    /// Paragraphs are consecutive PlainTextLine tokens until a blank line or other element.
    ///
    /// Pattern: <PlainTextLine>+ (consecutive, no <BlankLine> between)
    ///
    /// Returns: ParagraphBlock if matched, None otherwise
    fn try_parse_paragraph(&mut self) -> Result<Option<AstNode>, BlockParseError> {
        if self.position >= self.tokens.len() {
            return Ok(None);
        }

        let token = &self.tokens[self.position];

        // Match PlainTextLine tokens
        if let HighLevelToken::PlainTextLine { .. } = token {
            // Collect consecutive PlainTextLine tokens (no blank lines allowed)
            let mut paragraph_lines = Vec::new();

            while self.position < self.tokens.len() {
                let token = &self.tokens[self.position];

                match token {
                    HighLevelToken::PlainTextLine { .. } => {
                        paragraph_lines.push(token.clone());
                        self.position += 1;
                    }
                    HighLevelToken::BlankLine { .. } => {
                        // Blank line terminates paragraph
                        break;
                    }
                    _ => {
                        // Any other token terminates paragraph
                        break;
                    }
                }
            }

            // Delegate to paragraph element constructor with all lines
            let paragraph_block =
                crate::semantic::elements::paragraph::create_paragraph_element_multi(
                    &paragraph_lines,
                )?;

            Ok(Some(AstNode::Paragraph(paragraph_block)))
        } else {
            Ok(None)
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
/// Currently supports: Paragraph, Session, List, Definition, Annotation.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Paragraph block node
    Paragraph(crate::ast::elements::paragraph::ParagraphBlock),
    /// Session block node
    Session(crate::ast::elements::session::SessionBlock),
    /// List block node
    List(crate::ast::elements::list::ListBlock),
    /// Definition block node
    Definition(crate::ast::elements::definition::DefinitionBlock),
    /// Annotation block node
    Annotation(crate::ast::elements::annotation::annotation_block::AnnotationBlock),
    /// Verbatim block node
    Verbatim(crate::ast::elements::verbatim::block::VerbatimBlock),
}

impl AstNode {
    /// Convert an AstNode to an ElementNode for integration with the parsing pipeline
    pub fn to_element_node(&self) -> crate::ast::elements::core::ElementNode {
        match self {
            AstNode::Paragraph(block) => {
                crate::ast::elements::core::ElementNode::ParagraphBlock(block.clone())
            }
            AstNode::Session(block) => {
                crate::ast::elements::core::ElementNode::SessionBlock(block.clone())
            }
            AstNode::List(block) => {
                crate::ast::elements::core::ElementNode::ListBlock(block.clone())
            }
            AstNode::Definition(block) => {
                crate::ast::elements::core::ElementNode::DefinitionBlock(block.clone())
            }
            AstNode::Annotation(block) => {
                crate::ast::elements::core::ElementNode::AnnotationBlock(block.clone())
            }
            AstNode::Verbatim(block) => {
                crate::ast::elements::core::ElementNode::VerbatimBlock(block.clone())
            }
        }
    }
}

impl AstConstructor<'_> {
    /// Parse semantic tokens and return ElementNodes for pipeline integration
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
