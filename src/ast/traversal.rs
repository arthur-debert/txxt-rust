//! Tree Traversal API using ego-tree
//!
//! This module provides a simple, efficient tree traversal system for TXXT AST nodes
//! using the ego-tree crate for parent/child navigation. This approach prioritizes
//! simplicity and fast delivery over the complexity of rowan-style red-green trees.
//!
//! # Design Philosophy
//!
//! - **Simple & Fast**: Use ego-tree for O(1) parent/child/sibling access
//! - **Query-Focused**: Build a powerful query API on top of simple traversal
//! - **Pragmatic**: Deliver value quickly, migrate to rowan later if needed for LSP
//! - **Type-Safe**: Maintain compile-time safety while providing flexible queries
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::ast::traversal::TraversableDocument;
//!
//! let doc = parse_document("# Hello\nThis is a paragraph.");
//! let traversable = TraversableDocument::from_document(&doc);
//!
//! // Find all paragraphs
//! let paragraphs = traversable.query()
//!     .find_by_type(ElementType::Block)
//!     .filter_blocks(|block| matches!(block, BlockElement::Paragraph(_)))
//!     .collect();
//!
//! // Text search
//! let matches = traversable.query()
//!     .text_contains("Hello")
//!     .collect();
//! ```

use std::collections::HashMap;

use ego_tree::{NodeRef, Tree};
use regex::Regex;

use crate::ast::{
    base::Document,
    elements::{
        containers::{
            content::{ContentContainer, ContentContainerElement},
            session::{SessionContainer, SessionContainerElement},
        },
        core::{ElementType, TxxtElement},
    },
};

/// Wrapper around a TXXT document that provides efficient tree traversal
/// using ego-tree for parent/child navigation.
pub struct TraversableDocument {
    /// The underlying tree structure using ego-tree
    tree: Tree<ElementWrapper>,

    /// Cache for frequently accessed nodes
    node_cache: HashMap<ElementId, ego_tree::NodeId>,
}

/// Unique identifier for AST elements (for caching and lookup)
type ElementId = usize;

/// Wrapper around AST elements to store in ego-tree  
pub struct ElementWrapper {
    /// The wrapped AST element (we'll implement Debug manually)
    pub element: Box<dyn TxxtElement + Send + Sync>,

    /// Unique identifier for this element
    pub id: ElementId,

    /// Element type (cached for performance)
    pub element_type: ElementType,
}

impl ElementWrapper {
    fn new(element: Box<dyn TxxtElement + Send + Sync>, id: ElementId) -> Self {
        let element_type = element.element_type();
        Self {
            element,
            id,
            element_type,
        }
    }
}

// Manual Debug implementation since TxxtElement doesn't implement Debug
impl std::fmt::Debug for ElementWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementWrapper")
            .field("id", &self.id)
            .field("element_type", &self.element_type)
            .finish()
    }
}

impl TraversableDocument {
    /// Create a traversable document from a TXXT document
    ///
    /// Note: This creates an owned copy of the document structure for traversal.
    /// The original document is not modified or referenced.
    pub fn from_document(document: &Document) -> Self {
        let mut id_counter = 0;
        let mut node_cache = HashMap::new();

        // Create document root
        let root_wrapper = ElementWrapper::new(
            Box::new(DocumentElementOwned::from_document(document)),
            id_counter,
        );
        id_counter += 1;

        let mut tree = Tree::new(root_wrapper);

        // Build the tree recursively from the document content
        {
            let mut root_node = tree.root_mut();
            Self::build_session_container_recursive(
                &document.content,
                &mut root_node,
                &mut id_counter,
                &mut node_cache,
            );
        }

        Self { tree, node_cache }
    }

