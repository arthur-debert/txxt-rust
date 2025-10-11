#[derive(Debug, Clone, PartialEq)]
pub enum ContainerType {
    /// ContentContainer - can hold any element except sessions
    Content,
    /// SessionContainer - can hold any element including sessions
    Session,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    /// Document root - always a session container
    Root,

    /// Session with title and session container
    Session {
        title_tokens: Vec<crate::tokenizer::Token>,
    },

    /// Paragraph - sequence of text lines, no container
    Paragraph {
        tokens: Vec<crate::tokenizer::Token>,
    },

    /// List (ordered or unordered) with content container
    List { items: Vec<ListItem> },

    /// Individual list item with inline content + optional container
    ListItem {
        marker_token: crate::tokenizer::Token,
        inline_tokens: Vec<crate::tokenizer::Token>,
    },

    /// Definition with term and content container
    Definition {
        term_tokens: Vec<crate::tokenizer::Token>,
        definition_marker: crate::tokenizer::Token,
    },

    /// Annotation with label and content container
    Annotation {
        label: String,
        pragma_tokens: Vec<crate::tokenizer::Token>,
    },

    /// Verbatim block - no container (literal content)
    Verbatim {
        tokens: Vec<crate::tokenizer::Token>,
    },

    /// TextLine - represents a single line within a paragraph
    TextLine {
        tokens: Vec<crate::tokenizer::Token>,
    },

    /// Blank line token
    BlankLine { token: crate::tokenizer::Token },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub marker_token: crate::tokenizer::Token,
    pub inline_tokens: Vec<crate::tokenizer::Token>,
}

impl BlockType {
    /// Returns true if this block type can have a container with children
    pub fn can_have_container(&self) -> bool {
        matches!(
            self,
            BlockType::Root
                | BlockType::Session { .. }
                | BlockType::List { .. }
                | BlockType::ListItem { .. }
                | BlockType::Definition { .. }
                | BlockType::Annotation { .. }
        )
    }

    /// Returns the container type this block should use, if any
    pub fn container_type(&self) -> Option<ContainerType> {
        match self {
            BlockType::Root | BlockType::Session { .. } => Some(ContainerType::Session),
            BlockType::List { .. }
            | BlockType::ListItem { .. }
            | BlockType::Definition { .. }
            | BlockType::Annotation { .. } => Some(ContainerType::Content),
            _ => None,
        }
    }

    /// Returns true if this is a session-type block
    pub fn is_session(&self) -> bool {
        matches!(self, BlockType::Root | BlockType::Session { .. })
    }

    /// Returns true if this is a content block (no container)
    pub fn is_content_block(&self) -> bool {
        matches!(
            self,
            BlockType::Paragraph { .. }
                | BlockType::Verbatim { .. }
                | BlockType::TextLine { .. }
                | BlockType::BlankLine { .. }
        )
    }
}
