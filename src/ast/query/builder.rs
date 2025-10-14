//! Query Builder - Fluent API for AST queries
//!
//! Provides a Django QuerySet / XPath-like builder pattern for constructing
//! complex AST queries with a readable, chainable API.

use super::node_ref::NodeRef;
use super::predicates::Predicate;
use super::traversal::{TraversalMode, TreeIterator};

/// Fluent query builder for AST traversal and filtering
///
/// Provides chainable methods for constructing complex queries:
///
/// ```rust
/// # use txxt::ast::query::Query;
/// # fn example(root: &txxt::ast::nodes::paragraph::Paragraph) {
/// let results = Query::from(root)
///     .descendants()
///     .filter_type("Paragraph")
///     .filter(|n| n.has_parameter("id"))
///     .collect::<Vec<_>>();
/// # }
/// ```
pub struct Query<'a> {
    iterator: Box<dyn Iterator<Item = NodeRef<'a>> + 'a>,
}

impl<'a> Query<'a> {
    /// Create a query starting from a root node
    pub fn from<N: Into<NodeRef<'a>>>(node: N) -> Self {
        let node_ref = node.into();
        Self {
            iterator: Box::new(std::iter::once(node_ref)),
        }
    }

    /// Traverse all descendants in depth-first preorder
    pub fn descendants(self) -> Self {
        // TODO: Implement using TreeIterator with preorder traversal
        self
    }

    /// Traverse all descendants in depth-first postorder
    pub fn descendants_postorder(self) -> Self {
        // TODO: Implement using TreeIterator with postorder traversal
        self
    }

    /// Traverse all descendants in breadth-first order
    pub fn descendants_breadth_first(self) -> Self {
        // TODO: Implement using TreeIterator with breadth-first traversal
        self
    }

    /// Get direct children only
    pub fn children(self) -> Self {
        // TODO: Implement children-only iteration
        self
    }

    /// Filter by node type
    pub fn filter_type(self, node_type: &'a str) -> Self {
        Self {
            iterator: Box::new(self.iterator.filter(move |node| {
                node.node_type() == node_type
            })),
        }
    }

    /// Filter by parameter existence and value
    pub fn filter_parameter(self, key: &'a str, value: &'a str) -> Self {
        Self {
            iterator: Box::new(self.iterator.filter(move |node| {
                node.parameter(key).map_or(false, |v| v == value)
            })),
        }
    }

    /// Filter by text content containing a substring
    pub fn filter_content_contains(self, substring: &'a str) -> Self {
        Self {
            iterator: Box::new(self.iterator.filter(move |node| {
                node.text_content().contains(substring)
            })),
        }
    }

    /// Filter using a custom predicate
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&NodeRef<'a>) -> bool + 'a,
    {
        Self {
            iterator: Box::new(self.iterator.filter(predicate)),
        }
    }

    /// Select nodes using an XPath-like path expression
    pub fn select(self, _path: &str) -> Self {
        // TODO: Implement path parsing and matching
        self
    }

    /// Get the first matching node
    pub fn first(mut self) -> Option<NodeRef<'a>> {
        self.iterator.next()
    }

    /// Collect all matching nodes
    pub fn collect<B: FromIterator<NodeRef<'a>>>(self) -> B {
        self.iterator.collect()
    }

    /// Map nodes to a different type
    pub fn map<F, T>(self, f: F) -> impl Iterator<Item = T> + 'a
    where
        F: Fn(NodeRef<'a>) -> T + 'a,
    {
        self.iterator.map(f)
    }
}

impl<'a> Iterator for Query<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}