    /// Get the root node of the tree
    pub fn root(&self) -> NodeRef<'_, ElementWrapper> {
        self.tree.root()
    }

    /// Create a query builder for this document
    pub fn query(&self) -> DocumentQuery<'_> {
        DocumentQuery::new(self)
    }

    /// Build the ego-tree recursively from a SessionContainer
    fn build_session_container_recursive(
        container: &SessionContainer,
        parent_node: &mut ego_tree::NodeMut<ElementWrapper>,
        id_counter: &mut ElementId,
        node_cache: &mut HashMap<ElementId, ego_tree::NodeId>,
    ) {
        for element in &container.content {
            Self::build_session_element_recursive(element, parent_node, id_counter, node_cache);
        }
    }

    /// Build the ego-tree recursively from a SessionContainerElement
    fn build_session_element_recursive(
        element: &SessionContainerElement,
        parent_node: &mut ego_tree::NodeMut<ElementWrapper>,
        id_counter: &mut ElementId,
        node_cache: &mut HashMap<ElementId, ego_tree::NodeId>,
    ) {
        match element {
            SessionContainerElement::Paragraph(paragraph) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_paragraph(paragraph)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            SessionContainerElement::List(list) => {
                let wrapper =
                    ElementWrapper::new(Box::new(ElementAdapter::from_list(list)), *id_counter);
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            SessionContainerElement::Definition(definition) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_definition(definition)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            SessionContainerElement::Verbatim(verbatim) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_verbatim(verbatim)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            SessionContainerElement::Annotation(annotation) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_annotation(annotation)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            SessionContainerElement::Session(session) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_session(session)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            SessionContainerElement::ContentContainer(content_container) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_content_container(content_container)),
                    *id_counter,
                );
                *id_counter += 1;
                let mut container_node = parent_node.append(wrapper);
                Self::build_content_container_recursive(
                    content_container,
                    &mut container_node,
                    id_counter,
                    node_cache,
                );
            }
            SessionContainerElement::SessionContainer(session_container) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_session_container(session_container)),
                    *id_counter,
                );
                *id_counter += 1;
                let mut container_node = parent_node.append(wrapper);
                Self::build_session_container_recursive(
                    session_container,
                    &mut container_node,
                    id_counter,
                    node_cache,
                );
            }
            SessionContainerElement::BlankLine(blank_line) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_blank_line(blank_line)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
        }
    }

    /// Build the ego-tree recursively from a ContentContainer
    fn build_content_container_recursive(
        container: &ContentContainer,
        parent_node: &mut ego_tree::NodeMut<ElementWrapper>,
        id_counter: &mut ElementId,
        node_cache: &mut HashMap<ElementId, ego_tree::NodeId>,
    ) {
        for element in &container.content {
            Self::build_content_element_recursive(element, parent_node, id_counter, node_cache);
        }
    }

    /// Build the ego-tree recursively from a ContentContainerElement
    fn build_content_element_recursive(
        element: &ContentContainerElement,
        parent_node: &mut ego_tree::NodeMut<ElementWrapper>,
        id_counter: &mut ElementId,
        node_cache: &mut HashMap<ElementId, ego_tree::NodeId>,
    ) {
        match element {
            ContentContainerElement::Paragraph(paragraph) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_paragraph(paragraph)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            ContentContainerElement::List(list) => {
                let wrapper =
                    ElementWrapper::new(Box::new(ElementAdapter::from_list(list)), *id_counter);
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            ContentContainerElement::Definition(definition) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_definition(definition)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            ContentContainerElement::Verbatim(verbatim) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_verbatim(verbatim)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            ContentContainerElement::Annotation(annotation) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_annotation(annotation)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
            ContentContainerElement::Container(content_container) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_content_container(content_container)),
                    *id_counter,
                );
                *id_counter += 1;
                let mut container_node = parent_node.append(wrapper);
                Self::build_content_container_recursive(
                    content_container,
                    &mut container_node,
                    id_counter,
                    node_cache,
                );
            }
            ContentContainerElement::BlankLine(blank_line) => {
                let wrapper = ElementWrapper::new(
                    Box::new(ElementAdapter::from_blank_line(blank_line)),
                    *id_counter,
                );
                *id_counter += 1;
                parent_node.append(wrapper);
            }
        }
    }

    /// Find a node by its element ID (using cache)
    pub fn find_node(&self, id: ElementId) -> Option<NodeRef<'_, ElementWrapper>> {
        self.node_cache.get(&id).and({
            // ego-tree doesn't expose node lookup by ID directly
            // We'd need to traverse or maintain our own lookup
            None // Placeholder
        })
    }
}

