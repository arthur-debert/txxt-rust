use super::container_types::{BlockType, ContainerType};

#[derive(Debug, Clone, PartialEq)]
pub struct BlockNode {
    pub block_type: BlockType,
    pub container: Option<Box<Container>>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Container {
    Content(ContentContainer),
    Session(SessionContainer),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentContainer {
    pub children: Vec<Block>,
    pub indent_level: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionContainer {
    pub children: Vec<Block>,
    pub indent_level: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Node(BlockNode),
    Container(Container),
}

impl BlockNode {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            block_type,
            container: None,
            start_line: None,
            end_line: None,
        }
    }

    pub fn with_container(mut self, container: Container) -> Self {
        self.container = Some(Box::new(container));
        self
    }

    pub fn with_lines(mut self, start_line: usize, end_line: usize) -> Self {
        self.start_line = Some(start_line);
        self.end_line = Some(end_line);
        self
    }

    pub fn add_child(&mut self, child: Block) -> Result<(), String> {
        let container = self.container.get_or_insert_with(|| {
            let container_type = self
                .block_type
                .container_type()
                .unwrap_or(ContainerType::Content);
            Box::new(match container_type {
                ContainerType::Content => Container::Content(ContentContainer {
                    children: Vec::new(),
                    indent_level: 0,
                }),
                ContainerType::Session => Container::Session(SessionContainer {
                    children: Vec::new(),
                    indent_level: 0,
                }),
            })
        });

        match (container.as_mut(), &child) {
            (Container::Content(content), Block::Node(node)) => {
                // Content containers cannot contain sessions
                if node.block_type.is_session() {
                    return Err("Content containers cannot contain sessions".to_string());
                }
                content.children.push(child);
                Ok(())
            }
            (Container::Session(session), _) => {
                // Session containers can contain anything
                session.children.push(child);
                Ok(())
            }
            (Container::Content(content), Block::Container(_)) => {
                // Allow container blocks in content containers
                content.children.push(child);
                Ok(())
            }
        }
    }

    pub fn get_children(&self) -> Vec<&Block> {
        match &self.container {
            Some(container) => match container.as_ref() {
                Container::Content(content) => content.children.iter().collect(),
                Container::Session(session) => session.children.iter().collect(),
            },
            None => Vec::new(),
        }
    }
}

impl ContentContainer {
    pub fn new(indent_level: usize) -> Self {
        Self {
            children: Vec::new(),
            indent_level,
        }
    }

    pub fn add_child(&mut self, child: Block) -> Result<(), String> {
        // Validate that content containers cannot contain sessions
        if let Block::Node(node) = &child {
            if node.block_type.is_session() {
                return Err("Content containers cannot contain sessions".to_string());
            }
        }
        self.children.push(child);
        Ok(())
    }
}

impl SessionContainer {
    pub fn new(indent_level: usize) -> Self {
        Self {
            children: Vec::new(),
            indent_level,
        }
    }

    pub fn add_child(&mut self, child: Block) {
        self.children.push(child);
    }
}

impl Block {
    pub fn node(block_type: BlockType) -> Self {
        Block::Node(BlockNode::new(block_type))
    }

    pub fn content_container(indent_level: usize) -> Self {
        Block::Container(Container::Content(ContentContainer::new(indent_level)))
    }

    pub fn session_container(indent_level: usize) -> Self {
        Block::Container(Container::Session(SessionContainer::new(indent_level)))
    }

    pub fn start_line(&self) -> Option<usize> {
        match self {
            Block::Node(node) => node.start_line,
            Block::Container(_) => None, // Containers don't have direct line numbers
        }
    }

    pub fn end_line(&self) -> Option<usize> {
        match self {
            Block::Node(node) => node.end_line,
            Block::Container(_) => None,
        }
    }
}
