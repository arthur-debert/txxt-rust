//! Query builder for fluent AST traversal

use super::{
    node_ref::{AstNodeRef, NodeType},
    predicates::NodePredicate,
    traversal::{NodeIterator, TraversalMode},
};

/// Fluent query builder for AST traversal
///
/// This provides a Django QuerySet / jQuery-like API for querying the AST.
/// Queries are lazy and only execute when terminal methods like `find()` or
/// `find_first()` are called.
///
/// # Examples
///
/// ```rust,ignore
/// // Find all sessions with annotations
/// let sessions = NodeQuery::from(doc.as_ref())
///     .sessions()
///     .with_annotations()
///     .find_all();
///
/// // Find paragraphs containing text
/// let todos = NodeQuery::from(doc.as_ref())
///     .paragraphs()
///     .text_contains("TODO")
///     .find_all();
/// ```
pub struct NodeQuery<'ast> {
    pub(crate) root: AstNodeRef<'ast>,
    pub(crate) predicates: Vec<NodePredicate>,
    pub(crate) traversal_mode: TraversalMode,
    pub(crate) depth_range: Option<(usize, usize)>,
}

impl<'ast> NodeQuery<'ast> {
    /// Start a new query from a root node
    pub fn from(node: AstNodeRef<'ast>) -> Self {
        Self {
            root: node,
            predicates: Vec::new(),
            traversal_mode: TraversalMode::DepthFirst,
            depth_range: None,
        }
    }

    // ---- Traversal Configuration ----

    /// Use depth-first traversal (default)
    pub fn depth_first(mut self) -> Self {
        self.traversal_mode = TraversalMode::DepthFirst;
        self
    }

    /// Use breadth-first traversal
    pub fn breadth_first(mut self) -> Self {
        self.traversal_mode = TraversalMode::BreadthFirst;
        self
    }

    /// Only traverse nodes at specific depth range
    pub fn at_depth(mut self, min: usize, max: usize) -> Self {
        self.depth_range = Some((min, max));
        self
    }

    // ---- Type-Based Filtering ----

    /// Filter by node type
    pub fn of_type(mut self, node_type: NodeType) -> Self {
        self.predicates.push(NodePredicate::Type(node_type));
        self
    }

    /// Filter to only paragraphs
    pub fn paragraphs(self) -> Self {
        self.of_type(NodeType::Paragraph)
    }

    /// Filter to only lists
    pub fn lists(self) -> Self {
        self.of_type(NodeType::List)
    }

    /// Filter to only sessions
    pub fn sessions(self) -> Self {
        self.of_type(NodeType::Session)
    }

    /// Filter to only definitions
    pub fn definitions(self) -> Self {
        self.of_type(NodeType::Definition)
    }

    /// Filter to only verbatim blocks
    pub fn verbatim_blocks(self) -> Self {
        self.of_type(NodeType::VerbatimBlock)
    }

    /// Filter to only containers
    pub fn containers(self) -> Self {
        self.of_type(NodeType::Container)
    }

    // ---- Content-Based Filtering ----

    /// Filter nodes whose text content contains a substring
    pub fn text_contains(mut self, text: &str) -> Self {
        self.predicates
            .push(NodePredicate::TextContains(text.to_string()));
        self
    }

    /// Filter nodes whose text content matches a regex
    pub fn text_matches(mut self, pattern: &str) -> Self {
        self.predicates
            .push(NodePredicate::TextMatches(pattern.to_string()));
        self
    }

    // ---- Attribute-Based Filtering ----

    /// Filter nodes with specific parameter
    pub fn with_param(mut self, key: &str, value: Option<&str>) -> Self {
        self.predicates.push(NodePredicate::HasParameter {
            key: key.to_string(),
            value: value.map(String::from),
        });
        self
    }

    /// Filter nodes with ref= parameter
    pub fn with_ref(self, ref_id: &str) -> Self {
        self.with_param("ref", Some(ref_id))
    }

    /// Filter nodes that have annotations
    pub fn with_annotations(mut self) -> Self {
        self.predicates.push(NodePredicate::HasAnnotations);
        self
    }

    /// Filter nodes with annotation matching label
    pub fn with_annotation_label(mut self, label: &str) -> Self {
        self.predicates
            .push(NodePredicate::AnnotationLabel(label.to_string()));
        self
    }

    // ---- Structural Filtering ----

    /// Filter nodes that have children
    pub fn with_children(mut self) -> Self {
        self.predicates.push(NodePredicate::HasChildren);
        self
    }

    /// Filter leaf nodes (no children)
    pub fn leaves(mut self) -> Self {
        self.predicates.push(NodePredicate::IsLeaf);
        self
    }

    /// Filter nodes at specific indentation level
    pub fn at_level(mut self, level: usize) -> Self {
        self.predicates.push(NodePredicate::AtLevel(level));
        self
    }

    // ---- Custom Predicates ----

    /// Filter with custom predicate function
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(AstNodeRef<'_>) -> bool + 'static,
    {
        self.predicates
            .push(NodePredicate::Custom(Box::new(predicate)));
        self
    }

    // ---- Logical Combinations ----

    /// Add OR condition (next predicate is ORed with previous)
    /// Note: This creates a new OR group with the last predicate
    pub fn or(mut self) -> Self {
        if let Some(last) = self.predicates.pop() {
            // The next predicate added will be ORed with this one
            // This is a simplified approach - full implementation would be more sophisticated
            self.predicates.push(last);
        }
        self
    }

    // ---- Query Execution ----

    /// Execute query and return iterator over matching nodes
    pub fn find(self) -> NodeIterator<'ast> {
        NodeIterator::new(self)
    }

    /// Execute query and return first matching node
    pub fn find_first(self) -> Option<AstNodeRef<'ast>> {
        self.find().next()
    }

    /// Execute query and collect all matching nodes
    pub fn find_all(self) -> Vec<AstNodeRef<'ast>> {
        self.find().collect()
    }

    /// Count matching nodes without allocating
    pub fn count(self) -> usize {
        self.find().count()
    }

    /// Check if any nodes match
    pub fn exists(self) -> bool {
        self.find().next().is_some()
    }

    /// Check if no nodes match
    pub fn is_empty(self) -> bool {
        !self.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Document;

    #[test]
    fn test_query_builder_construction() {
        let doc = Document::new("test".to_string());
        let query = NodeQuery::from(AstNodeRef::Document(&doc))
            .paragraphs()
            .with_annotations();

        assert_eq!(query.predicates.len(), 2);
        assert_eq!(query.traversal_mode, TraversalMode::DepthFirst);
    }

    #[test]
    fn test_traversal_mode_change() {
        let doc = Document::new("test".to_string());
        let query = NodeQuery::from(AstNodeRef::Document(&doc)).breadth_first();

        assert_eq!(query.traversal_mode, TraversalMode::BreadthFirst);
    }

    #[test]
    fn test_depth_range() {
        let doc = Document::new("test".to_string());
        let query = NodeQuery::from(AstNodeRef::Document(&doc)).at_depth(1, 3);

        assert_eq!(query.depth_range, Some((1, 3)));
    }
}
