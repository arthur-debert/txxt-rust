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
            let element_groups = self.split_tokens_into_elements(&block.tokens);
            for element_tokens in element_groups {
                let token_refs: Vec<&Token> = element_tokens.to_vec();
                if let Some(element) = self.parse_element_refs(&token_refs) {
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
        self.group_list_items(elements)
    }

    /// Parse a single TokenBlock into an AstNode
    #[allow(clippy::only_used_in_recursion)]
    #[allow(dead_code)]
    fn parse_block(&self, block: &TokenBlock) -> AstNode {
        // First try to parse the tokens as a specific element
        if let Some(element) = self.parse_element(&block.tokens) {
            // If this block has children, we need to handle them based on the element type
            if !block.children.is_empty() {
                let mut elements = vec![element];
                // Use SessionContainer context for recursive block parsing (allows sessions)
                self.connect_child_blocks_to_elements(&mut elements, &block.children, true);
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

    /// Parse a list of token references into a specific element type
    fn parse_element_refs(&self, tokens: &[&Token]) -> Option<AstNode> {
        self.parse_element_internal(tokens)
    }

    /// Parse a list of tokens into a specific element type
    #[allow(dead_code)]
    fn parse_element(&self, tokens: &[Token]) -> Option<AstNode> {
        let token_refs: Vec<&Token> = tokens.iter().collect();
        self.parse_element_internal(&token_refs)
    }

    /// Internal implementation for parsing elements
    fn parse_element_internal(&self, tokens: &[&Token]) -> Option<AstNode> {
        if tokens.is_empty() {
            return None;
        }

        // Skip EOF tokens for analysis
        let tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| t.token_type != TokenType::Eof)
            .cloned()
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
        if tokens.len() < 2 {
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

            // Extract title (text before VerbatimStart)
            if let Some(title) = self.extract_verbatim_title(tokens) {
                node.set_attribute("title".to_string(), title);
            }

            // Extract content (preserve exactly as tokenized - do NOT parse as TXXT)
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

            // Extract content (skip the marker, but only until next list item or element boundary)
            let mut content_tokens = Vec::new();
            let mut i = 1; // Skip the marker
            while i < tokens.len() {
                let token = tokens[i];

                // Stop if we encounter another list marker or element starter
                if matches!(
                    token.token_type,
                    TokenType::Dash | TokenType::SequenceMarker
                ) || token.token_type == TokenType::PragmaMarker
                    || token.token_type == TokenType::VerbatimStart
                    || token.token_type == TokenType::BlankLine
                    || (token.token_type == TokenType::Text && {
                        let remaining_tokens: Vec<Token> =
                            tokens[i..].iter().map(|&t| t.clone()).collect();
                        self.looks_like_definition_start(&remaining_tokens)
                    })
                {
                    break;
                }

                content_tokens.push(token);
                i += 1;
            }

            let content = self
                .extract_text_with_inlines(&content_tokens)
                .trim()
                .to_string();
            if !content.is_empty() {
                node.content = Some(content);
            }

            // Set location (from first token to last consumed token)
            let last_token_index = if content_tokens.is_empty() { 0 } else { i - 1 };
            if let (Some(first), Some(last)) = (tokens.first(), tokens.get(last_token_index)) {
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

    /// Extract verbatim title (text before VerbatimStart)
    fn extract_verbatim_title(&self, tokens: &[&Token]) -> Option<String> {
        let mut title_tokens = Vec::new();

        for token in tokens {
            if token.token_type == TokenType::VerbatimStart {
                break;
            }
            if matches!(token.token_type, TokenType::Text | TokenType::Identifier) {
                title_tokens.push(*token);
            }
        }

        if title_tokens.is_empty() {
            None
        } else {
            let title = self.extract_text(&title_tokens).trim().to_string();
            if title.is_empty() {
                None
            } else {
                Some(title)
            }
        }
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

    /// Split a token sequence into separate element groups
    fn split_tokens_into_elements<'a>(&self, tokens: &'a [Token]) -> Vec<Vec<&'a Token>> {
        let mut result = Vec::new();
        let mut current_group = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];

            // Skip EOF tokens
            if token.token_type == TokenType::Eof {
                i += 1;
                continue;
            }

            // Check if this token starts a new element
            let starts_new_element = match token.token_type {
                // Annotations start with ::
                TokenType::PragmaMarker => {
                    // Look ahead to see if this is an annotation pattern
                    self.looks_like_annotation_start(&tokens[i..])
                }
                // Definitions have text followed by ::
                // Verbatim blocks have text followed by :
                TokenType::Text => {
                    // Look ahead to see if this is a definition pattern or verbatim pattern
                    // But don't start a new element if we're already inside a verbatim block
                    if self.is_inside_verbatim_block(&current_group) {
                        false
                    } else {
                        self.looks_like_definition_start(&tokens[i..])
                            || self.looks_like_verbatim_start(&tokens[i..])
                    }
                }
                // List items start with - or numbers
                TokenType::Dash | TokenType::SequenceMarker => {
                    // Don't start new element if inside verbatim
                    !self.is_inside_verbatim_block(&current_group)
                }
                // Verbatim blocks (but only if not preceded by text on same line - that would be a title)
                TokenType::VerbatimStart => {
                    // Check if this VerbatimStart is preceded by text on the same line
                    // If so, it's part of a verbatim title and shouldn't start a new element
                    !self.verbatim_start_has_title(&tokens[..i + 1])
                }
                // Verbatim content should never start a new element
                TokenType::VerbatimContent | TokenType::VerbatimEnd => false,
                // Blank lines are individual elements (unless inside verbatim)
                TokenType::BlankLine => !self.is_inside_verbatim_block(&current_group),
                _ => false,
            };

            // If starting a new element and we have a current group, save it
            if starts_new_element && !current_group.is_empty() {
                result.push(current_group);
                current_group = Vec::new();
            }

            current_group.push(token);
            i += 1;
        }

        // Add the last group if it has content
        if !current_group.is_empty() {
            result.push(current_group);
        }

        result
    }

    /// Check if tokens look like an annotation start: :: label ::
    fn looks_like_annotation_start(&self, tokens: &[Token]) -> bool {
        if tokens.len() < 3 {
            return false;
        }

        // Must start with ::
        if tokens[0].token_type != TokenType::PragmaMarker {
            return false;
        }

        // Look for pattern :: identifier/text ::
        let mut pragma_count = 0;
        for token in tokens {
            if token.token_type == TokenType::PragmaMarker {
                pragma_count += 1;
                if pragma_count >= 2 {
                    return true;
                }
            } else if matches!(token.token_type, TokenType::Identifier | TokenType::Text)
                && pragma_count == 1
            {
                // Found content between pragmas, continue looking for second ::
                continue;
            } else if token.token_type == TokenType::Newline {
                // Newline before second :: means this might be multiline
                return pragma_count >= 2;
            }
        }

        false
    }

    /// Check if tokens look like a definition start: term ::
    fn looks_like_definition_start(&self, tokens: &[Token]) -> bool {
        if tokens.len() < 2 {
            return false;
        }

        // Look for :: somewhere in the tokens
        for token in tokens {
            if token.token_type == TokenType::DefinitionMarker {
                return true;
            }
            // Stop looking if we hit a newline without finding ::
            if token.token_type == TokenType::Newline {
                break;
            }
        }

        false
    }

    /// Check if tokens look like a verbatim start: title :
    fn looks_like_verbatim_start(&self, tokens: &[Token]) -> bool {
        if tokens.len() < 2 {
            return false;
        }

        // Look for VerbatimStart (:) after text
        for token in tokens {
            if token.token_type == TokenType::VerbatimStart {
                return true;
            }
            // Stop looking if we hit a newline without finding :
            if token.token_type == TokenType::Newline {
                break;
            }
        }

        false
    }

    /// Check if a VerbatimStart token has a title (text preceding it on the same line)
    fn verbatim_start_has_title(&self, tokens_up_to_start: &[Token]) -> bool {
        if tokens_up_to_start.is_empty() {
            return false;
        }

        // Look backward from the VerbatimStart to see if there's text without a newline
        for token in tokens_up_to_start.iter().rev() {
            if token.token_type == TokenType::VerbatimStart {
                continue; // Skip the VerbatimStart itself
            }
            if matches!(token.token_type, TokenType::Text | TokenType::Identifier) {
                return true; // Found text before the VerbatimStart
            }
            if token.token_type == TokenType::Newline {
                return false; // Hit a newline, so no title on the same line
            }
        }

        false
    }

    /// Check if we're currently inside a verbatim block (started but not finished)
    fn is_inside_verbatim_block(&self, current_group: &[&Token]) -> bool {
        let mut verbatim_start_count = 0;
        let mut verbatim_end_count = 0;

        for token in current_group {
            if token.token_type == TokenType::VerbatimStart {
                verbatim_start_count += 1;
            } else if token.token_type == TokenType::VerbatimEnd {
                verbatim_end_count += 1;
            }
        }

        // We're inside a verbatim block if we've seen a start but not completed the end pair
        verbatim_start_count > 0 && verbatim_end_count < 2
    }

    /// Group consecutive list items into list elements
    ///
    /// Takes a flat sequence of elements and groups consecutive list_item elements
    /// into list containers. This implements flat list parsing.
    fn group_list_items(&self, elements: Vec<AstNode>) -> Vec<AstNode> {
        let mut result = Vec::new();
        let mut current_list: Option<AstNode> = None;

        for element in elements {
            if element.node_type == "list_item" {
                // This is a list item - add it to current list or start a new one
                if let Some(ref mut list) = current_list {
                    // Add to existing list
                    list.add_child(element);
                } else {
                    // Start a new list
                    let mut new_list = AstNode::new("list".to_string());

                    // Set location based on first list item
                    if let (Some(start), Some(end)) = (element.start_line, element.end_line) {
                        new_list.set_location(start, end);
                    }

                    new_list.add_child(element);
                    current_list = Some(new_list);
                }
            } else {
                // Not a list item - finish current list if any, then add this element
                if let Some(list) = current_list.take() {
                    result.push(list);
                }
                result.push(element);
            }
        }

        // Don't forget the final list if we ended with list items
        if let Some(list) = current_list {
            result.push(list);
        }

        result
    }

    /// Connect child blocks to their appropriate parent elements
    /// is_session_container determines whether we create SessionContainers or ContentContainers
    #[allow(dead_code)]
    fn connect_child_blocks_to_elements(
        &self,
        elements: &mut [AstNode],
        child_blocks: &[TokenBlock],
        is_session_container: bool,
    ) {
        for child_block in child_blocks {
            if let Some(child_start_line) = child_block.start_line {
                // Find the element that should own this child block
                // This is the most recent element that ends just before the child block starts
                let mut target_element_index = None;
                let mut best_end_line = 0;
                for (i, element) in elements.iter().enumerate() {
                    if let Some(element_end_line) = element.end_line {
                        if element_end_line < child_start_line && element_end_line > best_end_line {
                            target_element_index = Some(i);
                            best_end_line = element_end_line;
                        }
                    }
                }

                // If we found a target element, check if it should be a session or content container
                if let Some(i) = target_element_index {
                    // Check if this might be a session (only in session containers)
                    let should_be_session = if is_session_container {
                        self.is_session_title(&elements[i], child_block)
                    } else {
                        false
                    };

                    if should_be_session {
                        // Convert the element to a session and add a session container
                        let mut session = AstNode::new("session".to_string());

                        // Copy attributes from the original element (which becomes the session title)
                        if let Some(content) = &elements[i].content {
                            session.set_attribute("title".to_string(), content.clone());
                        }

                        // Set location from original element
                        if let (Some(start), Some(end)) =
                            (elements[i].start_line, elements[i].end_line)
                        {
                            session.set_location(start, end);
                        }

                        // Create session container for the content
                        let mut session_container = AstNode::new("session_container".to_string());

                        // Parse all elements in the child block
                        let child_elements = self.parse_block_contents_as_session(child_block);
                        for child_element in child_elements {
                            session_container.add_child(child_element);
                        }

                        session.add_child(session_container);
                        elements[i] = session;
                    } else if matches!(
                        elements[i].node_type.as_str(),
                        "definition" | "annotation" | "list_item"
                    ) {
                        // Create a content container for non-session elements that can have children
                        let mut content_container = AstNode::new("content_container".to_string());

                        // Parse all elements in the child block (no sessions allowed in content containers)
                        let child_elements = self.parse_block_contents(child_block);
                        for child_element in child_elements {
                            content_container.add_child(child_element);
                        }

                        elements[i].add_child(content_container);
                    }
                }
            }
        }
    }

    /// Check if an element with the given child block should be considered a session
    /// Sessions require: preceded by blank line, non-empty indented content
    #[allow(dead_code)]
    fn is_session_title(&self, element: &AstNode, child_block: &TokenBlock) -> bool {
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

    /// Parse block contents as a session container (can contain sessions)
    #[allow(dead_code)]
    fn parse_block_contents_as_session(&self, block: &TokenBlock) -> Vec<AstNode> {
        let mut elements = Vec::new();

        // First try to parse the block's own tokens as elements
        if !block.tokens.is_empty() {
            let element_groups = self.split_tokens_into_elements(&block.tokens);
            for element_tokens in element_groups {
                let token_refs: Vec<&Token> = element_tokens.to_vec();
                if let Some(element) = self.parse_element_refs(&token_refs) {
                    elements.push(element);
                }
            }
        }

        // Then process child blocks and check for sessions
        self.connect_child_blocks_to_elements(&mut elements, &block.children, true);

        // Group consecutive list items into lists
        self.group_list_items(elements)
    }

    /// Parse a block using bottom-up approach for proper session detection
    /// is_session_context: true if this context can contain sessions
    fn parse_block_bottom_up(&self, block: &TokenBlock, is_session_context: bool) -> Vec<AstNode> {
        let mut elements = Vec::new();

        // First, parse any tokens in this block as individual elements
        if !block.tokens.is_empty() {
            let element_groups = self.split_tokens_into_elements(&block.tokens);
            for element_tokens in element_groups {
                let token_refs: Vec<&Token> = element_tokens.to_vec();
                if let Some(element) = self.parse_element_refs(&token_refs) {
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
            return self.group_list_items(elements);
        }

        // Now try to associate child blocks with parent elements
        // and determine if any should be sessions
        if is_session_context {
            self.associate_children_and_detect_sessions(&mut elements, &parsed_child_blocks);
        } else {
            self.associate_children_as_content_containers(&mut elements, &parsed_child_blocks);
        }

        // Group consecutive list items into lists
        self.group_list_items(elements)
    }

    /// Associate child blocks with elements and detect sessions (for SessionContainers)
    fn associate_children_and_detect_sessions(
        &self,
        elements: &mut [AstNode],
        parsed_child_blocks: &[(&TokenBlock, Vec<AstNode>)],
    ) {
        for (child_block, child_elements) in parsed_child_blocks {
            if let Some(child_start_line) = child_block.start_line {
                // Find the most recent element that could be the parent
                let mut target_element_index = None;
                let mut best_end_line = 0;
                for (i, element) in elements.iter().enumerate() {
                    if let Some(element_end_line) = element.end_line {
                        if element_end_line < child_start_line && element_end_line > best_end_line {
                            target_element_index = Some(i);
                            best_end_line = element_end_line;
                        }
                    }
                }

                if let Some(i) = target_element_index {
                    // Check if this should be a session
                    let should_be_session = self.should_be_session(&elements[i], child_elements);

                    if should_be_session {
                        // Convert to session
                        let mut session = AstNode::new("session".to_string());

                        // Copy title from original element
                        if let Some(content) = &elements[i].content {
                            session.set_attribute("title".to_string(), content.clone());
                        }

                        // Set location from original element
                        if let (Some(start), Some(end)) =
                            (elements[i].start_line, elements[i].end_line)
                        {
                            session.set_location(start, end);
                        }

                        // Create session container
                        let mut session_container = AstNode::new("session_container".to_string());
                        for child_element in child_elements {
                            session_container.add_child(child_element.clone());
                        }

                        session.add_child(session_container);
                        elements[i] = session;
                    } else if matches!(
                        elements[i].node_type.as_str(),
                        "definition" | "annotation" | "list_item"
                    ) {
                        // Create content container for non-session elements
                        // Re-parse child block with is_session_context = false to ensure no sessions
                        let content_child_elements = self.parse_block_bottom_up(child_block, false);
                        let mut content_container = AstNode::new("content_container".to_string());
                        for child_element in content_child_elements {
                            content_container.add_child(child_element);
                        }
                        elements[i].add_child(content_container);
                    }
                }
            }
        }
    }

    /// Associate child blocks with elements as ContentContainers (no sessions allowed)
    fn associate_children_as_content_containers(
        &self,
        elements: &mut [AstNode],
        parsed_child_blocks: &[(&TokenBlock, Vec<AstNode>)],
    ) {
        for (child_block, child_elements) in parsed_child_blocks {
            if let Some(child_start_line) = child_block.start_line {
                // Find the most recent element that could be the parent
                let mut target_element_index = None;
                let mut best_end_line = 0;
                for (i, element) in elements.iter().enumerate() {
                    if let Some(element_end_line) = element.end_line {
                        if element_end_line < child_start_line && element_end_line > best_end_line {
                            target_element_index = Some(i);
                            best_end_line = element_end_line;
                        }
                    }
                }

                if let Some(i) = target_element_index {
                    if matches!(
                        elements[i].node_type.as_str(),
                        "definition" | "annotation" | "list_item"
                    ) {
                        // Create content container (no sessions allowed)
                        let mut content_container = AstNode::new("content_container".to_string());
                        for child_element in child_elements {
                            content_container.add_child(child_element.clone());
                        }
                        elements[i].add_child(content_container);
                    }
                }
            }
        }
    }

    /// Determine if an element with given child content should be a session
    fn should_be_session(&self, element: &AstNode, child_elements: &[AstNode]) -> bool {
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
}

/// Main entry point for parsing
pub fn parse_document(source: String, block: &TokenBlock) -> Document {
    let parser = DocumentParser::new(source);
    parser.parse(block)
}
