//! Node reference types for type-erased traversal
//!
//! This module provides the `AstNodeRef` enum and `AstNode` trait that enable
//! uniform traversal across heterogeneous AST node types.

use crate::ast::{
    annotations::Annotation,
    base::Document,
    blocks::{Block, Definition, List, ListItem, VerbatimBlock},
    inlines::Inline,
    parameters::Parameters,
    structure::{BlankLine, Container, IgnoreLine, Paragraph, Session},
    tokens::ScannerTokenSequence,
};

/// Core trait for all AST nodes enabling uniform traversal
pub trait AstNode {
    /// Visit all direct children of this node
    fn children(&self) -> Vec<AstNodeRef<'_>>;

    /// Get node type for filtering/matching
    fn node_type(&self) -> NodeType;

    /// Get text content if applicable (for text nodes, labels, etc.)
    fn text_content(&self) -> Option<String>;

    /// Get annotations attached to this node
    fn annotations(&self) -> &[Annotation];

    /// Get parameters if this node supports them
    fn parameters(&self) -> Option<&Parameters>;

    /// Get computed level in tree (expensive - requires tree walk)
    fn level(&self) -> usize {
        0 // Default implementation
    }

    /// Check if node matches a predicate
    fn matches(&self, predicate: &super::predicates::NodePredicate) -> bool {
        predicate.matches(self)
    }
}

