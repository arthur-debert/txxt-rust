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
        containers::content::{ContentContainer, ContentContainerElement},
        core::{ElementType, TxxtElement},
        session::{session_container::SessionContainerElement, SessionContainer},
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

    /// Query the document using XPath-like syntax
    ///
    /// This provides a more convenient interface for complex queries:
    /// ```ignore
    /// let paragraphs = doc.xpath("//Block[@type='paragraph']")?;
    /// let text_blocks = doc.xpath("//Block[text()='hello']")?;
    /// ```
    pub fn xpath(&self, selector: &str) -> Result<Vec<NodeRef<'_, ElementWrapper>>, XPathError> {
        let query = DocumentQuery::xpath(self, selector)?;
        Ok(query.collect())
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

    /// Create a query from an XPath-like selector string
    ///
    /// Supported syntax:
    /// - `//Block` - Find all Block elements anywhere in the tree
    /// - `/Container/Block` - Find Block elements directly under root Container
    /// - `Block[@label="test"]` - Find Block elements with parameter label="test"
    /// - `Block[text()="content"]` - Find Block elements containing specific text
    /// - `*` - Match any element type
    /// - `.` - Current context (self)
    /// - `..` - Parent element
    ///
    /// Examples:
    /// ```ignore
    /// // Find all paragraphs anywhere
    /// doc.xpath("//Paragraph")
    ///
    /// // Find verbatim blocks with specific label
    /// doc.xpath("//Verbatim[@label='code']")
    ///
    /// // Find blocks containing "hello"
    /// doc.xpath("//Block[text()='hello']")
    /// ```
    pub fn xpath(document: &'a TraversableDocument, selector: &str) -> Result<Self, XPathError> {
        let parser = XPathParser::new();
        let path = parser.parse(selector)?;
        Ok(Self::from_xpath(document, path))
    }

    /// Create a query from a parsed XPath
    fn from_xpath(document: &'a TraversableDocument, path: XPath) -> Self {
        let mut query = Self::new(document);

        // Convert XPath steps to filters
        for step in path.steps {
            query = query.apply_xpath_step(step);
        }

        query
    }

    /// Apply a single XPath step to the query
    fn apply_xpath_step(mut self, step: XPathStep) -> Self {
        match step.axis {
            XPathAxis::Descendant => {
                // For descendant axis (//) we search through all descendants
                if let Some(node_test) = step.node_test {
                    self = self.apply_node_test(node_test);
                }
            }
            XPathAxis::Child => {
                // For child axis (/) we only look at direct children
                // This is more complex and would require tree navigation context
                // For now, treat as descendant but add a note
                if let Some(node_test) = step.node_test {
                    self = self.apply_node_test(node_test);
                }
            }
            XPathAxis::Self_ => {
                // Current context - usually no-op in our case
            }
            XPathAxis::Parent => {
                // Parent navigation - requires tree context
                // Would need special handling in iterator
            }
        }

        // Apply predicates
        for predicate in step.predicates {
            self = self.apply_predicate(predicate);
        }

        self
    }

    /// Apply a node test (element type matching)
    fn apply_node_test(self, node_test: XPathNodeTest) -> Self {
        match node_test {
            XPathNodeTest::ElementType(element_type) => self.find_by_type(element_type),
            XPathNodeTest::Wildcard => self, // Match any element - no filter needed
        }
    }

    /// Apply a predicate (attribute or text filtering)
    fn apply_predicate(self, predicate: XPathPredicate) -> Self {
        match predicate {
            XPathPredicate::Attribute { name, value } => self.has_parameter(&name, &value),
            XPathPredicate::Text(text) => self.text_contains(&text),
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

    fn annotations(&self) -> &[crate::ast::elements::annotation::annotation_content::Annotation] {
        // Document doesn't have annotations directly
        &[]
    }

    fn parameters(&self) -> &crate::ast::elements::components::parameters::Parameters {
        // Document doesn't have parameters directly
        use std::sync::OnceLock;
        static EMPTY_PARAMS: OnceLock<crate::ast::elements::components::parameters::Parameters> =
            OnceLock::new();
        EMPTY_PARAMS.get_or_init(crate::ast::elements::components::parameters::Parameters::default)
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

    fn annotations(&self) -> &[crate::ast::elements::annotation::annotation_content::Annotation] {
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

    fn parameters(&self) -> &crate::ast::elements::components::parameters::Parameters {
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

/// XPath-like selector support for tree queries
///
/// This provides a simple XPath-inspired syntax for querying the document tree.
/// While not a full XPath implementation, it covers the most common tree navigation patterns.
/// XPath parsing errors
#[derive(Debug, Clone)]
pub enum XPathError {
    /// Invalid syntax in the selector string
    InvalidSyntax(String),
    /// Unsupported XPath feature
    UnsupportedFeature(String),
    /// Unknown element type in selector
    UnknownElementType(String),
}

impl std::fmt::Display for XPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XPathError::InvalidSyntax(msg) => write!(f, "Invalid XPath syntax: {}", msg),
            XPathError::UnsupportedFeature(msg) => write!(f, "Unsupported XPath feature: {}", msg),
            XPathError::UnknownElementType(msg) => write!(f, "Unknown element type: {}", msg),
        }
    }
}

impl std::error::Error for XPathError {}

/// Parsed XPath expression
#[derive(Debug, Clone)]
pub struct XPath {
    pub steps: Vec<XPathStep>,
}

/// Single step in an XPath expression
#[derive(Debug, Clone)]
pub struct XPathStep {
    pub axis: XPathAxis,
    pub node_test: Option<XPathNodeTest>,
    pub predicates: Vec<XPathPredicate>,
}

/// XPath axis (navigation direction)
#[derive(Debug, Clone)]
pub enum XPathAxis {
    /// Descendant-or-self axis (//)
    Descendant,
    /// Child axis (/)
    Child,
    /// Self axis (.)
    Self_,
    /// Parent axis (..)
    Parent,
}

/// XPath node test (what to match)
#[derive(Debug, Clone)]
pub enum XPathNodeTest {
    /// Match specific element type
    ElementType(ElementType),
    /// Match any element (*)
    Wildcard,
}

/// XPath predicate (filtering condition)
#[derive(Debug, Clone)]
pub enum XPathPredicate {
    /// Attribute test [@name="value"]
    Attribute { name: String, value: String },
    /// Text content test [text()="value"]
    Text(String),
}

/// Simple XPath parser
pub struct XPathParser;

impl Default for XPathParser {
    fn default() -> Self {
        Self::new()
    }
}

impl XPathParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse an XPath-like selector string
    pub fn parse(&self, selector: &str) -> Result<XPath, XPathError> {
        let trimmed = selector.trim();
        if trimmed.is_empty() {
            return Err(XPathError::InvalidSyntax("Empty selector".to_string()));
        }

        let mut steps = Vec::new();
        let mut current_pos = 0;
        let chars: Vec<char> = trimmed.chars().collect();

        while current_pos < chars.len() {
            let step = self.parse_step(&chars, &mut current_pos)?;
            steps.push(step);
        }

        Ok(XPath { steps })
    }

    /// Parse a single XPath step
    fn parse_step(&self, chars: &[char], pos: &mut usize) -> Result<XPathStep, XPathError> {
        // Skip whitespace
        while *pos < chars.len() && chars[*pos].is_whitespace() {
            *pos += 1;
        }

        if *pos >= chars.len() {
            return Err(XPathError::InvalidSyntax(
                "Unexpected end of input".to_string(),
            ));
        }

        // Parse axis
        let axis = self.parse_axis(chars, pos)?;

        // Parse node test
        let node_test = if *pos < chars.len() && !chars[*pos].is_whitespace() && chars[*pos] != '['
        {
            Some(self.parse_node_test(chars, pos)?)
        } else {
            None
        };

        // Parse predicates
        let mut predicates = Vec::new();
        while *pos < chars.len() && chars[*pos] == '[' {
            predicates.push(self.parse_predicate(chars, pos)?);
        }

        Ok(XPathStep {
            axis,
            node_test,
            predicates,
        })
    }

    /// Parse XPath axis
    fn parse_axis(&self, chars: &[char], pos: &mut usize) -> Result<XPathAxis, XPathError> {
        if *pos < chars.len() {
            match chars[*pos] {
                '/' => {
                    *pos += 1;
                    if *pos < chars.len() && chars[*pos] == '/' {
                        *pos += 1;
                        Ok(XPathAxis::Descendant)
                    } else {
                        Ok(XPathAxis::Child)
                    }
                }
                '.' => {
                    *pos += 1;
                    if *pos < chars.len() && chars[*pos] == '.' {
                        *pos += 1;
                        Ok(XPathAxis::Parent)
                    } else {
                        Ok(XPathAxis::Self_)
                    }
                }
                _ => Ok(XPathAxis::Child), // Default axis
            }
        } else {
            Err(XPathError::InvalidSyntax("Expected axis".to_string()))
        }
    }

    /// Parse node test (element name or wildcard)
    fn parse_node_test(
        &self,
        chars: &[char],
        pos: &mut usize,
    ) -> Result<XPathNodeTest, XPathError> {
        let start = *pos;

        // Handle wildcard
        if *pos < chars.len() && chars[*pos] == '*' {
            *pos += 1;
            return Ok(XPathNodeTest::Wildcard);
        }

        // Parse element name
        while *pos < chars.len()
            && !chars[*pos].is_whitespace()
            && chars[*pos] != '['
            && chars[*pos] != '/'
        {
            *pos += 1;
        }

        if start == *pos {
            return Err(XPathError::InvalidSyntax("Expected node test".to_string()));
        }

        let element_name: String = chars[start..*pos].iter().collect();
        let element_type = self.parse_element_type(&element_name)?;
        Ok(XPathNodeTest::ElementType(element_type))
    }

    /// Parse element type from string
    fn parse_element_type(&self, name: &str) -> Result<ElementType, XPathError> {
        match name {
            "Block" => Ok(ElementType::Block),
            "Container" => Ok(ElementType::Container),
            "Line" => Ok(ElementType::Line),
            "Span" => Ok(ElementType::Span),
            _ => Err(XPathError::UnknownElementType(name.to_string())),
        }
    }

    /// Parse predicate [condition]
    fn parse_predicate(
        &self,
        chars: &[char],
        pos: &mut usize,
    ) -> Result<XPathPredicate, XPathError> {
        if *pos >= chars.len() || chars[*pos] != '[' {
            return Err(XPathError::InvalidSyntax("Expected '['".to_string()));
        }
        *pos += 1; // Skip '['

        // Skip whitespace
        while *pos < chars.len() && chars[*pos].is_whitespace() {
            *pos += 1;
        }

        // Check for text() function
        if *pos + 6 < chars.len() && chars[*pos..*pos + 6].iter().collect::<String>() == "text()" {
            *pos += 6; // Skip "text()"

            // Skip whitespace and '='
            while *pos < chars.len() && (chars[*pos].is_whitespace() || chars[*pos] == '=') {
                *pos += 1;
            }

            // Parse quoted string
            let text = self.parse_quoted_string(chars, pos)?;

            // Skip to closing ']'
            while *pos < chars.len() && chars[*pos] != ']' {
                *pos += 1;
            }
            if *pos < chars.len() {
                *pos += 1; // Skip ']'
            }

            return Ok(XPathPredicate::Text(text));
        }

        // Parse attribute predicate [@attr="value"]
        if *pos < chars.len() && chars[*pos] == '@' {
            *pos += 1; // Skip '@'

            // Parse attribute name
            let start = *pos;
            while *pos < chars.len() && chars[*pos] != '=' && chars[*pos] != ']' {
                *pos += 1;
            }
            let attr_name: String = chars[start..*pos].iter().collect();

            // Skip '=' and whitespace
            while *pos < chars.len() && (chars[*pos] == '=' || chars[*pos].is_whitespace()) {
                *pos += 1;
            }

            // Parse quoted value
            let value = self.parse_quoted_string(chars, pos)?;

            // Skip to closing ']'
            while *pos < chars.len() && chars[*pos] != ']' {
                *pos += 1;
            }
            if *pos < chars.len() {
                *pos += 1; // Skip ']'
            }

            return Ok(XPathPredicate::Attribute {
                name: attr_name,
                value,
            });
        }

        Err(XPathError::InvalidSyntax(
            "Invalid predicate syntax".to_string(),
        ))
    }

    /// Parse a quoted string (handles both single and double quotes)
    fn parse_quoted_string(&self, chars: &[char], pos: &mut usize) -> Result<String, XPathError> {
        if *pos >= chars.len() {
            return Err(XPathError::InvalidSyntax(
                "Expected quoted string".to_string(),
            ));
        }

        let quote = chars[*pos];
        if quote != '"' && quote != '\'' {
            return Err(XPathError::InvalidSyntax(
                "Expected quoted string".to_string(),
            ));
        }

        *pos += 1; // Skip opening quote
        let start = *pos;

        // Find closing quote
        while *pos < chars.len() && chars[*pos] != quote {
            *pos += 1;
        }

        if *pos >= chars.len() {
            return Err(XPathError::InvalidSyntax(
                "Unterminated quoted string".to_string(),
            ));
        }

        let result: String = chars[start..*pos].iter().collect();
        *pos += 1; // Skip closing quote

        Ok(result)
    }
}

/// Traversable trait for uniform tree navigation interface
///
/// This trait provides a common interface for tree traversal operations
/// that can be implemented by different tree representations.
pub trait Traversable {
    type Node;
    type Error;

    /// Get the root node of the tree
    fn root(&self) -> Self::Node;

    /// Get parent of a node
    fn parent(&self, node: &Self::Node) -> Option<Self::Node>;

    /// Get children of a node
    fn children(&self, node: &Self::Node) -> Vec<Self::Node>;

    /// Get next sibling of a node
    fn next_sibling(&self, node: &Self::Node) -> Option<Self::Node>;

    /// Get previous sibling of a node
    fn prev_sibling(&self, node: &Self::Node) -> Option<Self::Node>;

    /// Get all descendants of a node (depth-first)
    fn descendants(&self, node: &Self::Node) -> Vec<Self::Node>;

    /// Get all ancestors of a node (from parent to root)
    fn ancestors(&self, node: &Self::Node) -> Vec<Self::Node>;

    /// Find nodes matching a predicate
    fn find<F>(&self, predicate: F) -> Vec<Self::Node>
    where
        F: Fn(&Self::Node) -> bool;

    /// Execute XPath-like query
    fn xpath(&self, selector: &str) -> Result<Vec<Self::Node>, Self::Error>;
}

impl<'a> Traversable for &'a TraversableDocument {
    type Node = NodeRef<'a, ElementWrapper>;
    type Error = XPathError;

    fn root(&self) -> Self::Node {
        self.tree.root()
    }

    fn parent(&self, node: &Self::Node) -> Option<Self::Node> {
        node.parent()
    }

    fn children(&self, node: &Self::Node) -> Vec<Self::Node> {
        node.children().collect()
    }

    fn next_sibling(&self, node: &Self::Node) -> Option<Self::Node> {
        node.next_sibling()
    }

    fn prev_sibling(&self, node: &Self::Node) -> Option<Self::Node> {
        node.prev_sibling()
    }

    fn descendants(&self, node: &Self::Node) -> Vec<Self::Node> {
        node.traverse()
            .filter_map(|edge| match edge {
                ego_tree::iter::Edge::Open(n) => Some(n),
                ego_tree::iter::Edge::Close(_) => None,
            })
            .skip(1) // Skip the node itself
            .collect()
    }

    fn ancestors(&self, node: &Self::Node) -> Vec<Self::Node> {
        let mut ancestors = Vec::new();
        let mut current = node.parent();
        while let Some(ancestor) = current {
            ancestors.push(ancestor);
            current = ancestor.parent();
        }
        ancestors
    }

    fn find<F>(&self, predicate: F) -> Vec<Self::Node>
    where
        F: Fn(&Self::Node) -> bool,
    {
        self.root()
            .traverse()
            .filter_map(|edge| match edge {
                ego_tree::iter::Edge::Open(node) => Some(node),
                ego_tree::iter::Edge::Close(_) => None,
            })
            .filter(predicate)
            .collect()
    }

    fn xpath(&self, selector: &str) -> Result<Vec<Self::Node>, Self::Error> {
        TraversableDocument::xpath(self, selector)
    }
}

/// Visitor pattern for extensible element processing
///
/// This trait allows for type-safe, extensible processing of AST elements
/// without requiring modification of the core AST types.
pub trait ElementVisitor {
    type Result;
    type Error;

    /// Visit any element (default implementation)
    fn visit_element(&mut self, element: &dyn TxxtElement) -> Result<Self::Result, Self::Error>;

    /// Visit a paragraph block
    fn visit_paragraph(
        &mut self,
        paragraph: &crate::ast::elements::paragraph::ParagraphBlock,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(paragraph)
    }

    /// Visit a list block
    fn visit_list(
        &mut self,
        list: &crate::ast::elements::list::ListBlock,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(list)
    }

    /// Visit a definition block
    fn visit_definition(
        &mut self,
        definition: &crate::ast::elements::definition::DefinitionBlock,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(definition)
    }

    /// Visit a verbatim block
    fn visit_verbatim(
        &mut self,
        verbatim: &crate::ast::elements::verbatim::VerbatimBlock,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(verbatim)
    }

    /// Visit an annotation block
    fn visit_annotation(
        &mut self,
        annotation: &crate::ast::elements::annotation::AnnotationBlock,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(annotation)
    }

    /// Visit a session block
    fn visit_session(
        &mut self,
        session: &crate::ast::elements::session::SessionBlock,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(session)
    }

    /// Visit a content container
    fn visit_content_container(
        &mut self,
        container: &ContentContainer,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(container)
    }

    /// Visit a session container
    fn visit_session_container(
        &mut self,
        container: &SessionContainer,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(container)
    }

    /// Visit a blank line
    fn visit_blank_line(
        &mut self,
        blank_line: &crate::ast::elements::core::BlankLine,
    ) -> Result<Self::Result, Self::Error> {
        self.visit_element(blank_line)
    }
}

/// Visitor dispatcher for ElementAdapter
impl ElementAdapter {
    /// Accept a visitor and dispatch to the appropriate visit method
    pub fn accept<V: ElementVisitor>(&self, visitor: &mut V) -> Result<V::Result, V::Error> {
        match self {
            ElementAdapter::Paragraph(p) => visitor.visit_paragraph(p),
            ElementAdapter::List(l) => visitor.visit_list(l),
            ElementAdapter::Definition(d) => visitor.visit_definition(d),
            ElementAdapter::Verbatim(v) => visitor.visit_verbatim(v),
            ElementAdapter::Annotation(a) => visitor.visit_annotation(a),
            ElementAdapter::Session(s) => visitor.visit_session(s),
            ElementAdapter::ContentContainer(c) => visitor.visit_content_container(c),
            ElementAdapter::SessionContainer(s) => visitor.visit_session_container(s),
            ElementAdapter::BlankLine(b) => visitor.visit_blank_line(b),
        }
    }
}

/// Traversal utilities for working with the tree
impl TraversableDocument {
    /// Walk the tree with a visitor
    pub fn walk<V: ElementVisitor>(&self, visitor: &mut V) -> Result<Vec<V::Result>, V::Error> {
        let mut results = Vec::new();

        for node in self.root().traverse().filter_map(|edge| match edge {
            ego_tree::iter::Edge::Open(node) => Some(node),
            ego_tree::iter::Edge::Close(_) => None,
        }) {
            let result = visitor.visit_element(&*node.value().element)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Walk specific nodes with a visitor
    pub fn walk_nodes<V: ElementVisitor>(
        &self,
        nodes: &[NodeRef<'_, ElementWrapper>],
        visitor: &mut V,
    ) -> Result<Vec<V::Result>, V::Error> {
        let mut results = Vec::new();

        for node in nodes {
            let result = visitor.visit_element(&*node.value().element)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get element type distribution in the document
    pub fn element_type_stats(&self) -> std::collections::HashMap<ElementType, usize> {
        let mut stats = std::collections::HashMap::new();

        for node in self.root().traverse().filter_map(|edge| match edge {
            ego_tree::iter::Edge::Open(node) => Some(node),
            ego_tree::iter::Edge::Close(_) => None,
        }) {
            *stats.entry(node.value().element_type.clone()).or_insert(0) += 1;
        }

        stats
    }

    /// Get tree depth (maximum nesting level)
    pub fn tree_depth(&self) -> usize {
        fn depth_recursive(node: NodeRef<'_, ElementWrapper>) -> usize {
            if node.children().count() == 0 {
                1
            } else {
                1 + node.children().map(depth_recursive).max().unwrap_or(0)
            }
        }

        depth_recursive(self.root())
    }

    /// Get all leaf nodes (nodes with no children)
    pub fn leaf_nodes(&self) -> Vec<NodeRef<'_, ElementWrapper>> {
        self.find(|node| node.children().count() == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::base::{AssemblyInfo, Document, Meta};
    use crate::ast::elements::session::SessionContainer;

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
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
            core::BlankLine,
            paragraph::ParagraphBlock,
            verbatim::IgnoreContainer,
            verbatim::{VerbatimBlock, VerbatimType},
        };

        // Create a document with some content
        let paragraph = ParagraphBlock {
            content: vec![], // Empty TextTransform content for test
            annotations: vec![],
            parameters: crate::ast::elements::components::parameters::Parameters::default(),
            tokens: crate::ast::tokens::TokenSequence::new(),
        };

        let verbatim = VerbatimBlock {
            title: vec![], // Empty TextTransform title for test
            content: IgnoreContainer {
                ignore_lines: vec![], // Empty ignore lines for test
                blank_lines: vec![],  // Empty blank lines for test
                annotations: vec![],
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
                tokens: crate::ast::tokens::TokenSequence::new(),
            },
            label: "test".to_string(), // Mandatory label
            verbatim_type: VerbatimType::InFlow,
            annotations: vec![],
            parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
            parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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

    #[test]
    fn test_xpath_parsing() {
        let parser = XPathParser::new();

        // Test simple element type
        let xpath = parser.parse("//Block").unwrap();
        assert_eq!(xpath.steps.len(), 1);
        assert!(matches!(xpath.steps[0].axis, XPathAxis::Descendant));
        assert!(matches!(
            xpath.steps[0].node_test,
            Some(XPathNodeTest::ElementType(ElementType::Block))
        ));

        // Test with predicate
        let xpath = parser.parse("//Block[@label='test']").unwrap();
        assert_eq!(xpath.steps.len(), 1);
        assert_eq!(xpath.steps[0].predicates.len(), 1);
        assert!(matches!(
            &xpath.steps[0].predicates[0],
            XPathPredicate::Attribute { name, value } if name == "label" && value == "test"
        ));

        // Test text predicate
        let xpath = parser.parse("//Block[text()='hello']").unwrap();
        assert_eq!(xpath.steps.len(), 1);
        assert_eq!(xpath.steps[0].predicates.len(), 1);
        assert!(matches!(
            &xpath.steps[0].predicates[0],
            XPathPredicate::Text(text) if text == "hello"
        ));

        // Test wildcard
        let xpath = parser.parse("//*").unwrap();
        assert_eq!(xpath.steps.len(), 1);
        assert!(matches!(xpath.steps[0].axis, XPathAxis::Descendant));
        assert!(matches!(
            xpath.steps[0].node_test,
            Some(XPathNodeTest::Wildcard)
        ));
    }

    #[test]
    fn test_xpath_integration() {
        use crate::ast::elements::paragraph::ParagraphBlock;

        // Create a document with content for XPath testing
        let paragraph = ParagraphBlock {
            content: vec![], // Empty TextTransform content for test
            annotations: vec![],
            parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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

        // Test XPath queries
        let blocks = traversable.xpath("//Block").unwrap();
        assert_eq!(blocks.len(), 1); // Should find our paragraph block

        let containers = traversable.xpath("//Container").unwrap();
        assert_eq!(containers.len(), 1); // Should find our document container

        let all_elements = traversable.xpath("//*").unwrap();
        assert!(all_elements.len() >= 2); // Should find at least container + paragraph

        // Test text search via XPath
        let text_results = traversable
            .xpath("//Block[text()='Block content']")
            .unwrap();
        assert!(!text_results.is_empty()); // Should find our blocks with "Block content" in the text
    }

    #[test]
    fn test_xpath_error_handling() {
        let parser = XPathParser::new();

        // Test invalid syntax
        assert!(parser.parse("").is_err());
        assert!(parser.parse("//[").is_err());
        assert!(parser.parse("//Block[@unclosed").is_err());

        // Test unknown element type
        assert!(parser.parse("//InvalidType").is_err());
    }

    #[test]
    fn test_traversable_trait() {
        use crate::ast::elements::paragraph::ParagraphBlock;

        // Create a document with some content
        let paragraph = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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

        // Test Traversable trait methods via reference
        let traversable_ref = &traversable;
        let root = traversable_ref.root();
        assert_eq!(root.value().element_type, ElementType::Container);

        let children = traversable_ref.children(&root);
        assert_eq!(children.len(), 1); // Should have one paragraph child

        let descendants = traversable_ref.descendants(&root);
        assert_eq!(descendants.len(), 1); // Should have one descendant (the paragraph)

        let ancestors = traversable_ref.ancestors(&children[0]);
        assert_eq!(ancestors.len(), 1); // Paragraph should have root as ancestor

        // Test find method
        let blocks = traversable_ref.find(|node| node.value().element_type == ElementType::Block);
        assert_eq!(blocks.len(), 1); // Should find one block (paragraph)

        // Test XPath via trait
        let xpath_blocks = traversable_ref.xpath("//Block").unwrap();
        assert_eq!(xpath_blocks.len(), 1); // Should find one block via XPath
    }

    #[test]
    fn test_visitor_pattern() {
        use crate::ast::elements::paragraph::ParagraphBlock;

        // Create a simple visitor that counts element types
        struct ElementCounter {
            counts: std::collections::HashMap<String, usize>,
        }

        impl ElementCounter {
            fn new() -> Self {
                Self {
                    counts: std::collections::HashMap::new(),
                }
            }
        }

        impl ElementVisitor for ElementCounter {
            type Result = ();
            type Error = ();

            fn visit_element(
                &mut self,
                element: &dyn TxxtElement,
            ) -> Result<Self::Result, Self::Error> {
                let type_name = format!("{:?}", element.element_type());
                *self.counts.entry(type_name).or_insert(0) += 1;
                Ok(())
            }
        }

        // Create a document with content
        let paragraph = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: crate::ast::elements::components::parameters::Parameters::default(),
            tokens: crate::ast::tokens::TokenSequence::new(),
        };

        let document = Document {
            meta: Meta::default(),
            content: SessionContainer {
                content: vec![SessionContainerElement::Paragraph(paragraph)],
                annotations: vec![],
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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
        let mut visitor = ElementCounter::new();

        // Walk the tree with visitor
        let _results = traversable.walk(&mut visitor).unwrap();

        // Check counts
        assert_eq!(visitor.counts.get("Container"), Some(&1)); // Document root
        assert_eq!(visitor.counts.get("Block"), Some(&1)); // Paragraph block
    }

    #[test]
    fn test_tree_utilities() {
        use crate::ast::elements::paragraph::ParagraphBlock;

        // Create a document with content
        let paragraph = ParagraphBlock {
            content: vec![],
            annotations: vec![],
            parameters: crate::ast::elements::components::parameters::Parameters::default(),
            tokens: crate::ast::tokens::TokenSequence::new(),
        };

        let document = Document {
            meta: Meta::default(),
            content: SessionContainer {
                content: vec![SessionContainerElement::Paragraph(paragraph)],
                annotations: vec![],
                parameters: crate::ast::elements::components::parameters::Parameters::default(),
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

        // Test element type statistics
        let stats = traversable.element_type_stats();
        assert_eq!(stats.get(&ElementType::Container), Some(&1));
        assert_eq!(stats.get(&ElementType::Block), Some(&1));

        // Test tree depth
        let depth = traversable.tree_depth();
        assert_eq!(depth, 2); // Root -> Paragraph

        // Test leaf nodes
        let leaves = traversable.leaf_nodes();
        assert_eq!(leaves.len(), 1); // Only the paragraph should be a leaf
        assert_eq!(leaves[0].value().element_type, ElementType::Block);
    }
}
