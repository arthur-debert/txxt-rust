use super::blocks::{Block, BlockNode, Container, ContentContainer, SessionContainer};
use super::container_types::{BlockType, ContainerType};
use crate::tokenizer::{Token, TokenType};

/// Intermediate token block for initial tree building
#[derive(Debug, Clone)]
struct TokenBlock {
    tokens: Vec<Token>,
    children: Vec<TokenBlock>,
    indent_level: usize,
    start_line: Option<usize>,
    end_line: Option<usize>,
}

impl TokenBlock {
    fn new(indent_level: usize) -> Self {
        Self {
            tokens: Vec::new(),
            children: Vec::new(),
            indent_level,
            start_line: None,
            end_line: None,
        }
    }

    fn add_token(&mut self, token: Token) {
        if self.start_line.is_none() {
            self.start_line = Some(token.line);
        }
        self.end_line = Some(token.line);
        self.tokens.push(token);
    }

    fn add_child(&mut self, child: TokenBlock) {
        if let Some(end_line) = child.end_line {
            self.end_line = Some(end_line);
        }
        self.children.push(child);
    }
}

pub struct BlockTreeBuilder {
    tokens: Vec<Token>,
    position: usize,
}

impl BlockTreeBuilder {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn build(&mut self) -> Block {
        // Stage 1: Build basic indentation tree (similar to Python reference)
        let token_tree = self.build_token_tree();

        // Stage 2: Split by blank lines
        let split_tree = self.split_by_blank_lines(token_tree);

        // Stage 3: Convert to semantic blocks with containers
        self.convert_to_semantic_blocks(split_tree)
    }

    /// Build basic indentation tree from tokens
    fn build_token_tree(&mut self) -> TokenBlock {
        // Use a simpler approach that builds the tree recursively
        self.position = 0;
        self.build_token_block_recursive(0)
    }

    fn build_token_block_recursive(&mut self, indent_level: usize) -> TokenBlock {
        let mut block = TokenBlock::new(indent_level);

        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position];