/// High-level query interface for traversable documents
pub struct DocumentQuery<'a> {
    document: &'a TraversableDocument,
    filters: Vec<QueryFilter>,
}

impl<'a> DocumentQuery<'a> {
    fn new(document: &'a TraversableDocument) -> Self {
        Self {
            document,
            filters: Vec::new(),
        }
    }

    /// Find elements by type
    pub fn find_by_type(mut self, element_type: ElementType) -> Self {
        self.filters.push(QueryFilter::ElementType(element_type));
        self
    }

    /// Find elements containing specific text
    pub fn text_contains(mut self, text: &str) -> Self {
        self.filters
            .push(QueryFilter::TextContains(text.to_string()));
        self
    }

    /// Find elements matching a regex pattern
    pub fn text_matches(mut self, pattern: Regex) -> Self {
        self.filters.push(QueryFilter::TextMatches(pattern));
        self
    }

    /// Find elements with specific annotation
    pub fn has_annotation(mut self, annotation_type: &str) -> Self {
        self.filters
            .push(QueryFilter::HasAnnotation(annotation_type.to_string()));
        self
    }

    /// Find elements with specific parameter
    pub fn has_parameter(mut self, key: &str, value: &str) -> Self {
        self.filters.push(QueryFilter::HasParameter(
            key.to_string(),
            value.to_string(),
        ));
        self
    }

    /// Execute the query and collect results
    pub fn collect(self) -> Vec<NodeRef<'a, ElementWrapper>> {
        let mut results = Vec::new();

        // Use iter() to traverse and convert edges to nodes
        for node in self.iter_nodes() {
            if self.matches_filters(node) {
                results.push(node);
            }
        }

        results
    }

    /// Get an iterator over all nodes in the tree
    fn iter_nodes(&self) -> impl Iterator<Item = NodeRef<'a, ElementWrapper>> {
        // ego-tree's traverse() returns Edge enum, we need to extract the nodes
        self.document
            .root()
            .traverse()
            .filter_map(|edge| match edge {
                ego_tree::iter::Edge::Open(node) => Some(node),
                ego_tree::iter::Edge::Close(_) => None,
            })
    }

    /// Get an iterator over matching nodes
    pub fn iter(self) -> impl Iterator<Item = NodeRef<'a, ElementWrapper>> {
        self.iter_nodes()
            .filter(move |node| self.matches_filters(*node))
    }

    /// Check if a node matches all filters
    fn matches_filters(&self, node: NodeRef<ElementWrapper>) -> bool {
        self.filters.iter().all(|filter| filter.matches(node))
    }
}

/// Query filters for element selection
#[derive(Debug, Clone)]
pub enum QueryFilter {
    /// Match elements by type
    ElementType(ElementType),

    /// Match elements containing text
    TextContains(String),

    /// Match elements with regex pattern
    TextMatches(Regex),

    /// Match elements with annotation
    HasAnnotation(String),

    /// Match elements with parameter
    HasParameter(String, String),

    /// Logical AND of filters
    And(Vec<QueryFilter>),

    /// Logical OR of filters
    Or(Vec<QueryFilter>),

    /// Logical NOT of filter
    Not(Box<QueryFilter>),
}

