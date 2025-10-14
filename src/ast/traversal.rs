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
    elements::core::{ElementType, TxxtElement},
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
        let id_counter = 0;

        // For now, create a simple tree with just a document root
        // We'll expand this to properly traverse the document structure
        let root_wrapper = ElementWrapper::new(
            Box::new(DocumentElementOwned::from_document(document)),
            id_counter,
        );
        let _id_counter = id_counter + 1;

        let tree = Tree::new(root_wrapper);
        let node_cache = HashMap::new();

        // TODO: Build the tree by traversing the document structure
        // This would involve converting each SessionContainerElement to ElementWrapper
        // and building the ego-tree structure recursively

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

    /// Build the ego-tree recursively from the AST structure
    #[allow(dead_code)]
    fn build_tree_recursive(
        _container_element: &dyn TxxtElement,
        _parent_node: &mut ego_tree::NodeMut<ElementWrapper>,
        _id_counter: &mut ElementId,
        _node_cache: &mut HashMap<ElementId, ego_tree::NodeId>,
    ) {
        // This is a simplified implementation - we need to properly handle
        // the different container types and their children

        // TODO: Implement proper recursive tree building based on
        // SessionContainerElement and ContentContainerElement variants
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
    fn extract_text_content(&self, _node: NodeRef<ElementWrapper>) -> String {
        // TODO: Implement text extraction based on element type
        // This would use the visitor pattern or direct access to text content
        // For now, return empty string as placeholder
        String::new()
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
}