            match token.token_type {
                TokenType::Indent => {
                    // Start a new indented block
                    self.position += 1; // Skip the INDENT token
                    let child_block = self.build_token_block_recursive(indent_level + 1);
                    block.add_child(child_block);
                }

                TokenType::Dedent => {
                    // End this block and return to parent
                    self.position += 1; // Skip the DEDENT token
                    return block;
                }

                TokenType::Eof => {
                    // End of input
                    self.position += 1;
                    return block;
                }

                _ => {
                    // Regular token - add to current block
                    block.add_token(token.clone());
                    self.position += 1;
                }
            }
        }

        block
    }

    /// Split blocks by blank lines to create logical sections
    #[allow(clippy::only_used_in_recursion)]
    fn split_by_blank_lines(&self, mut block: TokenBlock) -> TokenBlock {
        // First, recursively process children
        for i in 0..block.children.len() {
            block.children[i] = self.split_by_blank_lines(block.children[i].clone());
        }

        // Check if this block has blank lines to split by
        let blank_line_positions: Vec<usize> = block
            .tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.token_type == TokenType::BlankLine)
            .map(|(i, _)| i)
            .collect();

        if blank_line_positions.is_empty() {
            return block;
        }

        // Split tokens into groups separated by blank lines
        let mut new_children = Vec::new();
        let mut start = 0;

        for &blank_pos in &blank_line_positions {
            // Add block for tokens before blank line
            if start < blank_pos {
                let mut new_block = TokenBlock::new(block.indent_level);
                new_block.tokens = block.tokens[start..blank_pos].to_vec();
                if let Some(first) = new_block.tokens.first() {
                    new_block.start_line = Some(first.line);
                }
                if let Some(last) = new_block.tokens.last() {
                    new_block.end_line = Some(last.line);
                }
                new_children.push(new_block);
            }

            // Add blank line as its own block
            let mut blank_block = TokenBlock::new(block.indent_level);
            blank_block.add_token(block.tokens[blank_pos].clone());
            new_children.push(blank_block);

            start = blank_pos + 1;
        }

        // Add remaining tokens
        if start < block.tokens.len() {
            let mut new_block = TokenBlock::new(block.indent_level);
            new_block.tokens = block.tokens[start..].to_vec();
            if let Some(first) = new_block.tokens.first() {
                new_block.start_line = Some(first.line);
            }
            if let Some(last) = new_block.tokens.last() {
                new_block.end_line = Some(last.line);
            }
            new_children.push(new_block);
        }

        // Distribute existing children to appropriate new blocks
        for child in block.children {
            let mut added = false;
            for new_block in &mut new_children {
                if let (Some(child_start), Some(block_end)) = (child.start_line, new_block.end_line)
                {
                    if child_start > block_end {
                        continue;
                    }
                    if let Some(block_start) = new_block.start_line {
                        if child_start >= block_start {
                            new_block.add_child(child.clone());
                            added = true;
                            break;
                        }
                    }
                }
            }

            // If not added anywhere, add to last non-blank block
            if !added {
                for new_block in new_children.iter_mut().rev() {
                    if !new_block.tokens.is_empty()
                        && new_block.tokens[0].token_type != TokenType::BlankLine
                    {
                        new_block.add_child(child);
                        break;
                    }
                }
            }
        }

        // Update the block
        block.tokens.clear();
        block.children = new_children;
        block
    }

    /// Convert token blocks to semantic blocks with proper containers
    fn convert_to_semantic_blocks(&self, token_block: TokenBlock) -> Block {
        // Determine the block type from tokens
        let block_type = self.determine_block_type(&token_block);

        let mut block_node = BlockNode::new(block_type);

        // Set line information
        if let (Some(start), Some(end)) = (token_block.start_line, token_block.end_line) {
            block_node = block_node.with_lines(start, end);
        }

        // Convert children if any
        if !token_block.children.is_empty() {
            let semantic_children: Vec<Block> = token_block
                .children
                .into_iter()
                .map(|child| self.convert_to_semantic_blocks(child))
                .collect();

            // Create appropriate container
            if block_node.block_type.can_have_container() {
                let container_type = block_node
                    .block_type
                    .container_type()
                    .unwrap_or(ContainerType::Content);

                let container = match container_type {
                    ContainerType::Content => {
                        let mut content = ContentContainer::new(token_block.indent_level + 1);
                        for child in semantic_children {
                            content.add_child(child).unwrap_or_else(|e| {
                                eprintln!("Warning: {}", e);
                            });
                        }
                        Container::Content(content)
                    }
                    ContainerType::Session => {
                        let mut session = SessionContainer::new(token_block.indent_level + 1);
                        for child in semantic_children {
                            session.add_child(child);
                        }
                        Container::Session(session)
                    }
                };

                block_node = block_node.with_container(container);
            }
        }

        Block::Node(block_node)
    }

    /// Determine block type from token analysis
    fn determine_block_type(&self, token_block: &TokenBlock) -> BlockType {
        if token_block.tokens.is_empty() {
            return BlockType::Root;
        }

        // Check for blank line
        if token_block.tokens.len() == 1 && token_block.tokens[0].token_type == TokenType::BlankLine
        {
            return BlockType::BlankLine {
                token: token_block.tokens[0].clone(),
            };
        }

        // Check for pragma (annotation)
        if self.starts_with_pragma(&token_block.tokens) {
            return self.parse_annotation(&token_block.tokens);
        }

        // Check for definition (ends with ::)
        if self.ends_with_definition_marker(&token_block.tokens) {
            return self.parse_definition(&token_block.tokens);
        }

        // Check for verbatim block
        if self.is_verbatim_block(&token_block.tokens) {
            return BlockType::Verbatim {
                tokens: token_block.tokens.clone(),
            };
        }

        // Check for list item
        if self.starts_with_list_marker(&token_block.tokens) {
            return self.parse_list_item(&token_block.tokens);
        }

        // Check for session (requires context analysis)
        if self.is_session_title(token_block) {
            return BlockType::Session {
                title_tokens: token_block.tokens.clone(),
            };
        }

        // Default to paragraph
        BlockType::Paragraph {
            tokens: token_block.tokens.clone(),
        }
    }

    fn starts_with_pragma(&self, tokens: &[Token]) -> bool {
        tokens
            .first()
            .map(|t| t.token_type == TokenType::PragmaMarker)
            .unwrap_or(false)
    }

    fn parse_annotation(&self, tokens: &[Token]) -> BlockType {
        // Extract label from pragma tokens
        let label = tokens
            .iter()
            .skip_while(|t| t.token_type == TokenType::PragmaMarker)
            .take_while(|t| t.token_type == TokenType::Identifier)
            .filter_map(|t| t.value.as_ref())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        BlockType::Annotation {
            label,
            pragma_tokens: tokens.to_vec(),
        }
    }

    fn ends_with_definition_marker(&self, tokens: &[Token]) -> bool {
        tokens
            .iter()
            .any(|t| t.token_type == TokenType::DefinitionMarker)
    }

    fn parse_definition(&self, tokens: &[Token]) -> BlockType {
        let def_marker_pos = tokens
            .iter()
            .position(|t| t.token_type == TokenType::DefinitionMarker)
            .unwrap_or(tokens.len());

        let term_tokens = tokens[..def_marker_pos].to_vec();
        let definition_marker = tokens.get(def_marker_pos).cloned().unwrap_or_else(|| {
            Token::new(TokenType::DefinitionMarker, Some("::".to_string()), 0, 0)
        });

        BlockType::Definition {
            term_tokens,
            definition_marker,
        }
    }

    fn is_verbatim_block(&self, tokens: &[Token]) -> bool {
        tokens.iter().any(|t| {
            matches!(
                t.token_type,
                TokenType::VerbatimStart | TokenType::VerbatimContent | TokenType::VerbatimEnd
            )
        })
    }

    fn starts_with_list_marker(&self, tokens: &[Token]) -> bool {
        tokens
            .first()
            .map(|t| matches!(t.token_type, TokenType::SequenceMarker | TokenType::Dash))
            .unwrap_or(false)
    }

    fn parse_list_item(&self, tokens: &[Token]) -> BlockType {
        let marker_token = tokens[0].clone();
        let inline_tokens = tokens[1..].to_vec();

        BlockType::ListItem {
            marker_token,
            inline_tokens,
        }
    }

    fn is_session_title(&self, token_block: &TokenBlock) -> bool {
        // Session detection logic based on documentation:
        // 1. Must have blank line before (handled by parent)
        // 2. Must have indented children (has children)
        // 3. Cannot be empty (has content)

        // For now, simplified: if it has children and isn't a list/annotation/definition
        !token_block.children.is_empty()
            && !self.starts_with_list_marker(&token_block.tokens)
            && !self.starts_with_pragma(&token_block.tokens)
            && !self.ends_with_definition_marker(&token_block.tokens)
            && !self.is_verbatim_block(&token_block.tokens)
    }
}

/// Main entry point for block tree building
pub fn build_block_tree(tokens: Vec<Token>) -> Block {
    let mut builder = BlockTreeBuilder::new(tokens);
    builder.build()
}