impl QueryFilter {
    /// Check if a node matches this filter
    pub fn matches(&self, node: NodeRef<ElementWrapper>) -> bool {
        match self {
            QueryFilter::ElementType(expected) => node.value().element_type == *expected,

            QueryFilter::TextContains(text) => {
                // Extract text content from the element and check if it contains the text
                self.extract_text_content(node).contains(text)
            }

            QueryFilter::TextMatches(regex) => {
                let text = self.extract_text_content(node);
                regex.is_match(&text)
            }

            QueryFilter::HasAnnotation(annotation_type) => node
                .value()
                .element
                .annotations()
                .iter()
                .any(|ann| ann.label == *annotation_type),

            QueryFilter::HasParameter(key, value) => node
                .value()
                .element
                .parameters()
                .get(key)
                .map(|v| v == value)
                .unwrap_or(false),

            QueryFilter::And(filters) => filters.iter().all(|f| f.matches(node)),

            QueryFilter::Or(filters) => filters.iter().any(|f| f.matches(node)),

            QueryFilter::Not(filter) => !filter.matches(node),
        }
    }

    /// Extract text content from an element (helper for text-based filters)
    fn extract_text_content(&self, node: NodeRef<ElementWrapper>) -> String {
        match &*node.value().element {
            // For adapters, extract text based on the wrapped element type
            element if element.element_type() == ElementType::Block => {
                // Try to extract text from block elements
                self.extract_block_text(element)
            }
            element if element.element_type() == ElementType::Line => {
                // Try to extract text from line elements (like BlankLine - usually empty)
                self.extract_line_text(element)
            }
            element if element.element_type() == ElementType::Container => {
                // For containers, we could recursively collect text from children
                // For now, just return basic identifier text
                format!("Container({:?})", element.element_type())
            }
            element => {
                // Fallback: use element type as identifier
                format!("{:?}", element.element_type())
            }
        }
    }

    /// Extract text content from block elements
    fn extract_block_text(&self, element: &dyn TxxtElement) -> String {
        // For this basic implementation, we'll use a heuristic approach
        // In a full implementation, we'd need to properly visit element content

        // Check if we can downcast to known types through the adapter
        // This is a simplified approach - ideally we'd have proper visitor pattern

        // For now, return a basic representation
        format!("Block content (type: {:?})", element.element_type())
    }

    /// Extract text content from line elements  
    fn extract_line_text(&self, element: &dyn TxxtElement) -> String {
        // Line elements typically don't have much text content
        // BlankLine for example is just structural
        format!("Line content (type: {:?})", element.element_type())
    }
}

/// Owned wrapper to make Document implement TxxtElement for tree root
/// This avoids lifetime issues by owning the data needed for traversal
#[derive(Debug)]
pub struct DocumentElementOwned {
    /// Document title (if any)
    #[allow(dead_code)]
    title: Option<String>,
    /// Number of top-level elements
    #[allow(dead_code)]
    content_count: usize,
}

impl DocumentElementOwned {
    pub fn from_document(document: &Document) -> Self {
        // Extract string from MetaValue if present
        let title = document.meta.title.as_ref().and_then(|meta_value| {
            match meta_value {
                crate::ast::base::MetaValue::String(s) => Some(s.clone()),
                _ => None, // For complex metadata, we'd need more sophisticated extraction
            }
        });

        Self {
            title,
            content_count: document.content.content.len(),
        }
    }
}

impl TxxtElement for DocumentElementOwned {
    fn element_type(&self) -> ElementType {
        ElementType::Container
    }

    fn tokens(&self) -> &crate::ast::tokens::TokenSequence {
        // Document doesn't have tokens directly, return empty sequence
        use std::sync::OnceLock;
        static EMPTY_TOKENS: OnceLock<crate::ast::tokens::TokenSequence> = OnceLock::new();
        EMPTY_TOKENS.get_or_init(crate::ast::tokens::TokenSequence::new)
    }

    fn annotations(&self) -> &[crate::ast::annotations::Annotation] {
        // Document doesn't have annotations directly
        &[]
    }

    fn parameters(&self) -> &crate::ast::parameters::Parameters {
        // Document doesn't have parameters directly
        use std::sync::OnceLock;
        static EMPTY_PARAMS: OnceLock<crate::ast::parameters::Parameters> = OnceLock::new();
        EMPTY_PARAMS.get_or_init(crate::ast::parameters::Parameters::default)
    }
}

