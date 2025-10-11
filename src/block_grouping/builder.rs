use crate::tokenizer::{Token, TokenType};

/// A block is simply a group of tokens with nested child blocks
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBlock {
    pub tokens: Vec<Token>,
    pub children: Vec<TokenBlock>,
    pub indent_level: usize,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

impl TokenBlock {
    pub fn new(indent_level: usize) -> Self {
        Self {
            tokens: Vec::new(),
            children: Vec::new(),
            indent_level,
            start_line: None,
            end_line: None,
        }
    }

    pub fn add_token(&mut self, token: Token) {
        if self.start_line.is_none() {
            self.start_line = Some(token.line);
        }
        self.end_line = Some(token.line);
        self.tokens.push(token);
    }

    pub fn add_child(&mut self, child: TokenBlock) {
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

    pub fn build(&mut self) -> TokenBlock {
        // Stage 1: Build basic indentation tree
        let token_tree = self.build_token_tree();

        // Stage 2: Split by blank lines
        self.split_by_blank_lines(token_tree)
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
}

/// Main entry point for block tree building
pub fn build_block_tree(tokens: Vec<Token>) -> TokenBlock {
    let mut builder = BlockTreeBuilder::new(tokens);
    builder.build()
}