/// Type-erased node reference for heterogeneous traversal
///
/// This enum allows us to traverse the AST uniformly even though different
/// node types have different structures. It's similar to rowan's untyped nodes
/// but with more type information preserved.
#[derive(Clone, Copy)]
pub enum AstNodeRef<'a> {
    Document(&'a Document),
    Block(&'a Block),
    Container(&'a Container),
    Session(&'a Session),
    Paragraph(&'a Paragraph),
    List(&'a List),
    ListItem(&'a ListItem),
    Definition(&'a Definition),
    VerbatimBlock(&'a VerbatimBlock),
    Inline(&'a Inline),
    BlankLine(&'a BlankLine),
    IgnoreLine(&'a IgnoreLine),
    Annotation(&'a Annotation),
}

/// Node type discriminator for type-based filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Document,
    Block,
    Container,
    Session,
    Paragraph,
    List,
    ListItem,
    Definition,
    VerbatimBlock,
    Inline,
    BlankLine,
    IgnoreLine,
    Annotation,
}

impl<'a> AstNode for AstNodeRef<'a> {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        match self {
            AstNodeRef::Document(doc) => doc.children(),
            AstNodeRef::Block(block) => block.children(),
            AstNodeRef::Container(container) => container.children(),
            AstNodeRef::Session(session) => session.children(),
            AstNodeRef::Paragraph(para) => para.children(),
            AstNodeRef::List(list) => list.children(),
            AstNodeRef::ListItem(item) => item.children(),
            AstNodeRef::Definition(def) => def.children(),
            AstNodeRef::VerbatimBlock(vb) => vb.children(),
            AstNodeRef::Inline(inline) => inline.children(),
            AstNodeRef::BlankLine(bl) => bl.children(),
            AstNodeRef::IgnoreLine(il) => il.children(),
            AstNodeRef::Annotation(ann) => ann.children(),
        }
    }

    fn node_type(&self) -> NodeType {
        match self {
            AstNodeRef::Document(_) => NodeType::Document,
            AstNodeRef::Block(_) => NodeType::Block,
            AstNodeRef::Container(_) => NodeType::Container,
            AstNodeRef::Session(_) => NodeType::Session,
            AstNodeRef::Paragraph(_) => NodeType::Paragraph,
            AstNodeRef::List(_) => NodeType::List,
            AstNodeRef::ListItem(_) => NodeType::ListItem,
            AstNodeRef::Definition(_) => NodeType::Definition,
            AstNodeRef::VerbatimBlock(_) => NodeType::VerbatimBlock,
            AstNodeRef::Inline(_) => NodeType::Inline,
            AstNodeRef::BlankLine(_) => NodeType::BlankLine,
            AstNodeRef::IgnoreLine(_) => NodeType::IgnoreLine,
            AstNodeRef::Annotation(_) => NodeType::Annotation,
        }
    }

    fn text_content(&self) -> Option<String> {
        match self {
            AstNodeRef::Paragraph(para) => Some(extract_text_from_inlines(&para.content)),
            AstNodeRef::IgnoreLine(il) => Some(il.content.clone()),
            AstNodeRef::Annotation(ann) => ann.text_content(),
            AstNodeRef::Inline(inline) => Some(extract_text_from_inline(inline)),
            _ => None,
        }
    }

    fn annotations(&self) -> &[Annotation] {
        match self {
            AstNodeRef::Container(c) => &c.annotations,
            AstNodeRef::Session(s) => &s.annotations,
            AstNodeRef::Paragraph(p) => &p.annotations,
            AstNodeRef::List(l) => &l.annotations,
            AstNodeRef::ListItem(li) => &li.annotations,
            AstNodeRef::Definition(d) => &d.annotations,
            AstNodeRef::VerbatimBlock(vb) => &vb.annotations,
            _ => &[],
        }
    }

    fn parameters(&self) -> Option<&Parameters> {
        match self {
            AstNodeRef::Definition(d) => Some(&d.parameters),
            AstNodeRef::VerbatimBlock(vb) => Some(&vb.parameters),
            _ => None,
        }
    }
}

impl<'a> AstNodeRef<'a> {
    /// Get a unique identifier for this node (for cycle detection)
    pub fn id(&self) -> usize {
        match self {
            AstNodeRef::Document(doc) => doc as *const _ as usize,
            AstNodeRef::Block(block) => *block as *const _ as usize,
            AstNodeRef::Container(c) => *c as *const _ as usize,
            AstNodeRef::Session(s) => *s as *const _ as usize,
            AstNodeRef::Paragraph(p) => *p as *const _ as usize,
            AstNodeRef::List(l) => *l as *const _ as usize,
            AstNodeRef::ListItem(li) => *li as *const _ as usize,
            AstNodeRef::Definition(d) => *d as *const _ as usize,
            AstNodeRef::VerbatimBlock(vb) => *vb as *const _ as usize,
            AstNodeRef::Inline(i) => *i as *const _ as usize,
            AstNodeRef::BlankLine(bl) => *bl as *const _ as usize,
            AstNodeRef::IgnoreLine(il) => *il as *const _ as usize,
            AstNodeRef::Annotation(ann) => *ann as *const _ as usize,
        }
    }
}

// Implement AstNode for concrete types
impl AstNode for Document {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        vec![AstNodeRef::Container(&self.content)]
    }

    fn node_type(&self) -> NodeType {
        NodeType::Document
    }

    fn text_content(&self) -> Option<String> {
        None
    }

    fn annotations(&self) -> &[Annotation] {
        &[]
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for Block {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        match self {
            Block::Paragraph(p) => vec![AstNodeRef::Paragraph(p)],
            Block::VerbatimBlock(vb) => vec![AstNodeRef::VerbatimBlock(vb)],
            Block::BlankLine(bl) => vec![AstNodeRef::BlankLine(bl)],
            Block::List(l) => vec![AstNodeRef::List(l)],
            Block::Definition(d) => vec![AstNodeRef::Definition(d)],
            Block::Session(s) => vec![AstNodeRef::Session(s)],
            Block::Container(c) => vec![AstNodeRef::Container(c)],
        }
    }

    fn node_type(&self) -> NodeType {
        NodeType::Block
    }

    fn text_content(&self) -> Option<String> {
        match self {
            Block::Paragraph(p) => p.text_content(),
            Block::VerbatimBlock(vb) => vb.text_content(),
            _ => None,
        }
    }

    fn annotations(&self) -> &[Annotation] {
        match self {
            Block::Paragraph(p) => &p.annotations,
            Block::VerbatimBlock(vb) => &vb.annotations,
            Block::List(l) => &l.annotations,
            Block::Definition(d) => &d.annotations,
            Block::Session(s) => &s.annotations,
            Block::Container(c) => &c.annotations,
            Block::BlankLine(_) => &[],
        }
    }

    fn parameters(&self) -> Option<&Parameters> {
        match self {
            Block::Definition(d) => Some(&d.parameters),
            Block::VerbatimBlock(vb) => Some(&vb.parameters),
            _ => None,
        }
    }
}

impl AstNode for Container {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        self.content.iter().map(AstNodeRef::Block).collect()
    }

    fn node_type(&self) -> NodeType {
        NodeType::Container
    }

    fn text_content(&self) -> Option<String> {
        None
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for Session {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        vec![AstNodeRef::Container(&self.content)]
    }

    fn node_type(&self) -> NodeType {
        NodeType::Session
    }

    fn text_content(&self) -> Option<String> {
        Some(extract_text_from_inlines(&self.title.content))
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for Paragraph {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        self.content.iter().map(AstNodeRef::Inline).collect()
    }

    fn node_type(&self) -> NodeType {
        NodeType::Paragraph
    }

    fn text_content(&self) -> Option<String> {
        Some(extract_text_from_inlines(&self.content))
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for List {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        self.items.iter().map(AstNodeRef::ListItem).collect()
    }

    fn node_type(&self) -> NodeType {
        NodeType::List
    }

    fn text_content(&self) -> Option<String> {
        None
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for ListItem {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        let mut children: Vec<AstNodeRef<'_>> =
            self.content.iter().map(AstNodeRef::Inline).collect();

        if let Some(nested) = &self.nested {
            children.push(AstNodeRef::Container(nested));
        }

        children
    }

    fn node_type(&self) -> NodeType {
        NodeType::ListItem
    }

    fn text_content(&self) -> Option<String> {
        Some(extract_text_from_inlines(&self.content))
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for Definition {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        vec![AstNodeRef::Container(&self.content)]
    }

    fn node_type(&self) -> NodeType {
        NodeType::Definition
    }

    fn text_content(&self) -> Option<String> {
        Some(extract_text_from_inlines(&self.term.content))
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> Option<&Parameters> {
        Some(&self.parameters)
    }
}

impl AstNode for VerbatimBlock {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        // Verbatim blocks are leaf nodes - their content is not parsed
        vec![]
    }

    fn node_type(&self) -> NodeType {
        NodeType::VerbatimBlock
    }

    fn text_content(&self) -> Option<String> {
        // Could extract raw content from IgnoreContainer if needed
        None
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> Option<&Parameters> {
        Some(&self.parameters)
    }
}

impl AstNode for Inline {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        // Inline elements could have children in complex cases
        // For now, treating as leaf nodes
        vec![]
    }

    fn node_type(&self) -> NodeType {
        NodeType::Inline
    }

    fn text_content(&self) -> Option<String> {
        Some(extract_text_from_inline(self))
    }

    fn annotations(&self) -> &[Annotation] {
        &[]
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for BlankLine {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        vec![]
    }

    fn node_type(&self) -> NodeType {
        NodeType::BlankLine
    }

    fn text_content(&self) -> Option<String> {
        None
    }

    fn annotations(&self) -> &[Annotation] {
        &[]
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for IgnoreLine {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        vec![]
    }

    fn node_type(&self) -> NodeType {
        NodeType::IgnoreLine
    }

    fn text_content(&self) -> Option<String> {
        Some(self.content.clone())
    }

    fn annotations(&self) -> &[Annotation] {
        &[]
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

impl AstNode for Annotation {
    fn children(&self) -> Vec<AstNodeRef<'_>> {
        // Annotations could have content - implement based on actual structure
        vec![]
    }

    fn node_type(&self) -> NodeType {
        NodeType::Annotation
    }

    fn text_content(&self) -> Option<String> {
        // Extract annotation content - implement based on actual structure
        None
    }

    fn annotations(&self) -> &[Annotation] {
        &[]
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
}

// Helper functions for text extraction
fn extract_text_from_inlines(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .filter_map(|inline| extract_text_from_inline(inline).into())
        .collect::<Vec<String>>()
        .join("")
}

fn extract_text_from_inline(inline: &Inline) -> String {
    match inline {
        Inline::TextLine(transform) => {
            // Extract text from transform - simplified for now
            // Would need to traverse transform hierarchy
            String::new()
        }
        Inline::Link { content, .. } => extract_text_from_inlines(content),
        Inline::Reference { content, .. } => content
            .as_ref()
            .map(|c| extract_text_from_inlines(c))
            .unwrap_or_default(),
        Inline::Custom { .. } => String::new(),
    }
}