// Make DocumentElementOwned Send + Sync so it can be stored in ElementWrapper
unsafe impl Send for DocumentElementOwned {}
unsafe impl Sync for DocumentElementOwned {}

/// Adapter to wrap AST elements for tree storage
/// This provides a uniform interface for all element types
#[derive(Debug)]
pub enum ElementAdapter {
    Paragraph(crate::ast::elements::paragraph::ParagraphBlock),
    List(crate::ast::elements::list::ListBlock),
    Definition(crate::ast::elements::definition::DefinitionBlock),
    Verbatim(crate::ast::elements::verbatim::VerbatimBlock),
    Annotation(crate::ast::elements::annotation::AnnotationBlock),
    Session(crate::ast::elements::session::SessionBlock),
    ContentContainer(ContentContainer),
    SessionContainer(SessionContainer),
    BlankLine(crate::ast::elements::core::BlankLine),
}

impl ElementAdapter {
    pub fn from_paragraph(p: &crate::ast::elements::paragraph::ParagraphBlock) -> Self {
        Self::Paragraph(p.clone())
    }

    pub fn from_list(l: &crate::ast::elements::list::ListBlock) -> Self {
        Self::List(l.clone())
    }

    pub fn from_definition(d: &crate::ast::elements::definition::DefinitionBlock) -> Self {
        Self::Definition(d.clone())
    }

    pub fn from_verbatim(v: &crate::ast::elements::verbatim::VerbatimBlock) -> Self {
        Self::Verbatim(v.clone())
    }

    pub fn from_annotation(a: &crate::ast::elements::annotation::AnnotationBlock) -> Self {
        Self::Annotation(a.clone())
    }

    pub fn from_session(s: &crate::ast::elements::session::SessionBlock) -> Self {
        Self::Session(s.clone())
    }

    pub fn from_content_container(c: &ContentContainer) -> Self {
        Self::ContentContainer(c.clone())
    }

    pub fn from_session_container(s: &SessionContainer) -> Self {
        Self::SessionContainer(s.clone())
    }

    pub fn from_blank_line(b: &crate::ast::elements::core::BlankLine) -> Self {
        Self::BlankLine(b.clone())
    }
}

impl TxxtElement for ElementAdapter {
    fn element_type(&self) -> ElementType {
        match self {
            Self::Paragraph(p) => p.element_type(),
            Self::List(l) => l.element_type(),
            Self::Definition(d) => d.element_type(),
            Self::Verbatim(v) => v.element_type(),
            Self::Annotation(a) => a.element_type(),
            Self::Session(s) => s.element_type(),
            Self::ContentContainer(c) => c.element_type(),
            Self::SessionContainer(s) => s.element_type(),
            Self::BlankLine(b) => b.element_type(),
        }
    }

    fn tokens(&self) -> &crate::ast::tokens::TokenSequence {
        match self {
            Self::Paragraph(p) => p.tokens(),
            Self::List(l) => l.tokens(),
            Self::Definition(d) => d.tokens(),
            Self::Verbatim(v) => v.tokens(),
            Self::Annotation(a) => a.tokens(),
            Self::Session(s) => s.tokens(),
            Self::ContentContainer(c) => c.tokens(),
            Self::SessionContainer(s) => s.tokens(),
            Self::BlankLine(b) => b.tokens(),
        }
    }

    fn annotations(&self) -> &[crate::ast::annotations::Annotation] {
        match self {
            Self::Paragraph(p) => p.annotations(),
            Self::List(l) => l.annotations(),
            Self::Definition(d) => d.annotations(),
            Self::Verbatim(v) => v.annotations(),
            Self::Annotation(a) => a.annotations(),
            Self::Session(s) => s.annotations(),
            Self::ContentContainer(c) => c.annotations(),
            Self::SessionContainer(s) => s.annotations(),
            Self::BlankLine(b) => b.annotations(),
        }
    }

    fn parameters(&self) -> &crate::ast::parameters::Parameters {
        match self {
            Self::Paragraph(p) => p.parameters(),
            Self::List(l) => l.parameters(),
            Self::Definition(d) => d.parameters(),
            Self::Verbatim(v) => v.parameters(),
            Self::Annotation(a) => a.parameters(),
            Self::Session(s) => s.parameters(),
            Self::ContentContainer(c) => c.parameters(),
            Self::SessionContainer(s) => s.parameters(),
            Self::BlankLine(b) => b.parameters(),
        }
    }
}

// Make ElementAdapter Send + Sync for thread safety
unsafe impl Send for ElementAdapter {}
unsafe impl Sync for ElementAdapter {}

// TODO: We need to implement Send + Sync for our AST elements to store them in ego-tree
// This might require some refactoring of the element trait hierarchy

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::base::{AssemblyInfo, Document, Meta};
    use crate::ast::elements::containers::session::SessionContainer;

    #[test]
    fn test_ego_tree_basic_functionality() {
        // Test that ego-tree works as expected
        let mut tree = Tree::new("root");
        let mut root = tree.root_mut();
        root.append("child1");
        root.append("child2");

        // Test navigation
        let root_ref = tree.root();
        let children: Vec<_> = root_ref.children().map(|n| *n.value()).collect();
        assert_eq!(children, vec!["child1", "child2"]);

        // Test parent access
        let first_child = root_ref.first_child().unwrap();
        assert_eq!(first_child.parent().unwrap().value(), &"root");
    }

    #[test]
    fn test_traversable_document_creation() {
        // Create a minimal document for testing
        let document = Document {
            meta: Meta {
                title: Some(crate::ast::base::MetaValue::String(
                    "Test Document".to_string(),
                )),
                ..Meta::default()
            },
            content: SessionContainer {
                content: vec![], // Empty for now
                annotations: vec![],
                parameters: crate::ast::parameters::Parameters::default(),
                tokens: crate::ast::tokens::TokenSequence::new(),
            },
            assembly_info: AssemblyInfo {
                parser_version: "test".to_string(),
                source_path: None,
                processed_at: None,
                stats: crate::ast::base::ProcessingStats::default(),
            },
        };

        // Create traversable document
        let traversable = TraversableDocument::from_document(&document);

        // Test basic navigation
        let root = traversable.root();
        assert_eq!(root.value().element_type, ElementType::Container);
        assert_eq!(root.value().id, 0);

        // Test query interface
        let query = traversable.query();
        let containers = query.find_by_type(ElementType::Container).collect();
        assert_eq!(containers.len(), 1); // Just the root document
    }

    #[test]
    fn test_document_element_owned() {
        // Test the DocumentElementOwned wrapper
        let document = Document {
            meta: Meta {
                title: Some(crate::ast::base::MetaValue::String(
                    "Test Title".to_string(),
                )),
                ..Meta::default()
            },
            content: SessionContainer {
                content: vec![],
                annotations: vec![],
                parameters: crate::ast::parameters::Parameters::default(),
                tokens: crate::ast::tokens::TokenSequence::new(),
            },
            assembly_info: AssemblyInfo {
                parser_version: "test".to_string(),
                source_path: None,
                processed_at: None,
                stats: crate::ast::base::ProcessingStats::default(),
            },
        };

        let doc_element = DocumentElementOwned::from_document(&document);
        assert_eq!(doc_element.title, Some("Test Title".to_string()));
        assert_eq!(doc_element.content_count, 0);
        assert_eq!(doc_element.element_type(), ElementType::Container);
    }

    #[test]
    fn test_tree_building_with_content() {
        use crate::ast::elements::{
            containers::ignore::IgnoreContainer,
            core::BlankLine,
            paragraph::ParagraphBlock,
            verbatim::{VerbatimBlock, VerbatimType},
        };

        // Create a document with some content
        let paragraph = ParagraphBlock {
            content: vec![], // Empty TextTransform content for test
            annotations: vec![],
            parameters: crate::ast::parameters::Parameters::default(),
            tokens: crate::ast::tokens::TokenSequence::new(),
        };

        let verbatim = VerbatimBlock {
            title: vec![], // Empty TextTransform title for test
            content: IgnoreContainer {
                ignore_lines: vec![], // Empty ignore lines for test
                blank_lines: vec![],  // Empty blank lines for test
                annotations: vec![],
                parameters: crate::ast::parameters::Parameters::default(),
                tokens: crate::ast::tokens::TokenSequence::new(),
            },
            label: "test".to_string(), // Mandatory label
            verbatim_type: VerbatimType::InFlow,
            annotations: vec![],
            parameters: crate::ast::parameters::Parameters::default(),
            tokens: crate::ast::tokens::TokenSequence::new(),
        };

        let blank_line = BlankLine {
            tokens: crate::ast::tokens::TokenSequence::new(),
        };

        let document = Document {
            meta: Meta {
                title: Some(crate::ast::base::MetaValue::String(
                    "Test Document with Content".to_string(),
                )),
                ..Meta::default()
            },
            content: SessionContainer {
                content: vec![
                    SessionContainerElement::Paragraph(paragraph),
                    SessionContainerElement::BlankLine(blank_line),
                    SessionContainerElement::Verbatim(verbatim),
                ],
                annotations: vec![],
                parameters: crate::ast::parameters::Parameters::default(),
                tokens: crate::ast::tokens::TokenSequence::new(),
            },
            assembly_info: AssemblyInfo {
                parser_version: "test".to_string(),
                source_path: None,
                processed_at: None,
                stats: crate::ast::base::ProcessingStats::default(),
            },
        };

        // Create traversable document
        let traversable = TraversableDocument::from_document(&document);

        // Test that the tree has been built correctly
        let root = traversable.root();
        assert_eq!(root.value().element_type, ElementType::Container);

        // Check that we have child elements
        let children: Vec<_> = root.children().collect();
        assert_eq!(children.len(), 3); // paragraph, blank line, verbatim

        // Test element types
        let child_types: Vec<_> = children
            .iter()
            .map(|child| child.value().element_type.clone())
            .collect();
        assert_eq!(
            child_types,
            vec![ElementType::Block, ElementType::Line, ElementType::Block]
        );

        // Test query functionality on the built tree
        let blocks = traversable
            .query()
            .find_by_type(ElementType::Block)
            .collect();
        assert_eq!(blocks.len(), 2); // paragraph and verbatim are blocks

        let lines = traversable
            .query()
            .find_by_type(ElementType::Line)
            .collect();
        assert_eq!(lines.len(), 1); // blank line is a line

        let containers = traversable
            .query()
            .find_by_type(ElementType::Container)
            .collect();
        assert_eq!(containers.len(), 1); // Just the root document container
    }

    #[test]
    fn test_text_search_functionality() {
        use crate::ast::elements::paragraph::ParagraphBlock;

        // Create a document with content for text search testing
        let paragraph = ParagraphBlock {
            content: vec![], // Empty TextTransform content for test
            annotations: vec![],
            parameters: crate::ast::parameters::Parameters::default(),
            tokens: crate::ast::tokens::TokenSequence::new(),
        };

        let document = Document {
            meta: Meta {
                title: Some(crate::ast::base::MetaValue::String(
                    "Test Document".to_string(),
                )),
                ..Meta::default()
            },
            content: SessionContainer {
                content: vec![SessionContainerElement::Paragraph(paragraph)],
                annotations: vec![],
                parameters: crate::ast::parameters::Parameters::default(),
                tokens: crate::ast::tokens::TokenSequence::new(),
            },
            assembly_info: AssemblyInfo {
                parser_version: "test".to_string(),
                source_path: None,
                processed_at: None,
                stats: crate::ast::base::ProcessingStats::default(),
            },
        };

        let traversable = TraversableDocument::from_document(&document);

        // Test text search - since we have basic text extraction, this should work
        let block_search = traversable.query().text_contains("Block").collect();
        assert!(!block_search.is_empty()); // Should find our block elements

        let container_search = traversable.query().text_contains("Container").collect();
        assert!(!container_search.is_empty()); // Should find our container elements
    }
}
